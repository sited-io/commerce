use deadpool_postgres::Pool;
use jwtk::jwk::RemoteJwksVerifier;
use tonic::{async_trait, Request, Response, Status};
use uuid::Uuid;

use crate::api::peoplesmarkets::commerce::v1::{
    DeleteShopCustomizationRequest, DeleteShopCustomizationResponse,
    GetShopCustomizationRequest, GetShopCustomizationResponse,
    PutBannerImageToShopRequest, PutBannerImageToShopResponse,
    PutLogoImageToShopRequest, PutLogoImageToShopResponse,
    PutShopCustomizationRequest, PutShopCustomizationResponse,
    RemoveBannerImageFromShopRequest, RemoveBannerImageFromShopResponse,
    RemoveLogoImageFromShopRequest, RemoveLogoImageFromShopResponse,
    ShopCustomizationResponse, ShopLayoutType,
};
use crate::api::peoplesmarkets::commerce::v1::shop_customization_service_server::{ShopCustomizationServiceServer, self};
use crate::auth::get_user_id;
use crate::db::DbError;
use crate::images::ImageService;
use crate::model::ShopCustomization;
use crate::parse_uuid;

pub struct ShopCustomizationService {
    pool: Pool,
    verifier: RemoteJwksVerifier,
    image_service: ImageService,
}

impl ShopCustomizationService {
    fn new(
        pool: Pool,
        verifier: RemoteJwksVerifier,
        image_service: ImageService,
    ) -> Self {
        Self {
            pool,
            verifier,
            image_service,
        }
    }

    pub fn build(
        pool: Pool,
        verifier: RemoteJwksVerifier,
        image_service: ImageService,
    ) -> ShopCustomizationServiceServer<Self> {
        let service = Self::new(pool, verifier, image_service);

        ShopCustomizationServiceServer::new(service)
    }

    fn customization_to_response(
        &self,
        shop_customization: ShopCustomization,
    ) -> ShopCustomizationResponse {
        let layout_type =
            ShopLayoutType::from_str_name(&shop_customization.layout_type)
                .map(i32::from)
                .unwrap_or(0);

        ShopCustomizationResponse {
            shop_id: shop_customization.shop_id.to_string(),
            user_id: shop_customization.user_id,
            created_at: u64::try_from(
                shop_customization.created_at.timestamp(),
            )
            .unwrap(),
            updated_at: u64::try_from(
                shop_customization.updated_at.timestamp(),
            )
            .unwrap(),
            logo_image_light_url: self.image_service.get_opt_image_url(
                shop_customization.logo_image_light_url_path,
            ),
            logo_image_dark_url: self
                .image_service
                .get_opt_image_url(shop_customization.logo_image_dark_url_path),

            banner_image_light_url: self.image_service.get_opt_image_url(
                shop_customization.banner_image_light_url_path,
            ),
            banner_image_dark_url: self.image_service.get_opt_image_url(
                shop_customization.banner_image_dark_url_path,
            ),
            primary_color: shop_customization.primary_color,
            layout_type,
        }
    }

    pub fn gen_image_path(user_id: &String, shop_id: &Uuid) -> String {
        format!("{}/{}/{}", user_id, shop_id, Uuid::new_v4())
    }
}

#[async_trait]
impl shop_customization_service_server::ShopCustomizationService
    for ShopCustomizationService
{
    async fn put_shop_customization(
        &self,
        request: Request<PutShopCustomizationRequest>,
    ) -> Result<Response<PutShopCustomizationResponse>, Status> {
        let user_id = get_user_id(request.metadata(), &self.verifier).await?;

        let PutShopCustomizationRequest {
            shop_id,
            primary_color,
            layout_type,
        } = request.into_inner();

        let shop_id = parse_uuid(&shop_id, "shop_id")?;

        let layout_type = ShopLayoutType::from_i32(layout_type)
            .ok_or(Status::invalid_argument("layout_type"))?
            .as_str_name()
            .to_string();

        let shop_customization = ShopCustomization::put(
            &self.pool,
            &shop_id,
            &user_id,
            primary_color,
            layout_type,
        )
        .await?;

        Ok(Response::new(PutShopCustomizationResponse {
            shop_customization: Some(
                self.customization_to_response(shop_customization),
            ),
        }))
    }

    async fn get_shop_customization(
        &self,
        request: Request<GetShopCustomizationRequest>,
    ) -> Result<Response<GetShopCustomizationResponse>, Status> {
        let GetShopCustomizationRequest { shop_id } = request.into_inner();

        let shop_uuid = parse_uuid(&shop_id, "shop_id")?;

        let shop_customization = ShopCustomization::get(&self.pool, &shop_uuid)
            .await?
            .ok_or(Status::not_found(shop_id))?;

        Ok(Response::new(GetShopCustomizationResponse {
            shop_customization: Some(
                self.customization_to_response(shop_customization),
            ),
        }))
    }

    async fn delete_shop_customization(
        &self,
        request: Request<DeleteShopCustomizationRequest>,
    ) -> Result<Response<DeleteShopCustomizationResponse>, Status> {
        let user_id = get_user_id(request.metadata(), &self.verifier).await?;

        let DeleteShopCustomizationRequest { shop_id } = request.into_inner();

        let shop_uuid = parse_uuid(&shop_id, "shop_id")?;

        let shop_customization = ShopCustomization::get(&self.pool, &shop_uuid)
            .await?
            .ok_or(Status::not_found(shop_id))?;

        let mut conn = self.pool.get().await.map_err(DbError::from)?;
        let transaction = conn.transaction().await.map_err(DbError::from)?;

        ShopCustomization::delete(&transaction, &shop_uuid, &user_id).await?;

        if let Some(image_path) = shop_customization.logo_image_light_url_path {
            self.image_service.remove_image(&image_path).await?;
        }
        if let Some(image_path) = shop_customization.banner_image_light_url_path
        {
            self.image_service.remove_image(&image_path).await?;
        }

        transaction.commit().await.map_err(DbError::from)?;

        Ok(Response::new(DeleteShopCustomizationResponse {}))
    }

    async fn put_banner_image_to_shop(
        &self,
        request: Request<PutBannerImageToShopRequest>,
    ) -> Result<Response<PutBannerImageToShopResponse>, Status> {
        let user_id = get_user_id(request.metadata(), &self.verifier).await?;

        let PutBannerImageToShopRequest {
            shop_id,
            image,
            image_dark,
        } = request.into_inner();

        let shop_uuid = parse_uuid(&shop_id, "shop_id")?;

        let shop_customization = ShopCustomization::get(&self.pool, &shop_uuid)
            .await?
            .ok_or_else(|| Status::not_found(shop_id))?;

        let mut image_light_update_path = None;
        let mut image_light_update = None;
        if let Some(image) = image {
            self.image_service.validate_image(&image.data)?;

            if let Some(existing) =
                shop_customization.banner_image_light_url_path
            {
                self.image_service.remove_image(&existing).await?;
            }

            let image_path = Self::gen_image_path(&user_id, &shop_uuid);

            image_light_update_path = Some(Some(image_path));
            image_light_update = Some(image);
        };

        let mut image_dark_update_path = None;
        let mut image_dark_update = None;
        if let Some(image) = image_dark {
            self.image_service.validate_image(&image.data)?;

            if let Some(existing) =
                shop_customization.banner_image_dark_url_path
            {
                self.image_service.remove_image(&existing).await?;
            }

            let image_path = Self::gen_image_path(&user_id, &shop_uuid);

            image_dark_update_path = Some(Some(image_path));
            image_dark_update = Some(image);
        };

        let mut conn = self.pool.get().await.map_err(DbError::from)?;
        let transaction = conn.transaction().await.map_err(DbError::from)?;

        ShopCustomization::update_banner_image_url_paths(
            &transaction,
            &shop_uuid,
            &user_id,
            image_light_update_path.clone(),
            image_dark_update_path.clone(),
        )
        .await?;

        if let (Some(Some(image_path)), Some(image)) =
            (image_light_update_path, image_light_update)
        {
            self.image_service
                .put_image(&image_path, &image.data)
                .await?;
        }
        if let (Some(Some(image_path)), Some(image)) =
            (image_dark_update_path, image_dark_update)
        {
            self.image_service
                .put_image(&image_path, &image.data)
                .await?;
        }

        transaction.commit().await.map_err(DbError::from)?;

        Ok(Response::new(PutBannerImageToShopResponse {}))
    }

    async fn remove_banner_image_from_shop(
        &self,
        request: Request<RemoveBannerImageFromShopRequest>,
    ) -> Result<Response<RemoveBannerImageFromShopResponse>, Status> {
        let user_id = get_user_id(request.metadata(), &self.verifier).await?;

        let RemoveBannerImageFromShopRequest { shop_id } = request.into_inner();

        let shop_id = parse_uuid(&shop_id, "shop_id")?;

        let shop_customization = ShopCustomization::get(&self.pool, &shop_id)
            .await?
            .ok_or_else(|| Status::not_found(""))?;

        let mut conn = self.pool.get().await.map_err(DbError::from)?;
        let transaction = conn.transaction().await.map_err(DbError::from)?;

        ShopCustomization::update_banner_image_url_paths(
            &transaction,
            &shop_id,
            &user_id,
            Some(None),
            Some(None),
        )
        .await?;

        if let Some(image_light_path) =
            shop_customization.banner_image_light_url_path
        {
            self.image_service.remove_image(&image_light_path).await?;
        }
        if let Some(image_dark) = shop_customization.banner_image_dark_url_path
        {
            self.image_service.remove_image(&image_dark).await?;
        }

        transaction.commit().await.map_err(DbError::from)?;

        Ok(Response::new(RemoveBannerImageFromShopResponse {}))
    }

    async fn put_logo_image_to_shop(
        &self,
        request: Request<PutLogoImageToShopRequest>,
    ) -> Result<Response<PutLogoImageToShopResponse>, Status> {
        let user_id = get_user_id(request.metadata(), &self.verifier).await?;

        let PutLogoImageToShopRequest {
            shop_id,
            image,
            image_dark,
        } = request.into_inner();

        let shop_uuid = parse_uuid(&shop_id, "shop_id")?;

        let shop_customization = ShopCustomization::get(&self.pool, &shop_uuid)
            .await?
            .ok_or_else(|| Status::not_found(shop_id))?;

        let mut image_light_update_path = None;
        let mut image_light_update = None;
        if let Some(image) = image {
            self.image_service.validate_image(&image.data)?;

            if let Some(existing) = shop_customization.logo_image_light_url_path
            {
                self.image_service.remove_image(&existing).await?;
            }

            let image_path = Self::gen_image_path(&user_id, &shop_uuid);
            image_light_update_path = Some(Some(image_path));
            image_light_update = Some(image);
        }

        let mut image_dark_update_path = None;
        let mut image_dark_update = None;
        if let Some(image) = image_dark {
            self.image_service.validate_image(&image.data)?;

            if let Some(existing) = shop_customization.logo_image_dark_url_path
            {
                self.image_service.remove_image(&existing).await?;
            }

            let image_path = Self::gen_image_path(&user_id, &shop_uuid);
            image_dark_update_path = Some(Some(image_path));
            image_dark_update = Some(image);
        }

        let mut conn = self.pool.get().await.map_err(DbError::from)?;
        let transaction = conn.transaction().await.map_err(DbError::from)?;

        ShopCustomization::update_logo_image_url_paths(
            &transaction,
            &shop_uuid,
            &user_id,
            image_light_update_path.clone(),
            image_dark_update_path.clone(),
        )
        .await?;

        if let (Some(Some(image_path)), Some(image)) =
            (image_light_update_path, image_light_update)
        {
            self.image_service
                .put_image(&image_path, &image.data)
                .await?;
        }
        if let (Some(Some(image_path)), Some(image)) =
            (image_dark_update_path, image_dark_update)
        {
            self.image_service
                .put_image(&image_path, &image.data)
                .await?;
        }

        transaction.commit().await.map_err(DbError::from)?;

        Ok(Response::new(PutLogoImageToShopResponse {}))
    }

    async fn remove_logo_image_from_shop(
        &self,
        request: Request<RemoveLogoImageFromShopRequest>,
    ) -> Result<Response<RemoveLogoImageFromShopResponse>, Status> {
        let user_id = get_user_id(request.metadata(), &self.verifier).await?;

        let RemoveLogoImageFromShopRequest { shop_id } = request.into_inner();

        let shop_id = parse_uuid(&shop_id, "shop_id")?;

        let shop_customization = ShopCustomization::get(&self.pool, &shop_id)
            .await?
            .ok_or_else(|| Status::not_found(""))?;

        let mut conn = self.pool.get().await.map_err(DbError::from)?;
        let transaction = conn.transaction().await.map_err(DbError::from)?;

        ShopCustomization::update_logo_image_url_paths(
            &transaction,
            &shop_id,
            &user_id,
            Some(None),
            Some(None),
        )
        .await?;

        if let Some(image_path) = shop_customization.logo_image_light_url_path {
            self.image_service.remove_image(&image_path).await?;
        }
        if let Some(image_path) = shop_customization.logo_image_dark_url_path {
            self.image_service.remove_image(&image_path).await?;
        }

        transaction.commit().await.map_err(DbError::from)?;

        Ok(Response::new(RemoveLogoImageFromShopResponse {}))
    }
}
