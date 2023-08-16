use deadpool_postgres::Pool;
use jwtk::{jwk::RemoteJwksVerifier, Claims};
use tonic::{async_trait, Request, Response, Status};
use uuid::Uuid;

use crate::api::peoplesmarkets::commerce::v1::market_booth_service_server;
use crate::api::peoplesmarkets::commerce::v1::market_booth_service_server::MarketBoothServiceServer;
use crate::api::peoplesmarkets::commerce::v1::CreateMarketBoothRequest;
use crate::api::peoplesmarkets::commerce::v1::CreateMarketBoothResponse;
use crate::api::peoplesmarkets::commerce::v1::DeleteMarketBoothRequest;
use crate::api::peoplesmarkets::commerce::v1::DeleteMarketBoothResponse;
use crate::api::peoplesmarkets::commerce::v1::GetMarketBoothRequest;
use crate::api::peoplesmarkets::commerce::v1::GetMarketBoothResponse;
use crate::api::peoplesmarkets::commerce::v1::ListMarketBoothsRequest;
use crate::api::peoplesmarkets::commerce::v1::ListMarketBoothsResponse;
use crate::api::peoplesmarkets::commerce::v1::UpdateMarketBoothRequest;
use crate::api::peoplesmarkets::commerce::v1::UpdateMarketBoothResponse;
use crate::api::peoplesmarkets::pagination::v1::Pagination;
use crate::auth::get_auth_token;
use crate::error::db_err_to_grpc_status;
use crate::model::MarketBooth;
use crate::uuid_err_to_grpc_status;

pub struct MarketBoothService {
    pool: Pool,
    verifier: RemoteJwksVerifier,
}

impl MarketBoothService {
    fn new(pool: Pool, verifier: RemoteJwksVerifier) -> Self {
        Self { pool, verifier }
    }

    pub fn build(
        pool: Pool,
        verifier: RemoteJwksVerifier,
    ) -> MarketBoothServiceServer<Self> {
        MarketBoothServiceServer::new(Self::new(pool, verifier))
    }

    /**
     * Returns limit and offset for requested Pagination or defaults.
     */
    fn paginate(request: Option<Pagination>) -> (u64, u64, Pagination) {
        let mut limit = 10;
        let mut offset = 0;
        let mut pagination = Pagination {
            page: 1,
            size: limit,
        };

        if let Some(request) = request {
            limit = request.size;
            offset = (request.page - 1) * request.size;
            pagination = request;
        }

        (limit, offset, pagination)
    }
}

#[async_trait]
impl market_booth_service_server::MarketBoothService for MarketBoothService {
    async fn create_market_booth(
        &self,
        request: Request<CreateMarketBoothRequest>,
    ) -> Result<Response<CreateMarketBoothResponse>, Status> {
        let token = get_auth_token(request.metadata())
            .ok_or_else(|| Status::unauthenticated(""))?;

        let CreateMarketBoothRequest { name, description } =
            request.into_inner();

        let claims = self.verifier.verify::<Claims<()>>(&token).await.map_err(
            |err| {
                tracing::log::error!("{err}");
                Status::internal(err.to_string())
            },
        )?;

        let user_id = claims
            .claims()
            .sub
            .as_ref()
            .ok_or_else(|| Status::unauthenticated(""))?;

        let created_shop =
            MarketBooth::create(&self.pool, user_id, name, description)
                .await
                .map_err(db_err_to_grpc_status)?;

        Ok(Response::new(CreateMarketBoothResponse {
            market_booth: Some(created_shop.into()),
        }))
    }

    async fn get_market_booth(
        &self,
        request: Request<GetMarketBoothRequest>,
    ) -> Result<Response<GetMarketBoothResponse>, Status> {
        let market_booth_id: Uuid = request
            .into_inner()
            .market_booth_id
            .parse()
            .map_err(|_| uuid_err_to_grpc_status("market_booth_id"))?;

        let found_shop = MarketBooth::get(&self.pool, &market_booth_id)
            .await
            .map_err(db_err_to_grpc_status)?
            .ok_or(Status::not_found(""))?;

        Ok(Response::new(GetMarketBoothResponse {
            market_booth: Some(found_shop.into()),
        }))
    }

    async fn list_market_booths(
        &self,
        request: Request<ListMarketBoothsRequest>,
    ) -> Result<Response<ListMarketBoothsResponse>, Status> {
        let ListMarketBoothsRequest {
            user_id,
            pagination,
            ..
        } = request.into_inner();

        let (limit, offset, pagination) = Self::paginate(pagination);

        let found_shops =
            MarketBooth::list(&self.pool, user_id.as_ref(), limit, offset)
                .await
                .map_err(db_err_to_grpc_status)?;

        Ok(Response::new(ListMarketBoothsResponse {
            market_booths: found_shops.into_iter().map(|s| s.into()).collect(),
            pagination: Some(pagination),
        }))
    }

    async fn update_market_booth(
        &self,
        _request: Request<UpdateMarketBoothRequest>,
    ) -> Result<Response<UpdateMarketBoothResponse>, Status> {
        todo!()
    }

    async fn delete_market_booth(
        &self,
        request: Request<DeleteMarketBoothRequest>,
    ) -> Result<Response<DeleteMarketBoothResponse>, Status> {
        let DeleteMarketBoothRequest { market_booth_id } = request.into_inner();
        let market_booth_id = market_booth_id
            .parse()
            .map_err(|_| uuid_err_to_grpc_status("market_booth_id"))?;

        MarketBooth::delete(&self.pool, &market_booth_id)
            .await
            .map_err(db_err_to_grpc_status)?;

        Ok(Response::new(DeleteMarketBoothResponse {}))
    }
}
