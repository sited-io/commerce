use deadpool_postgres::Pool;
use jwtk::jwk::RemoteJwksVerifier;

use tonic::{async_trait, Request, Response, Status};

use crate::api::peoplesmarkets::commerce::v1::shop_domain_service_server::{
    self, ShopDomainServiceServer,
};
use crate::api::peoplesmarkets::commerce::v1::{
    AddDomainToShopRequest, AddDomainToShopResponse, DomainStatus,
    DomainStatusResponse, GetClientIdForDomainRequest,
    GetClientIdForDomainResponse, GetDomainStatusRequest,
    GetDomainStatusResponse, RemoveDomainFromShopRequest,
    RemoveDomainFromShopResponse, UpdateDomainStatusRequest,
    UpdateDomainStatusResponse,
};
use crate::auth::{get_user_id, verify_service_user};
use crate::db::DbError;
use crate::model::{Shop, ShopDomain};
use crate::parse_uuid;

pub struct ShopDomainService {
    pool: Pool,
    verifier: RemoteJwksVerifier,
}

impl ShopDomainService {
    fn new(pool: Pool, verifier: RemoteJwksVerifier) -> Self {
        Self { pool, verifier }
    }

    pub fn build(
        pool: Pool,
        verifier: RemoteJwksVerifier,
    ) -> ShopDomainServiceServer<Self> {
        let service = Self::new(pool, verifier);

        ShopDomainServiceServer::new(service)
    }

    fn shop_domain_to_response(
        shop_domain: ShopDomain,
    ) -> Result<DomainStatusResponse, Status> {
        Ok(DomainStatusResponse {
            shop_id: shop_domain.shop_id.to_string(),
            domain: shop_domain.domain,
            status: DomainStatus::from_str_name(&shop_domain.status)
                .ok_or(Status::internal(""))?
                .into(),
            client_id: shop_domain.client_id,
        })
    }
}

#[async_trait]
impl shop_domain_service_server::ShopDomainService for ShopDomainService {
    async fn add_domain_to_shop(
        &self,
        request: Request<AddDomainToShopRequest>,
    ) -> Result<Response<AddDomainToShopResponse>, Status> {
        let user_id = get_user_id(request.metadata(), &self.verifier).await?;

        let AddDomainToShopRequest { shop_id, domain } = request.into_inner();

        let shop_uuid = parse_uuid(&shop_id, "shop_id")?;

        let found_domain =
            ShopDomain::get_for_user(&self.pool, &user_id, &shop_uuid).await?;

        if let Some(found_domain) = found_domain {
            if found_domain.domain == domain {
                return Ok(Response::new(AddDomainToShopResponse {}));
            } else {
                return Err(Status::already_exists(found_domain.domain));
            }
        }

        ShopDomain::create(&self.pool, &user_id, &shop_uuid, &domain).await?;

        Ok(Response::new(AddDomainToShopResponse {}))
    }

    async fn get_domain_status(
        &self,
        request: Request<GetDomainStatusRequest>,
    ) -> Result<Response<GetDomainStatusResponse>, Status> {
        let GetDomainStatusRequest { shop_id } = request.into_inner();

        let shop_uuid = parse_uuid(&shop_id, "shop_id")?;

        let found_domain = ShopDomain::get(&self.pool, &shop_uuid)
            .await?
            .ok_or(Status::not_found(shop_id))?;

        Ok(Response::new(GetDomainStatusResponse {
            domain_status: Some(Self::shop_domain_to_response(found_domain)?),
        }))
    }

    async fn get_client_id_for_domain(
        &self,
        request: Request<GetClientIdForDomainRequest>,
    ) -> Result<Response<GetClientIdForDomainResponse>, Status> {
        let GetClientIdForDomainRequest { domain } = request.into_inner();

        let found_domain =
            ShopDomain::get_by_domain(&self.pool, &domain).await?;

        Ok(Response::new(GetClientIdForDomainResponse {
            client_id: found_domain.and_then(|d| d.client_id),
        }))
    }

    async fn update_domain_status(
        &self,
        request: Request<UpdateDomainStatusRequest>,
    ) -> Result<Response<UpdateDomainStatusResponse>, Status> {
        verify_service_user(request.metadata(), &self.verifier).await?;

        let UpdateDomainStatusRequest {
            shop_id,
            domain,
            status,
            client_id,
        } = request.into_inner();

        let shop_uuid = parse_uuid(&shop_id, "shop_id")?;

        let mut conn = self.pool.get().await.map_err(DbError::from)?;
        let transaction = conn.transaction().await.map_err(DbError::from)?;

        let status = if status < 1 {
            return Err(Status::invalid_argument("status"));
        } else {
            DomainStatus::from_i32(status)
                .ok_or(Status::invalid_argument("status"))?
                .as_str_name()
                .to_owned()
        };

        ShopDomain::update(
            &transaction,
            &shop_uuid,
            &domain,
            &status,
            &client_id,
        )
        .await?;

        Shop::update_domain(&transaction, &shop_uuid, Some(domain)).await?;

        transaction.commit().await.map_err(DbError::from)?;

        Ok(Response::new(UpdateDomainStatusResponse {}))
    }

    async fn remove_domain_from_shop(
        &self,
        request: Request<RemoveDomainFromShopRequest>,
    ) -> Result<Response<RemoveDomainFromShopResponse>, Status> {
        let user_id = get_user_id(request.metadata(), &self.verifier).await?;

        let RemoveDomainFromShopRequest { shop_id, domain } =
            request.into_inner();

        let shop_uuid = parse_uuid(&shop_id, "shop_id")?;

        let mut conn = self.pool.get().await.map_err(DbError::from)?;
        let transaction = conn.transaction().await.map_err(DbError::from)?;

        ShopDomain::delete(&transaction, &user_id, &shop_uuid, &domain).await?;

        Shop::update_domain(&transaction, &shop_uuid, None).await?;

        transaction.commit().await.map_err(DbError::from)?;

        Ok(Response::new(RemoveDomainFromShopResponse {}))
    }
}
