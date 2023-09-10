use deadpool_postgres::Pool;
use jwtk::jwk::RemoteJwksVerifier;
use tonic::{async_trait, Request, Response, Status};
use uuid::Uuid;

use crate::api::peoplesmarkets::commerce::v1::offer_service_server::{
    self, OfferServiceServer,
};
use crate::api::peoplesmarkets::commerce::v1::{
    AddImageToOfferRequest, AddImageToOfferResponse, CreateOfferRequest,
    CreateOfferResponse, Currency, DeleteOfferRequest, DeleteOfferResponse,
    GetOfferRequest, GetOfferResponse, ListOffersRequest, ListOffersResponse,
    OfferImageResponse, OfferResponse, Price, PriceBillingScheme, PriceType,
    RemoveImageFromOfferRequest, RemoveImageFromOfferResponse,
    UpdateOfferRequest, UpdateOfferResponse,
};
use crate::auth::get_user_id;
use crate::images::ImageService;
use crate::model::{Offer, OfferImage, OfferPrice};
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
            price: offer.price.map(|p| Price {
                price_id: p.offer_price_id.to_string(),
                currency: Self::currency_i32(p.currency),
                price_type: Self::price_type_i32(p.price_type),
                billing_scheme: Self::billing_scheme_i32(p.billing_scheme),
                unit_amont: p.unit_amount,
            }),
            market_booth_name: offer.market_booth_name,
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

    fn currency_i32(currency: String) -> i32 {
        let currency = Currency::from_str_name(&currency);
        match currency {
            Some(Currency::Unspecified) => 0,
            Some(Currency::Eur) => 1,
            None => 0,
        }
    }

    fn currency_or_default<'a>(currency: i32) -> &'a str {
        let currency = Currency::from_i32(currency);
        match currency {
            Some(Currency::Unspecified) => Currency::Eur.as_str_name(),
            Some(Currency::Eur) => Currency::Eur.as_str_name(),
            None => Currency::Eur.as_str_name(),
        }
    }

    fn price_type_i32(price_type: String) -> i32 {
        let price_type = PriceType::from_str_name(&price_type);
        match price_type {
            Some(PriceType::Unspecified) => 0,
            Some(PriceType::OneTime) => 1,
            None => 0,
        }
    }

    fn price_type_or_default<'a>(price_type: i32) -> &'a str {
        let price_type = PriceType::from_i32(price_type);
        match price_type {
            Some(PriceType::Unspecified) => PriceType::OneTime.as_str_name(),
            Some(PriceType::OneTime) => PriceType::OneTime.as_str_name(),
            None => PriceType::OneTime.as_str_name(),
        }
    }

    fn billing_scheme_i32(billing_scheme: String) -> i32 {
        let billing_scheme = PriceBillingScheme::from_str_name(&billing_scheme);
        match billing_scheme {
            Some(PriceBillingScheme::Unspecified) => 0,
            Some(PriceBillingScheme::PerUnit) => 1,
            None => 0,
        }
    }

    fn billing_scheme_or_default<'a>(billing_scheme: i32) -> &'a str {
        let billing_scheme = PriceBillingScheme::from_i32(billing_scheme);
        match billing_scheme {
            Some(PriceBillingScheme::Unspecified) => {
                PriceBillingScheme::PerUnit.as_str_name()
            }
            Some(PriceBillingScheme::PerUnit) => {
                PriceBillingScheme::PerUnit.as_str_name()
            }
            None => PriceBillingScheme::PerUnit.as_str_name(),
        }
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
            price,
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

        if let Some(price) = price {
            OfferPrice::create(
                &self.pool,
                &created_offer.offer_id,
                &user_id,
                Self::currency_or_default(price.currency),
                Self::price_type_or_default(price.price_type),
                Self::billing_scheme_or_default(price.billing_scheme),
                price.unit_amont,
            )
            .await?;
        }

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
            price,
        } = request.into_inner();

        let offer_id = parse_uuid(&offer_id, "offer_id")?;

        if let Some(price) = price {
            OfferPrice::upsert_for_offer(
                &self.pool,
                &user_id,
                &offer_id,
                Self::currency_or_default(price.currency),
                Self::price_type_or_default(price.price_type),
                Self::billing_scheme_or_default(price.billing_scheme),
                price.unit_amont,
            )
            .await?;
        } else {
            OfferPrice::delete_for_offer(&self.pool, &user_id, &offer_id)
                .await?;
        }

        let updated_offer =
            Offer::update(&self.pool, &user_id, &offer_id, name, description)
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
