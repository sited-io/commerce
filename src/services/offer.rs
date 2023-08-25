use deadpool_postgres::Pool;
use jwtk::jwk::RemoteJwksVerifier;
use jwtk::Claims;
use tonic::{async_trait, Request, Response, Status};

use crate::api::peoplesmarkets::commerce::v1::offer_service_server::{
    self, OfferServiceServer,
};
use crate::api::peoplesmarkets::commerce::v1::{
    CreateOfferRequest, CreateOfferResponse, DeleteOfferRequest,
    DeleteOfferResponse, GetOfferRequest, GetOfferResponse, ListOffersRequest,
    ListOffersResponse, UpdateOfferRequest, UpdateOfferResponse,
};
use crate::auth::get_auth_token;
use crate::error::db_err_to_grpc_status;
use crate::model::Offer;
use crate::parse_uuid;

use super::paginate;

pub struct OfferService {
    pool: Pool,
    verifier: RemoteJwksVerifier,
}

impl OfferService {
    fn new(pool: Pool, verifier: RemoteJwksVerifier) -> Self {
        Self { pool, verifier }
    }

    pub fn build(
        pool: Pool,
        verifier: RemoteJwksVerifier,
    ) -> OfferServiceServer<Self> {
        OfferServiceServer::new(Self::new(pool, verifier))
    }
}

#[async_trait]
impl offer_service_server::OfferService for OfferService {
    async fn create_offer(
        &self,
        request: Request<CreateOfferRequest>,
    ) -> Result<Response<CreateOfferResponse>, Status> {
        let token = get_auth_token(request.metadata())
            .ok_or_else(|| Status::unauthenticated(""))?;

        let CreateOfferRequest {
            market_booth_id,
            name,
            description,
        } = request.into_inner();

        let claims = self
            .verifier
            .verify::<Claims<()>>(&token)
            .await
            .map_err(|err| Status::unauthenticated(err.to_string()))?;

        let user_id = claims
            .claims()
            .sub
            .as_ref()
            .ok_or_else(|| Status::unauthenticated(""))?;

        let market_booth_id = parse_uuid(market_booth_id, "market_booth_id")?;

        let created_offer = Offer::create(
            &self.pool,
            market_booth_id,
            user_id,
            name,
            description,
        )
        .await
        .map_err(db_err_to_grpc_status)?;

        Ok(Response::new(CreateOfferResponse {
            offer: Some(created_offer.into()),
        }))
    }

    async fn get_offer(
        &self,
        request: Request<GetOfferRequest>,
    ) -> Result<Response<GetOfferResponse>, Status> {
        let offer_id = parse_uuid(request.into_inner().offer_id, "offer_id")?;

        let found_offer = Offer::get(&self.pool, &offer_id)
            .await
            .map_err(db_err_to_grpc_status)?
            .ok_or(Status::not_found(""))?;

        Ok(Response::new(GetOfferResponse {
            offer: Some(found_offer.into()),
        }))
    }

    async fn list_offers(
        &self,
        request: Request<ListOffersRequest>,
    ) -> Result<Response<ListOffersResponse>, Status> {
        let ListOffersRequest {
            market_booth_id,
            user_id,
            pagination,
            ..
        } = request.into_inner();

        let (limit, offset, pagination) = paginate(pagination);

        let market_booth_id = match market_booth_id {
            Some(id) => Some(parse_uuid(id, "market_booth_id")?),
            None => None,
        };

        let found_offers = Offer::list(
            &self.pool,
            market_booth_id,
            user_id.as_ref(),
            limit,
            offset,
        )
        .await
        .map_err(db_err_to_grpc_status)?;

        Ok(Response::new(ListOffersResponse {
            offers: found_offers.into_iter().map(|o| o.into()).collect(),
            pagination: Some(pagination),
        }))
    }

    async fn update_offer(
        &self,
        request: Request<UpdateOfferRequest>,
    ) -> Result<Response<UpdateOfferResponse>, Status> {
        let UpdateOfferRequest {
            offer_id,
            name,
            description,
        } = request.into_inner();

        let updated_offer = Offer::update(
            &self.pool,
            &parse_uuid(offer_id, "offer_id")?,
            name,
            description,
        )
        .await
        .map_err(db_err_to_grpc_status)?;

        Ok(Response::new(UpdateOfferResponse {
            offer: Some(updated_offer.into()),
        }))
    }

    async fn delete_offer(
        &self,
        request: Request<DeleteOfferRequest>,
    ) -> Result<Response<DeleteOfferResponse>, Status> {
        let offer_id = parse_uuid(request.into_inner().offer_id, "offer_id")?;

        Offer::delete(&self.pool, &offer_id)
            .await
            .map_err(db_err_to_grpc_status)?;

        Ok(Response::new(DeleteOfferResponse {}))
    }
}
