use deadpool_postgres::Pool;
use jwtk::jwk::RemoteJwksVerifier;
use tonic::{async_trait, Request, Response, Status};
use uuid::Uuid;

use crate::api::peoplesmarkets::commerce::v1::market_booth_service_server::{
    self, MarketBoothServiceServer,
};
use crate::api::peoplesmarkets::commerce::v1::{
    CreateMarketBoothRequest, CreateMarketBoothResponse,
    DeleteMarketBoothRequest, DeleteMarketBoothResponse, GetMarketBoothRequest,
    GetMarketBoothResponse, ListMarketBoothsRequest, ListMarketBoothsResponse,
    MarketBoothResponse, MarketBoothsFilterField, MarketBoothsOrderByField,
    RemoveImageFromMarketBoothRequest, RemoveImageFromMarketBoothResponse,
    UpdateImageOfMarketBoothRequest, UpdateImageOfMarketBoothResponse,
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

    fn to_response(&self, market_booth: MarketBooth) -> MarketBoothResponse {
        MarketBoothResponse {
            market_booth_id: market_booth.market_booth_id.to_string(),
            user_id: market_booth.user_id,
            created_at: market_booth.created_at.timestamp(),
            updated_at: market_booth.updated_at.timestamp(),
            name: market_booth.name,
            description: market_booth.description,
            image_url: self
                .image_service
                .get_opt_image_url(market_booth.image_url_path),
            platform_fee_percent: market_booth.platform_fee_percent,
            minimum_platform_fee_cent: market_booth.minimum_platform_fee_cent,
        }
    }

    fn gen_image_path(user_id: &String, market_booth_id: &Uuid) -> String {
        format!("{}/{}/{}", user_id, market_booth_id, Uuid::new_v4())
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
            description,
            platform_fee_percent,
            minimum_platform_fee_cent,
        } = request.into_inner();

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
            name,
            description,
            platform_fee_percent,
            minimum_platform_fee_cent,
        )
        .await?;

        Ok(Response::new(CreateMarketBoothResponse {
            market_booth: Some(self.to_response(created_shop)),
        }))
    }

    async fn get_market_booth(
        &self,
        request: Request<GetMarketBoothRequest>,
    ) -> Result<Response<GetMarketBoothResponse>, Status> {
        let market_booth_id = parse_uuid(
            &request.into_inner().market_booth_id,
            "market_booth_id",
        )?;

        let found_market_booth = MarketBooth::get(&self.pool, &market_booth_id)
            .await?
            .ok_or(Status::not_found(""))?;

        Ok(Response::new(GetMarketBoothResponse {
            market_booth: Some(self.to_response(found_market_booth)),
        }))
    }

    async fn list_market_booths(
        &self,
        request: Request<ListMarketBoothsRequest>,
    ) -> Result<Response<ListMarketBoothsResponse>, Status> {
        let ListMarketBoothsRequest {
            user_id,
            pagination,
            filter,
            order_by,
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

        let found_market_booths = MarketBooth::list(
            &self.pool,
            user_id.as_ref(),
            limit,
            offset,
            filter,
            order_by,
        )
        .await?;

        Ok(Response::new(ListMarketBoothsResponse {
            market_booths: found_market_booths
                .into_iter()
                .map(|mb| self.to_response(mb))
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

        let updated_market_booth = MarketBooth::update(
            &self.pool,
            &user_id,
            &market_booth_id,
            name,
            description,
            platform_fee_percent,
            minimum_platform_fee_cent,
        )
        .await?;

        Ok(Response::new(UpdateMarketBoothResponse {
            market_booth: Some(self.to_response(updated_market_booth)),
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

    async fn update_image_of_market_booth(
        &self,
        request: Request<UpdateImageOfMarketBoothRequest>,
    ) -> Result<Response<UpdateImageOfMarketBoothResponse>, Status> {
        let user_id = get_user_id(request.metadata(), &self.verifier).await?;

        let UpdateImageOfMarketBoothRequest {
            market_booth_id,
            image,
        } = request.into_inner();

        let image = match image {
            None => return Err(Status::invalid_argument("image")),
            Some(i) => i,
        };

        let market_booth_id = parse_uuid(&market_booth_id, "market_booth_id")?;

        let market_booth = MarketBooth::get(&self.pool, &market_booth_id)
            .await?
            .ok_or_else(|| Status::not_found(""))?;

        self.image_service.validate_image(&image.data)?;

        if let Some(existing) = market_booth.image_url_path {
            self.image_service.remove_image(&existing).await?;
        }

        let image_path = Self::gen_image_path(&user_id, &market_booth_id);

        let mut conn = self.pool.get().await.map_err(DbError::from)?;
        let transaction = conn.transaction().await.map_err(DbError::from)?;

        MarketBooth::update_image_url_path(
            &transaction,
            &user_id,
            &market_booth_id,
            Some(image_path.clone()),
        )
        .await?;

        self.image_service
            .put_image(&image_path, &image.data)
            .await?;

        transaction.commit().await.map_err(DbError::from)?;

        Ok(Response::new(UpdateImageOfMarketBoothResponse {}))
    }

    async fn remove_image_from_market_booth(
        &self,
        request: Request<RemoveImageFromMarketBoothRequest>,
    ) -> Result<Response<RemoveImageFromMarketBoothResponse>, Status> {
        let user_id = get_user_id(request.metadata(), &self.verifier).await?;

        let market_booth_id = parse_uuid(
            &request.into_inner().market_booth_id,
            "market_booth_id",
        )?;

        let market_booth = MarketBooth::get(&self.pool, &market_booth_id)
            .await?
            .ok_or_else(|| Status::not_found(""))?;

        let mut conn = self.pool.get().await.map_err(DbError::from)?;
        let transaction = conn.transaction().await.map_err(DbError::from)?;

        if let Some(image_path) = market_booth.image_url_path {
            self.image_service.remove_image(&image_path).await?;
        }

        MarketBooth::update_image_url_path(
            &transaction,
            &user_id,
            &market_booth_id,
            None,
        )
        .await?;

        transaction.commit().await.map_err(DbError::from)?;

        Ok(Response::new(RemoveImageFromMarketBoothResponse {}))
    }
}
