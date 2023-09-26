use deadpool_postgres::Pool;
use jwtk::jwk::RemoteJwksVerifier;
use tonic::{async_trait, Request, Response, Status};

use crate::api::peoplesmarkets::commerce::v1::market_booth_service_server::{
    self, MarketBoothServiceServer,
};
use crate::api::peoplesmarkets::commerce::v1::{
    CreateMarketBoothRequest, CreateMarketBoothResponse,
    DeleteMarketBoothRequest, DeleteMarketBoothResponse, GetMarketBoothRequest,
    GetMarketBoothResponse, GetShopByDomainRequest, GetShopByDomainResponse,
    GetShopBySlugRequest, GetShopBySlugResponse, ListShopsRequest,
    ListShopsResponse, MarketBoothResponse, MarketBoothsFilterField,
    MarketBoothsOrderByField, ShopCustomizationResponse,
    UpdateMarketBoothRequest, UpdateMarketBoothResponse,
};
use crate::api::peoplesmarkets::ordering::v1::Direction;
use crate::auth::get_user_id;
use crate::db::DbError;
use crate::images::ImageService;
use crate::model::MarketBooth;
use crate::parse_uuid;

use super::paginate;

pub struct MarketBoothService {
    pool: Pool,
    verifier: RemoteJwksVerifier,
    image_service: ImageService,
    allowed_min_platform_fee_percent: u32,
    allowed_min_minimum_platform_fee_cent: u32,
}

impl MarketBoothService {
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
    ) -> MarketBoothServiceServer<Self> {
        let service = Self::new(
            pool,
            verifier,
            image_service,
            allowed_min_platform_fee_percent,
            allowed_min_minimum_platform_fee_cent,
        );
        MarketBoothServiceServer::new(service)
    }

    fn shop_to_response(
        &self,
        market_booth: MarketBooth,
    ) -> MarketBoothResponse {
        let customization = self.customization_to_response(&market_booth);

        MarketBoothResponse {
            market_booth_id: market_booth.market_booth_id.to_string(),
            user_id: market_booth.user_id,
            created_at: market_booth.created_at.timestamp(),
            updated_at: market_booth.updated_at.timestamp(),
            name: market_booth.name,
            slug: market_booth.slug,
            description: market_booth.description,
            platform_fee_percent: market_booth.platform_fee_percent,
            minimum_platform_fee_cent: market_booth.minimum_platform_fee_cent,
            customization,
            domain: market_booth.domain,
        }
    }

    fn customization_to_response(
        &self,
        shop: &MarketBooth,
    ) -> Option<ShopCustomizationResponse> {
        shop.customization.clone().map(|customization| {
            ShopCustomizationResponse {
                shop_id: shop.market_booth_id.to_string(),
                user_id: shop.user_id.to_string(),
                created_at: 0,
                updated_at: 0,
                banner_image_url: self
                    .image_service
                    .get_opt_image_url(customization.banner_image_url_path),
                logo_image_url: self
                    .image_service
                    .get_opt_image_url(customization.logo_image_url_path),
                header_background_color_light: customization
                    .header_background_color_light,
                header_background_color_dark: customization
                    .header_background_color_dark,
                header_content_color_light: customization
                    .header_content_color_light,
                header_content_color_dark: customization
                    .header_content_color_dark,
                secondary_background_color_light: customization
                    .secondary_background_color_light,
                secondary_background_color_dark: customization
                    .secondary_background_color_dark,
                secondary_content_color_light: customization
                    .secondary_content_color_light,
                secondary_content_color_dark: customization
                    .secondary_content_color_dark,
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
impl market_booth_service_server::MarketBoothService for MarketBoothService {
    async fn create_market_booth(
        &self,
        request: Request<CreateMarketBoothRequest>,
    ) -> Result<Response<CreateMarketBoothResponse>, Status> {
        let user_id = get_user_id(request.metadata(), &self.verifier).await?;

        let CreateMarketBoothRequest {
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

        let created_shop = MarketBooth::create(
            &self.pool,
            &user_id,
            &name,
            &slug,
            description,
            platform_fee_percent,
            minimum_platform_fee_cent,
        )
        .await?;

        Ok(Response::new(CreateMarketBoothResponse {
            market_booth: Some(self.shop_to_response(created_shop)),
        }))
    }

    async fn get_market_booth(
        &self,
        request: Request<GetMarketBoothRequest>,
    ) -> Result<Response<GetMarketBoothResponse>, Status> {
        let GetMarketBoothRequest {
            market_booth_id,
            extended,
        } = request.into_inner();

        let market_booth_id = parse_uuid(&market_booth_id, "market_booth_id")?;

        let extended = extended.unwrap_or(false);

        let found_market_booth =
            MarketBooth::get(&self.pool, &market_booth_id, extended)
                .await?
                .ok_or(Status::not_found(""))?;

        Ok(Response::new(GetMarketBoothResponse {
            market_booth: Some(self.shop_to_response(found_market_booth)),
        }))
    }

    async fn get_shop_by_slug(
        &self,
        request: Request<GetShopBySlugRequest>,
    ) -> Result<Response<GetShopBySlugResponse>, Status> {
        let GetShopBySlugRequest { slug } = request.into_inner();

        let found_shop = MarketBooth::get_by_slug(&self.pool, &slug)
            .await?
            .ok_or(Status::not_found(""))?;

        Ok(Response::new(GetShopBySlugResponse {
            market_booth: Some(self.shop_to_response(found_shop)),
        }))
    }

    async fn get_shop_by_domain(
        &self,
        request: Request<GetShopByDomainRequest>,
    ) -> Result<Response<GetShopByDomainResponse>, Status> {
        let GetShopByDomainRequest { domain } = request.into_inner();

        let found_shop = MarketBooth::get_by_domain(&self.pool, &domain)
            .await?
            .ok_or(Status::not_found(""))?;

        Ok(Response::new(GetShopByDomainResponse {
            market_booth: Some(self.shop_to_response(found_shop)),
        }))
    }

    async fn list_shops(
        &self,
        request: Request<ListShopsRequest>,
    ) -> Result<Response<ListShopsResponse>, Status> {
        let ListShopsRequest {
            user_id,
            pagination,
            filter,
            order_by,
            extended,
        } = request.into_inner();

        let (limit, offset, pagination) = paginate(pagination)?;

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
                        MarketBoothsFilterField::from_i32(f.field)
                            .ok_or(Status::invalid_argument("filter.field"))?,
                        f.query,
                    ))
                }
            }
            None => None,
        };

        let order_by = match order_by {
            Some(o) => Some((
                MarketBoothsOrderByField::from_i32(o.field)
                    .ok_or(Status::invalid_argument("order_by.field"))?,
                Direction::from_i32(o.direction)
                    .ok_or(Status::invalid_argument("order_by.direction"))?,
            )),
            None => None,
        };

        let extended = extended.unwrap_or(false);

        let found_market_booths = MarketBooth::list(
            &self.pool,
            user_id.as_ref(),
            limit,
            offset,
            filter,
            order_by,
            extended,
        )
        .await?;

        Ok(Response::new(ListShopsResponse {
            market_booths: found_market_booths
                .into_iter()
                .map(|mb| self.shop_to_response(mb))
                .collect(),
            pagination: Some(pagination),
        }))
    }

    async fn update_market_booth(
        &self,
        request: Request<UpdateMarketBoothRequest>,
    ) -> Result<Response<UpdateMarketBoothResponse>, Status> {
        let user_id = get_user_id(request.metadata(), &self.verifier).await?;

        let UpdateMarketBoothRequest {
            market_booth_id,
            name,
            description,
            platform_fee_percent,
            minimum_platform_fee_cent,
            slug,
        } = request.into_inner();

        let market_booth_id = parse_uuid(&market_booth_id, "market_booth_id")?;

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

        let updated_market_booth = MarketBooth::update(
            &self.pool,
            &user_id,
            &market_booth_id,
            name,
            slug,
            description,
            platform_fee_percent,
            minimum_platform_fee_cent,
        )
        .await?;

        Ok(Response::new(UpdateMarketBoothResponse {
            market_booth: Some(self.shop_to_response(updated_market_booth)),
        }))
    }

    async fn delete_market_booth(
        &self,
        request: Request<DeleteMarketBoothRequest>,
    ) -> Result<Response<DeleteMarketBoothResponse>, Status> {
        let user_id = get_user_id(request.metadata(), &self.verifier).await?;

        let market_booth_id = parse_uuid(
            &request.into_inner().market_booth_id,
            "market_booth_id",
        )?;

        let mut conn = self.pool.get().await.map_err(DbError::from)?;
        let transaction = conn.transaction().await.map_err(DbError::from)?;

        let deleted_market_booth =
            MarketBooth::delete(&transaction, &user_id, &market_booth_id)
                .await?;

        if let Some(image_path) = deleted_market_booth.image_url_path {
            self.image_service.remove_image(&image_path).await?;
        }

        transaction.commit().await.map_err(DbError::from)?;

        Ok(Response::new(DeleteMarketBoothResponse {}))
    }
}
