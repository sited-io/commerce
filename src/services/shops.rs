use tonic::{async_trait, Request, Response, Status};

use crate::api::peoplesmarkets::commerce::v1::{
    shop_service_server::{self, ShopServiceServer},
    CreateShopRequest, CreateShopResponse, DeleteShopRequest, DeleteShopResponse, GetShopRequest,
    GetShopResponse, ListShopsRequest, ListShopsResponse, UpdateShopRequest, UpdateShopResponse,
};

#[derive(Debug, Clone)]
pub struct ShopsService {}

impl ShopsService {
    fn new() -> Self {
        Self {}
    }

    pub fn build() -> ShopServiceServer<Self> {
        ShopServiceServer::new(Self::new())
    }
}

#[async_trait]
impl shop_service_server::ShopService for ShopsService {
    async fn create_shop(
        &self,
        _request: Request<CreateShopRequest>,
    ) -> Result<Response<CreateShopResponse>, Status> {
        todo!()
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
