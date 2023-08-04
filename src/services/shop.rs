use deadpool_postgres::Pool;
use jwtk::{jwk::RemoteJwksVerifier, Claims};
use tonic::{async_trait, Request, Response, Status};

use crate::{
    api::peoplesmarkets::commerce::v1::{
        shop_service_server::{self, ShopServiceServer},
        CreateShopRequest, CreateShopResponse, DeleteShopRequest,
        DeleteShopResponse, GetShopRequest, GetShopResponse, ListShopsRequest,
        ListShopsResponse, UpdateShopRequest, UpdateShopResponse,
    },
    auth::get_auth_token,
    model,
};

pub struct ShopsService {
    pool: Pool,
    verifier: RemoteJwksVerifier,
}

impl ShopsService {
    fn new(pool: Pool, verifier: RemoteJwksVerifier) -> Self {
        Self { pool, verifier }
    }

    pub fn build(
        pool: Pool,
        verifier: RemoteJwksVerifier,
    ) -> ShopServiceServer<Self> {
        ShopServiceServer::new(Self::new(pool, verifier))
    }
}

#[async_trait]
impl shop_service_server::ShopService for ShopsService {
    async fn create_shop(
        &self,
        request: Request<CreateShopRequest>,
    ) -> Result<Response<CreateShopResponse>, Status> {
        let token = get_auth_token(request.metadata())
            .ok_or_else(|| Status::unauthenticated(""))?;

        let CreateShopRequest { name, description } = request.into_inner();

        let claims = self.verifier.verify::<Claims<()>>(&token).await.map_err(
            |err| {
                tracing::log::error!("{err}");
                Status::internal(err.to_string())
            },
        )?;

        tracing::log::info!("Got Claims: {claims:?}");

        let user_id = claims
            .claims()
            .sub
            .as_ref()
            .ok_or_else(|| Status::unauthenticated(""))?;

        let created_shop =
            model::create_shop(&self.pool, &user_id, &name, &description)
                .await
                .map_err(|err| Status::internal(err))?;

        Ok(Response::new(CreateShopResponse {
            shop: Some(created_shop.into()),
        }))
    }

    async fn get_shop(
        &self,
        _request: Request<GetShopRequest>,
    ) -> Result<Response<GetShopResponse>, Status> {
        todo!()
    }

    async fn list_shops(
        &self,
        _request: Request<ListShopsRequest>,
    ) -> Result<Response<ListShopsResponse>, Status> {
        todo!()
    }

    async fn update_shop(
        &self,
        _request: Request<UpdateShopRequest>,
    ) -> Result<Response<UpdateShopResponse>, Status> {
        todo!()
    }

    async fn delete_shop(
        &self,
        _request: Request<DeleteShopRequest>,
    ) -> Result<Response<DeleteShopResponse>, Status> {
        todo!()
    }
}
