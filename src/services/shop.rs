use deadpool_postgres::Pool;
use jwtk::jwk::RemoteJwksVerifier;
use tonic::{async_trait, Request, Response, Status};

use crate::api::sited_io::commerce::v1::shop_service_server::{
    self, ShopServiceServer,
};
use crate::api::sited_io::commerce::v1::{
    CreateShopRequest, CreateShopResponse, DeleteShopRequest,
    DeleteShopResponse, GetShopRequest, GetShopResponse, ListShopsRequest,
    ListShopsResponse, ShopCustomizationResponse, ShopLayoutType, ShopResponse,
    ShopsFilterField, ShopsOrderByField, UpdateShopRequest, UpdateShopResponse,
};
use crate::api::sited_io::types::v1::Direction;
use crate::auth::get_user_id;
use crate::db::DbError;
use crate::images::ImageService;
use crate::model::{Offer, Shop, ShopCustomization};
use crate::parse_uuid;

use super::get_limit_offset_from_pagination;

pub struct ShopService {
    pool: Pool,
    verifier: RemoteJwksVerifier,
    image_service: ImageService,
    allowed_min_platform_fee_percent: u32,
    allowed_min_minimum_platform_fee_cent: u32,
}

impl ShopService {
    const SLUG_CHARS: [char; 65] = [
        '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e',
        'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's',
        't', 'u', 'v', 'w', 'x', 'y', 'z', 'A', 'B', 'C', 'D', 'E', 'F', 'G',
        'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U',
        'V', 'W', 'X', 'Y', 'Z', '-', '+', '_', '!',
    ];

    fn new(
        pool: Pool,
        verifier: RemoteJwksVerifier,
        image_service: ImageService,
        allowed_min_platform_fee_percent: u32,
        allowed_min_minimum_platform_fee_cent: u32,
    ) -> Self {
        Self {
            pool,
            verifier,
            image_service,
            allowed_min_platform_fee_percent,
            allowed_min_minimum_platform_fee_cent,
        }
    }

    pub fn build(
        pool: Pool,
        verifier: RemoteJwksVerifier,
        image_service: ImageService,
        allowed_min_platform_fee_percent: u32,
        allowed_min_minimum_platform_fee_cent: u32,
    ) -> ShopServiceServer<Self> {
        let service = Self::new(
            pool,
            verifier,
            image_service,
            allowed_min_platform_fee_percent,
            allowed_min_minimum_platform_fee_cent,
        );
        ShopServiceServer::new(service)
    }

    fn shop_to_response(&self, shop: Shop) -> ShopResponse {
        let customization = self.customization_to_response(&shop);

        ShopResponse {
            shop_id: shop.shop_id.to_string(),
            user_id: shop.user_id,
            created_at: u64::try_from(shop.created_at.timestamp()).unwrap(),
            updated_at: u64::try_from(shop.updated_at.timestamp()).unwrap(),
            name: shop.name,
            slug: shop.slug,
            description: shop.description,
            platform_fee_percent: shop.platform_fee_percent,
            minimum_platform_fee_cent: shop.minimum_platform_fee_cent,
            customization,
            domain: shop.domain,
            is_active: shop.is_active,
            contact_email_address: shop.contact_email_address,
            client_id: shop.client_id,
            website_id: shop.website_id,
        }
    }

    fn customization_to_response(
        &self,
        shop: &Shop,
    ) -> Option<ShopCustomizationResponse> {
        shop.customization.clone().map(|customization| {
            let layout_type =
                ShopLayoutType::from_str_name(&customization.layout_type)
                    .map(i32::from)
                    .unwrap_or(0);

            ShopCustomizationResponse {
                shop_id: shop.shop_id.to_string(),
                user_id: shop.user_id.to_string(),
                created_at: 0,
                updated_at: 0,
                logo_image_light_url: self
                    .image_service
                    .get_opt_image_url(customization.logo_image_light_url_path),
                logo_image_dark_url: self
                    .image_service
                    .get_opt_image_url(customization.logo_image_dark_url_path),
                banner_image_light_url: self.image_service.get_opt_image_url(
                    customization.banner_image_light_url_path,
                ),
                banner_image_dark_url: self.image_service.get_opt_image_url(
                    customization.banner_image_dark_url_path,
                ),
                primary_color: customization.primary_color,
                layout_type,
            }
        })
    }

    fn validate_slug(slug: &str) -> Result<(), Status> {
        for char in slug.chars() {
            if !Self::SLUG_CHARS.contains(&char) {
                return Err(Status::invalid_argument(format!(
                    "invalid character '{}'",
                    char
                )));
            }
        }

        Ok(())
    }
}

#[async_trait]
impl shop_service_server::ShopService for ShopService {
    async fn create_shop(
        &self,
        request: Request<CreateShopRequest>,
    ) -> Result<Response<CreateShopResponse>, Status> {
        let user_id = get_user_id(request.metadata(), &self.verifier).await?;

        let CreateShopRequest {
            name,
            slug,
            description,
            platform_fee_percent,
            minimum_platform_fee_cent,
        } = request.into_inner();

        Self::validate_slug(&slug)?;

        let platform_fee_percent = match platform_fee_percent {
            Some(pfp) => {
                if pfp < self.allowed_min_platform_fee_percent || pfp >= 100 {
                    return Err(Status::invalid_argument(
                        "platform_fee_percent",
                    ));
                }
                pfp
            }
            None => self.allowed_min_platform_fee_percent,
        };

        let minimum_platform_fee_cent = match minimum_platform_fee_cent {
            Some(mpfc) => {
                if mpfc < self.allowed_min_minimum_platform_fee_cent {
                    return Err(Status::invalid_argument(
                        "minimum_platform_fee_cent",
                    ));
                }
                mpfc
            }
            None => self.allowed_min_minimum_platform_fee_cent,
        };

        let created_shop = Shop::create(
            &self.pool,
            &user_id,
            &name,
            &slug,
            description,
            platform_fee_percent,
            minimum_platform_fee_cent,
        )
        .await?;

        ShopCustomization::create(&self.pool, &created_shop.shop_id, &user_id)
            .await?;

        Ok(Response::new(CreateShopResponse {
            shop: Some(self.shop_to_response(created_shop)),
        }))
    }

    async fn get_shop(
        &self,
        request: Request<GetShopRequest>,
    ) -> Result<Response<GetShopResponse>, Status> {
        let user_id =
            get_user_id(request.metadata(), &self.verifier).await.ok();

        let GetShopRequest {
            shop_id,
            extended,
            domain,
            slug,
            owner,
            website_id,
        } = request.into_inner();

        let extended = extended.unwrap_or(false);

        let found_shop = match (shop_id, domain, slug, website_id) {
            (Some(shop_id), _, _, _) => {
                let shop_id = parse_uuid(&shop_id, "shop_id")?;
                Shop::get(&self.pool, &shop_id, user_id.as_ref(), extended)
                    .await?
            }
            (_, Some(domain), _, _) => {
                Shop::get_by_domain(
                    &self.pool,
                    &domain,
                    user_id.as_ref(),
                    extended,
                )
                .await?
            }
            (_, _, Some(slug), _) => {
                Shop::get_by_slug(&self.pool, &slug, user_id.as_ref(), extended)
                    .await?
            }
            (_, _, _, Some(website_id)) => {
                Shop::get_by_website_id(&self.pool, &website_id).await?
            }
            (None, None, None, None) => {
                return Err(Status::invalid_argument(
                    "provide one of shop_id, slug or domain",
                ))
            }
        }
        .ok_or(Status::not_found(""))?;

        if matches!(owner, Some(owner) if found_shop.user_id != owner) {
            return Err(Status::not_found(""));
        }

        Ok(Response::new(GetShopResponse {
            shop: Some(self.shop_to_response(found_shop)),
        }))
    }

    async fn list_shops(
        &self,
        request: Request<ListShopsRequest>,
    ) -> Result<Response<ListShopsResponse>, Status> {
        let request_user_id =
            get_user_id(request.metadata(), &self.verifier).await.ok();

        let ListShopsRequest {
            user_id,
            pagination,
            filter,
            order_by,
            extended,
        } = request.into_inner();

        let (limit, offset, pagination) =
            get_limit_offset_from_pagination(pagination)?;

        if filter.is_none() && order_by.is_none() {
            return Err(Status::invalid_argument("filter,order_by"));
        }

        let filter = match filter {
            Some(f) => {
                if f.field < 1 {
                    return Err(Status::invalid_argument("filter.field"));
                } else if f.query.trim().is_empty() {
                    return Err(Status::invalid_argument("filter.query"));
                } else {
                    Some((
                        ShopsFilterField::from_i32(f.field)
                            .ok_or(Status::invalid_argument("filter.field"))?,
                        f.query,
                    ))
                }
            }
            None => None,
        };

        let order_by = match order_by {
            Some(o) => Some((
                ShopsOrderByField::from_i32(o.field)
                    .ok_or(Status::invalid_argument("order_by.field"))?,
                Direction::from_i32(o.direction)
                    .ok_or(Status::invalid_argument("order_by.direction"))?,
            )),
            None => None,
        };

        let extended = extended.unwrap_or(false);

        let found_shops = Shop::list(
            &self.pool,
            user_id.as_ref(),
            limit.into(),
            offset.into(),
            filter,
            order_by,
            extended,
            request_user_id.as_ref(),
        )
        .await?;

        Ok(Response::new(ListShopsResponse {
            shops: found_shops
                .into_iter()
                .map(|mb| self.shop_to_response(mb))
                .collect(),
            pagination: Some(pagination),
        }))
    }

    async fn update_shop(
        &self,
        request: Request<UpdateShopRequest>,
    ) -> Result<Response<UpdateShopResponse>, Status> {
        let user_id = get_user_id(request.metadata(), &self.verifier).await?;

        let UpdateShopRequest {
            shop_id,
            name,
            description,
            platform_fee_percent,
            minimum_platform_fee_cent,
            slug,
            is_active,
            contact_email_address,
        } = request.into_inner();

        if let Some(ref slug) = slug {
            Self::validate_slug(slug)?;
        }

        let shop_id = parse_uuid(&shop_id, "shop_id")?;

        if matches!(
            platform_fee_percent,
            Some(pfp) if pfp < self.allowed_min_platform_fee_percent || pfp >= 100,
        ) {
            return Err(Status::invalid_argument("platform_fee_percent"));
        }

        if matches!(
            minimum_platform_fee_cent,
            Some(mpfc) if mpfc < self.allowed_min_minimum_platform_fee_cent,
        ) {
            return Err(Status::invalid_argument("minimum_platform_fee_cent"));
        }

        if let Some(slug) = slug.as_ref() {
            Self::validate_slug(slug)?;
        }

        if matches!(is_active, Some(false)) {
            Offer::deactivate_for_shop(&self.pool, &user_id, &shop_id).await?;
        }

        let updated_shop = Shop::update(
            &self.pool,
            &user_id,
            &shop_id,
            name,
            slug,
            description,
            platform_fee_percent,
            minimum_platform_fee_cent,
            is_active,
            contact_email_address,
        )
        .await?;

        Ok(Response::new(UpdateShopResponse {
            shop: Some(self.shop_to_response(updated_shop)),
        }))
    }

    async fn delete_shop(
        &self,
        request: Request<DeleteShopRequest>,
    ) -> Result<Response<DeleteShopResponse>, Status> {
        let user_id = get_user_id(request.metadata(), &self.verifier).await?;

        let shop_id = parse_uuid(&request.into_inner().shop_id, "shop_id")?;

        let found_shop_customization =
            ShopCustomization::get(&self.pool, &shop_id).await?;

        let mut conn = self.pool.get().await.map_err(DbError::from)?;
        let transaction = conn.transaction().await.map_err(DbError::from)?;

        if let Some(found_shop_customization) = found_shop_customization {
            ShopCustomization::delete(&transaction, &shop_id, &user_id).await?;

            if let Some(image_path) =
                found_shop_customization.logo_image_light_url_path
            {
                self.image_service.remove_image(&image_path).await?;
            }
            if let Some(image_path) =
                found_shop_customization.banner_image_light_url_path
            {
                self.image_service.remove_image(&image_path).await?;
            }
        }

        Shop::delete(&transaction, &user_id, &shop_id).await?;

        transaction.commit().await.map_err(DbError::from)?;

        Ok(Response::new(DeleteShopResponse {}))
    }
}
