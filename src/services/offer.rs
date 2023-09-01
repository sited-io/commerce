use deadpool_postgres::Pool;
use jwtk::jwk::RemoteJwksVerifier;
use tonic::{async_trait, Request, Response, Status};
use uuid::Uuid;

use crate::api::peoplesmarkets::commerce::v1::offer_service_server::{
    self, OfferServiceServer,
};
use crate::api::peoplesmarkets::commerce::v1::{
    AddImageToOfferRequest, AddImageToOfferResponse, CreateOfferRequest,
    CreateOfferResponse, DeleteOfferRequest, DeleteOfferResponse,
    GetOfferRequest, GetOfferResponse, ListOffersRequest, ListOffersResponse,
    OfferImageResponse, OfferResponse, RemoveImageFromOfferRequest,
    RemoveImageFromOfferResponse, UpdateOfferRequest, UpdateOfferResponse,
};
use crate::auth::get_user_id;
use crate::images::ImageService;
use crate::model::{Offer, OfferImage};
use crate::parse_uuid;

use super::paginate;

pub struct OfferService {
    pool: Pool,
    verifier: RemoteJwksVerifier,
    image_service: ImageService,
}

impl OfferService {
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
    ) -> OfferServiceServer<Self> {
        OfferServiceServer::new(Self::new(pool, verifier, image_service))
    }

    fn to_response(&self, offer: Offer) -> OfferResponse {
        OfferResponse {
            offer_id: offer.offer_id.to_string(),
            market_booth_id: offer.market_booth_id.to_string(),
            user_id: offer.user_id,
            created_at: offer.created_at.timestamp(),
            updated_at: offer.updated_at.timestamp(),
            name: offer.name,
            description: offer.description,
            images: offer
                .images
                .into_iter()
                .map(|oi| OfferImageResponse {
                    offer_image_id: oi.offer_image_id.to_string(),
                    image_url: self
                        .image_service
                        .get_image_url(&oi.image_url_path),
                    ordering: oi.ordering,
                })
                .collect(),
        }
    }

    fn build_image_path(
        user_id: &String,
        market_booth_id: &Uuid,
        offer_id: &Uuid,
        offer_image_id: &Uuid,
    ) -> String {
        format!(
            "/{}/{}/{}/{}",
            user_id, market_booth_id, offer_id, offer_image_id
        )
    }
}

#[async_trait]
impl offer_service_server::OfferService for OfferService {
    async fn create_offer(
        &self,
        request: Request<CreateOfferRequest>,
    ) -> Result<Response<CreateOfferResponse>, Status> {
        let user_id = get_user_id(request.metadata(), &self.verifier).await?;

        let CreateOfferRequest {
            market_booth_id,
            name,
            description,
        } = request.into_inner();

        let market_booth_id = parse_uuid(&market_booth_id, "market_booth_id")?;

        let created_offer = Offer::create(
            &self.pool,
            market_booth_id,
            &user_id,
            name,
            description,
        )
        .await?;

        Ok(Response::new(CreateOfferResponse {
            offer: Some(self.to_response(created_offer)),
        }))
    }

    async fn get_offer(
        &self,
        request: Request<GetOfferRequest>,
    ) -> Result<Response<GetOfferResponse>, Status> {
        let offer_id = parse_uuid(&request.into_inner().offer_id, "offer_id")?;

        let found_offer = Offer::get(&self.pool, &offer_id)
            .await?
            .ok_or(Status::not_found(""))?;

        Ok(Response::new(GetOfferResponse {
            offer: Some(self.to_response(found_offer)),
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
            filter,
            ..
        } = request.into_inner();

        let (limit, offset, pagination) = paginate(pagination)?;

        let (name_query, description_query) = match filter {
            Some(filter) => {
                if filter.field == 1 {
                    (Some(filter.query), None)
                } else if filter.field == 2 {
                    (None, Some(filter.query))
                } else if filter.field == 3 {
                    (Some(filter.query.clone()), Some(filter.query))
                } else {
                    (None, None)
                }
            }
            None => (None, None),
        };

        let market_booth_id = match market_booth_id {
            Some(id) => Some(parse_uuid(&id, "market_booth_id")?),
            None => None,
        };

        let found_offers =
            if name_query.is_none() && description_query.is_none() {
                Offer::list(
                    &self.pool,
                    market_booth_id,
                    user_id.as_ref(),
                    limit,
                    offset,
                )
                .await?
            } else {
                Offer::search(
                    &self.pool,
                    limit,
                    offset,
                    name_query,
                    description_query,
                )
                .await
                .map_or_else(|err| err.ignore_to_ts_query(Vec::new()), Ok)?
            };

        Ok(Response::new(ListOffersResponse {
            offers: found_offers
                .into_iter()
                .map(|o| self.to_response(o))
                .collect(),
            pagination: Some(pagination),
        }))
    }

    async fn update_offer(
        &self,
        request: Request<UpdateOfferRequest>,
    ) -> Result<Response<UpdateOfferResponse>, Status> {
        let user_id = get_user_id(request.metadata(), &self.verifier).await?;

        let UpdateOfferRequest {
            offer_id,
            name,
            description,
        } = request.into_inner();

        let updated_offer = Offer::update(
            &self.pool,
            &user_id,
            &parse_uuid(&offer_id, "offer_id")?,
            name,
            description,
        )
        .await?;

        Ok(Response::new(UpdateOfferResponse {
            offer: Some(self.to_response(updated_offer)),
        }))
    }

    async fn delete_offer(
        &self,
        request: Request<DeleteOfferRequest>,
    ) -> Result<Response<DeleteOfferResponse>, Status> {
        let user_id = get_user_id(request.metadata(), &self.verifier).await?;
        let offer_id = parse_uuid(&request.into_inner().offer_id, "offer_id")?;

        Offer::delete(&self.pool, &user_id, &offer_id).await?;

        Ok(Response::new(DeleteOfferResponse {}))
    }

    async fn add_image_to_offer(
        &self,
        request: Request<AddImageToOfferRequest>,
    ) -> Result<Response<AddImageToOfferResponse>, Status> {
        let user_id = get_user_id(request.metadata(), &self.verifier).await?;

        let AddImageToOfferRequest {
            offer_id,
            image,
            ordering,
        } = request.into_inner();

        let image = image.ok_or_else(|| Status::invalid_argument("image"))?;

        let image_data = ImageService::decode_base64(&image.data)?;
        self.image_service.validate_image(&image_data)?;

        let offer_id = parse_uuid(&offer_id, "offer_id")?;

        let offer = Offer::get(&self.pool, &offer_id)
            .await?
            .ok_or_else(|| Status::not_found("offer"))?;

        let offer_image_id = Uuid::new_v4();
        let image_path = &Self::build_image_path(
            &user_id,
            &offer.market_booth_id,
            &offer_id,
            &offer_image_id,
        );

        // TODO: ensure consitency of separate storages
        self.image_service
            .put_image(image_path, &image_data)
            .await?;

        OfferImage::create(
            &self.pool,
            &offer_image_id,
            &offer_id,
            &user_id,
            image_path,
            ordering,
        )
        .await?;
        // TODO: ensure consitency of separate storages

        Ok(Response::new(AddImageToOfferResponse {}))
    }

    async fn remove_image_from_offer(
        &self,
        request: Request<RemoveImageFromOfferRequest>,
    ) -> Result<Response<RemoveImageFromOfferResponse>, Status> {
        let user_id = get_user_id(request.metadata(), &self.verifier).await?;

        let offer_image_id =
            parse_uuid(&request.into_inner().offer_image_id, "offer_image_id")?;

        // TODO: ensure consitency of separate storages
        let offer_image =
            OfferImage::get(&self.pool, &offer_image_id, Some(&user_id))
                .await?
                .ok_or_else(|| Status::not_found("offer_image"))?;

        self.image_service
            .remove_image(&offer_image.image_url_path)
            .await?;
        OfferImage::delete(&self.pool, &user_id, &offer_image_id).await?;

        Ok(Response::new(RemoveImageFromOfferResponse {}))
    }
}
