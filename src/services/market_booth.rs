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
use crate::images::ImageService;
use crate::model::MarketBooth;
use crate::parse_uuid;

use super::paginate;

pub struct MarketBoothService {
    pool: Pool,
    verifier: RemoteJwksVerifier,
    image_service: ImageService,
}

impl MarketBoothService {
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
    ) -> MarketBoothServiceServer<Self> {
        let service = Self::new(pool, verifier, image_service);
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
        }
    }

    fn gen_image_path(user_id: &String, market_booth_id: &Uuid) -> String {
        format!("/{}/{}/{}", user_id, market_booth_id, Uuid::new_v4())
    }
}

#[async_trait]
impl market_booth_service_server::MarketBoothService for MarketBoothService {
    async fn create_market_booth(
        &self,
        request: Request<CreateMarketBoothRequest>,
    ) -> Result<Response<CreateMarketBoothResponse>, Status> {
        let user_id = get_user_id(request.metadata(), &self.verifier).await?;

        let CreateMarketBoothRequest { name, description } =
            request.into_inner();

        let created_shop =
            MarketBooth::create(&self.pool, &user_id, name, description)
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

        tracing::log::info!("{:?}", filter);
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

        let order_by = if filter.is_none() {
            let order_by =
                order_by.ok_or(Status::invalid_argument("order_by"))?;

            Some((
                MarketBoothsOrderByField::from_i32(order_by.field)
                    .ok_or(Status::invalid_argument("order_by.field"))?,
                Direction::from_i32(order_by.direction)
                    .ok_or(Status::invalid_argument("order_by.direction"))?,
            ))
        } else {
            None
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
        } = request.into_inner();

        let market_booth_id = parse_uuid(&market_booth_id, "market_booth_id")?;

        let updated_market_booth = MarketBooth::update(
            &self.pool,
            &user_id,
            &market_booth_id,
            name,
            description,
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

        // TODO: ensure consitency of separate storages
        let deleted_market_booth =
            MarketBooth::delete(&self.pool, &user_id, &market_booth_id).await?;

        if let Some(image_path) = deleted_market_booth.image_url_path {
            self.image_service.remove_image(&image_path).await?;
        }
        // TODO: ensure consitency of separate storages

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

        let image_data = ImageService::decode_base64(&image.data)?;
        self.image_service.validate_image(&image_data)?;

        if let Some(existing) = market_booth.image_url_path {
            self.image_service.remove_image(&existing).await?;
        }

        let image_path = Self::gen_image_path(&user_id, &market_booth_id);

        // TODO: ensure consitency of separate storages
        self.image_service
            .put_image(&image_path, &image_data)
            .await?;

        MarketBooth::update_image_url_path(
            &self.pool,
            &user_id,
            &market_booth_id,
            Some(image_path),
        )
        .await?;
        // TODO: ensure consitency of separate storages

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

        // TODO: ensure consitency of separate storages
        let market_booth = MarketBooth::get(&self.pool, &market_booth_id)
            .await?
            .ok_or_else(|| Status::not_found(""))?;
        if let Some(image_path) = market_booth.image_url_path {
            self.image_service.remove_image(&image_path).await?;
        }
        MarketBooth::update_image_url_path(
            &self.pool,
            &user_id,
            &market_booth_id,
            None,
        )
        .await?;
        // TODO: ensure consitency of separate storages

        Ok(Response::new(RemoveImageFromMarketBoothResponse {}))
    }
}
