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
    OfferImageResponse, OfferResponse, OfferType, Price, PriceBillingScheme,
    PriceType, PutPriceToOfferRequest, PutPriceToOfferResponse, Recurring,
    RecurringInterval, RemoveImageFromOfferRequest,
    RemoveImageFromOfferResponse, RemovePriceFromOfferRequest,
    RemovePriceFromOfferResponse, UpdateOfferRequest, UpdateOfferResponse,
};
use crate::auth::get_user_id;
use crate::db::DbError;
use crate::images::ImageService;
use crate::model::{
    Offer, OfferImage, OfferImageAsRel, OfferPrice, OfferPriceAsRel,
};
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

    fn offer_to_response(&self, offer: Offer) -> Result<OfferResponse, Status> {
        let price = match offer.price {
            Some(p) => Some(self.offer_price_to_response(p)?),
            None => None,
        };

        let r#type = offer
            .type_
            .and_then(|t| OfferType::from_str_name(&t).map(i32::from))
            .unwrap_or(0);

        Ok(OfferResponse {
            offer_id: offer.offer_id.to_string(),
            shop_id: offer.shop_id.to_string(),
            shop_name: offer.shop_name,
            user_id: offer.user_id,
            created_at: offer.created_at.timestamp(),
            updated_at: offer.updated_at.timestamp(),
            name: offer.name,
            description: offer.description,
            is_active: offer.is_active,
            images: self.offer_images_to_response(offer.images),
            price,
            r#type,
            is_featured: offer.is_featured,
            shop_slug: offer.shop_slug,
            shop_domain: offer.shop_domain,
        })
    }

    fn offer_images_to_response(
        &self,
        offer_images: Vec<OfferImageAsRel>,
    ) -> Vec<OfferImageResponse> {
        offer_images
            .into_iter()
            .map(|oi| OfferImageResponse {
                offer_image_id: oi.offer_image_id.to_string(),
                image_url: self.image_service.get_image_url(&oi.image_url_path),
                ordering: oi.ordering,
            })
            .collect()
    }

    fn offer_price_to_response(
        &self,
        offer_price: OfferPriceAsRel,
    ) -> Result<Price, Status> {
        let recurring = match (
            offer_price.recurring_interval,
            offer_price.recurring_interval_count,
            offer_price.trial_period_days,
        ) {
            (Some(interval), Some(interval_count), trial_period_days) => {
                Some(Recurring {
                    interval: RecurringInterval::from_str_name(&interval)
                        .ok_or(Status::internal(""))?
                        .into(),
                    interval_count,
                    trial_period_days,
                })
            }
            _ => None,
        };

        Ok(Price {
            currency: Currency::from_str_name(&offer_price.currency)
                .ok_or(Status::internal(""))?
                .into(),
            price_type: PriceType::from_str_name(&offer_price.price_type)
                .ok_or(Status::internal(""))?
                .into(),
            billing_scheme: PriceBillingScheme::from_str_name(
                &offer_price.billing_scheme,
            )
            .ok_or(Status::internal(""))?
            .into(),
            unit_amount: offer_price.unit_amount,
            recurring,
        })
    }

    fn get_offer_type(type_: i32) -> Result<OfferType, Status> {
        if type_ < 1 {
            Err(Status::invalid_argument("type"))
        } else {
            OfferType::from_i32(type_).ok_or(Status::invalid_argument("type"))
        }
    }

    fn validate_price(price: &Price) -> Result<(), Status> {
        if price.currency() == Currency::Unspecified {
            return Err(Status::invalid_argument("price.currency"));
        }

        if price.price_type() == PriceType::Unspecified {
            return Err(Status::invalid_argument("price.price_type"));
        }

        if price.billing_scheme() == PriceBillingScheme::Unspecified {
            return Err(Status::invalid_argument("price.billing_scheme"));
        }

        if price.price_type == i32::from(PriceType::Recurring) {
            if let Some(recurring) = price.recurring.as_ref() {
                if recurring.interval < 1 {
                    Err(Status::invalid_argument("price.recurring.interval"))
                } else {
                    Ok(())
                }
            } else {
                Err(Status::invalid_argument("price.recurring"))
            }
        } else {
            Ok(())
        }
    }

    fn build_image_path(
        user_id: &String,
        shop_id: &Uuid,
        offer_id: &Uuid,
        offer_image_id: &Uuid,
    ) -> String {
        format!("{}/{}/{}/{}", user_id, shop_id, offer_id, offer_image_id)
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
            shop_id,
            name,
            description,
            r#type,
            is_featured,
        } = request.into_inner();

        let shop_id = parse_uuid(&shop_id, "shop_id")?;

        let type_ = Self::get_offer_type(r#type)?;

        let created_offer = Offer::create(
            &self.pool,
            shop_id,
            &user_id,
            name,
            description,
            type_.as_str_name(),
            is_featured,
        )
        .await?;

        Ok(Response::new(CreateOfferResponse {
            offer: Some(self.offer_to_response(created_offer)?),
        }))
    }

    async fn get_offer(
        &self,
        request: Request<GetOfferRequest>,
    ) -> Result<Response<GetOfferResponse>, Status> {
        let user_id =
            get_user_id(request.metadata(), &self.verifier).await.ok();
        let offer_id = parse_uuid(&request.into_inner().offer_id, "offer_id")?;

        let found_offer = Offer::get(&self.pool, &offer_id, user_id.as_ref())
            .await?
            .ok_or(Status::not_found(""))?;

        Ok(Response::new(GetOfferResponse {
            offer: Some(self.offer_to_response(found_offer)?),
        }))
    }

    async fn list_offers(
        &self,
        request: Request<ListOffersRequest>,
    ) -> Result<Response<ListOffersResponse>, Status> {
        let request_user_id =
            get_user_id(request.metadata(), &self.verifier).await.ok();

        let ListOffersRequest {
            shop_id,
            user_id,
            pagination,
            filter,
            order_by,
        } = request.into_inner();

        let (limit, offset, pagination) = paginate(pagination)?;

        if filter.is_none() && order_by.is_none() && shop_id.is_none() {
            return Err(Status::invalid_argument("filter,order_by"));
        }

        let filter = filter.map(|f| (f.field(), f.query));

        let order_by = order_by.map(|o| (o.field(), o.direction()));

        let shop_id = match shop_id {
            Some(id) => Some(parse_uuid(&id, "shop_id")?),
            None => None,
        };

        let found_offers = Offer::list(
            &self.pool,
            shop_id,
            user_id.as_ref(),
            limit,
            offset,
            filter,
            order_by,
            request_user_id.as_ref(),
        )
        .await?;

        let mut offers = Vec::with_capacity(found_offers.len());

        for offer in found_offers {
            offers.push(self.offer_to_response(offer)?);
        }

        Ok(Response::new(ListOffersResponse {
            offers,
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
            is_active,
            r#type,
            is_featured,
        } = request.into_inner();

        let offer_id = parse_uuid(&offer_id, "offer_id")?;

        let type_ = match r#type {
            Some(t) => Some(Self::get_offer_type(t)?.as_str_name()),
            None => None,
        };

        let updated_offer = Offer::update(
            &self.pool,
            &user_id,
            &offer_id,
            name,
            description,
            is_active,
            type_,
            is_featured,
        )
        .await?;

        Ok(Response::new(UpdateOfferResponse {
            offer: Some(self.offer_to_response(updated_offer)?),
        }))
    }

    async fn delete_offer(
        &self,
        request: Request<DeleteOfferRequest>,
    ) -> Result<Response<DeleteOfferResponse>, Status> {
        let user_id = get_user_id(request.metadata(), &self.verifier).await?;
        let offer_id = parse_uuid(&request.into_inner().offer_id, "offer_id")?;

        let mut conn = self.pool.get().await.map_err(DbError::from)?;
        let transaction = conn.transaction().await.map_err(DbError::from)?;

        let found_images =
            OfferImage::list(&transaction, &offer_id, &user_id).await?;

        for found_image in found_images {
            OfferImage::delete(
                &transaction,
                &user_id,
                &found_image.offer_image_id,
            )
            .await?;

            self.image_service
                .remove_image(&found_image.image_url_path)
                .await?;
        }

        Offer::delete(&transaction, &user_id, &offer_id).await?;

        transaction.commit().await.map_err(DbError::from)?;

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

        self.image_service.validate_image(&image.data)?;

        let offer_id = parse_uuid(&offer_id, "offer_id")?;

        let offer = Offer::get_for_user(&self.pool, &user_id, &offer_id)
            .await?
            .ok_or_else(|| Status::not_found("offer"))?;

        let offer_image_id = Uuid::new_v4();
        let image_path = &Self::build_image_path(
            &user_id,
            &offer.shop_id,
            &offer_id,
            &offer_image_id,
        );

        let mut conn = self.pool.get().await.map_err(DbError::from)?;
        let transaction = conn.transaction().await.map_err(DbError::from)?;

        OfferImage::create(
            &transaction,
            &offer_image_id,
            &offer_id,
            &user_id,
            image_path,
            ordering,
        )
        .await?;

        self.image_service
            .put_image(image_path, &image.data)
            .await?;

        transaction.commit().await.map_err(DbError::from)?;

        Ok(Response::new(AddImageToOfferResponse {}))
    }

    async fn remove_image_from_offer(
        &self,
        request: Request<RemoveImageFromOfferRequest>,
    ) -> Result<Response<RemoveImageFromOfferResponse>, Status> {
        let user_id = get_user_id(request.metadata(), &self.verifier).await?;

        let offer_image_id =
            parse_uuid(&request.into_inner().offer_image_id, "offer_image_id")?;

        let offer_image =
            OfferImage::get(&self.pool, &offer_image_id, Some(&user_id))
                .await?
                .ok_or_else(|| Status::not_found("offer_image"))?;

        let mut conn = self.pool.get().await.map_err(DbError::from)?;
        let transaction = conn.transaction().await.map_err(DbError::from)?;

        OfferImage::delete(&transaction, &user_id, &offer_image_id).await?;

        self.image_service
            .remove_image(&offer_image.image_url_path)
            .await?;

        transaction.commit().await.map_err(DbError::from)?;

        Ok(Response::new(RemoveImageFromOfferResponse {}))
    }

    async fn put_price_to_offer(
        &self,
        request: Request<PutPriceToOfferRequest>,
    ) -> Result<Response<PutPriceToOfferResponse>, Status> {
        let user_id = get_user_id(request.metadata(), &self.verifier).await?;

        let PutPriceToOfferRequest { offer_id, price } = request.into_inner();

        let offer_id = parse_uuid(&offer_id, "offer_id")?;
        let price = price.ok_or(Status::invalid_argument("price"))?;

        Self::validate_price(&price)?;

        let currency = price.currency().as_str_name();
        let price_type = price.price_type().as_str_name();
        let billing_scheme = price.billing_scheme().as_str_name();
        let unit_amount = price.unit_amount;
        let recurring_interval =
            price.recurring.as_ref().map(|r| r.interval().as_str_name());
        let recurring_interval_count =
            price.recurring.as_ref().map(|r| r.interval_count);
        let trial_period_days =
            price.recurring.and_then(|r| r.trial_period_days);

        let found_price =
            OfferPrice::get_by_offer_id(&self.pool, &offer_id).await?;

        if found_price.is_some() {
            OfferPrice::put(
                &self.pool,
                &user_id,
                &offer_id,
                currency,
                price_type,
                billing_scheme,
                unit_amount,
                recurring_interval,
                recurring_interval_count,
                trial_period_days,
            )
            .await?;
        } else {
            OfferPrice::create(
                &self.pool,
                &offer_id,
                &user_id,
                currency,
                price_type,
                billing_scheme,
                unit_amount,
                recurring_interval,
                recurring_interval_count,
                trial_period_days,
            )
            .await?;
        }

        Ok(Response::new(PutPriceToOfferResponse {}))
    }

    async fn remove_price_from_offer(
        &self,
        request: Request<RemovePriceFromOfferRequest>,
    ) -> Result<Response<RemovePriceFromOfferResponse>, Status> {
        let user_id = get_user_id(request.metadata(), &self.verifier).await?;

        let RemovePriceFromOfferRequest { offer_id } = request.into_inner();

        let offer_id = parse_uuid(&offer_id, "offer_id")?;

        OfferPrice::delete(&self.pool, &user_id, &offer_id).await?;

        Ok(Response::new(RemovePriceFromOfferResponse {}))
    }
}
