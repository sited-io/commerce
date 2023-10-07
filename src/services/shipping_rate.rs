use deadpool_postgres::Pool;
use jwtk::jwk::RemoteJwksVerifier;
use tonic::{async_trait, Request, Response, Status};

use crate::api::peoplesmarkets::commerce::v1::shipping_rate_service_server::{
    self, ShippingRateServiceServer,
};
use crate::api::peoplesmarkets::commerce::v1::{
    AddShippingRateToOfferRequest, AddShippingRateToOfferResponse, Currency,
    ListShippingRatesRequest, ListShippingRatesResponse,
    RemoveShippingRateFromOfferRequest, RemoveShippingRateFromOfferResponse,
    ShippingCountry, ShippingRateResponse,
};
use crate::auth::get_user_id;
use crate::model::ShippingRate;
use crate::parse_uuid;

use super::{get_limit_and_offset, get_pagination_response};

pub struct ShippingRateService {
    pool: Pool,
    verifier: RemoteJwksVerifier,
}

impl ShippingRateService {
    fn new(pool: Pool, verifier: RemoteJwksVerifier) -> Self {
        Self { pool, verifier }
    }

    pub fn build(
        pool: Pool,
        verifier: RemoteJwksVerifier,
    ) -> ShippingRateServiceServer<Self> {
        let service = Self::new(pool, verifier);

        ShippingRateServiceServer::new(service)
    }

    fn to_response(
        &self,
        shipping_rate: ShippingRate,
    ) -> Result<ShippingRateResponse, Status> {
        let country = ShippingCountry::from_str_name(&shipping_rate.country)
            .ok_or(Status::internal(format!(
                "error parsing country '{}'",
                shipping_rate.country
            )))?
            .try_into()
            .unwrap();

        let currency = Currency::from_str_name(&shipping_rate.currency)
            .ok_or(Status::internal(format!(
                "error parsing currency '{}'",
                shipping_rate.currency
            )))?
            .try_into()
            .unwrap();

        Ok(ShippingRateResponse {
            shipping_rate_id: shipping_rate.shipping_rate_id.to_string(),
            offer_id: shipping_rate.offer_id.to_string(),
            user_id: shipping_rate.user_id,
            country,
            amount: shipping_rate.amount,
            currency,
        })
    }
}

#[async_trait]
impl shipping_rate_service_server::ShippingRateService for ShippingRateService {
    async fn add_shipping_rate_to_offer(
        &self,
        request: Request<AddShippingRateToOfferRequest>,
    ) -> Result<Response<AddShippingRateToOfferResponse>, Status> {
        let user_id = get_user_id(request.metadata(), &self.verifier).await?;

        let AddShippingRateToOfferRequest {
            offer_id,
            country,
            amount,
            currency,
        } = request.into_inner();

        let offer_uuid = parse_uuid(&offer_id, "offer_id")?;
        let country = ShippingCountry::from_i32(country)
            .ok_or(Status::invalid_argument("country"))?
            .as_str_name();
        let currency = Currency::from_i32(currency)
            .ok_or(Status::invalid_argument("currency"))?
            .as_str_name();

        ShippingRate::create(
            &self.pool,
            &offer_uuid,
            &user_id,
            country,
            amount,
            currency,
        )
        .await?;

        Ok(Response::new(AddShippingRateToOfferResponse {}))
    }

    async fn list_shipping_rates(
        &self,
        request: Request<ListShippingRatesRequest>,
    ) -> Result<Response<ListShippingRatesResponse>, Status> {
        let ListShippingRatesRequest {
            offer_id,
            pagination,
            order_by,
        } = request.into_inner();

        let offer_uuid = parse_uuid(&offer_id, "offer_id")?;

        let (limit, offset) = get_limit_and_offset(pagination)?;

        let order_by = order_by.map(|o| (o.field(), o.direction()));

        let (found_shipping_rates, total_elements) = ShippingRate::list(
            &self.pool,
            &offer_uuid,
            limit,
            offset,
            order_by,
        )
        .await?;

        let mut shipping_rates = Vec::with_capacity(found_shipping_rates.len());

        for shipping_rate in found_shipping_rates {
            shipping_rates.push(self.to_response(shipping_rate)?);
        }

        Ok(Response::new(ListShippingRatesResponse {
            shipping_rates,
            pagination: Some(get_pagination_response(
                limit,
                offset,
                total_elements,
            )),
        }))
    }

    async fn remove_shipping_rate_from_offer(
        &self,
        request: Request<RemoveShippingRateFromOfferRequest>,
    ) -> Result<Response<RemoveShippingRateFromOfferResponse>, Status> {
        let user_id = get_user_id(request.metadata(), &self.verifier).await?;

        let RemoveShippingRateFromOfferRequest {
            offer_id,
            shipping_rate_id,
        } = request.into_inner();

        let offer_uuid = parse_uuid(&offer_id, "offer_id")?;

        let shipping_rate_uuid =
            parse_uuid(&shipping_rate_id, "shipping_rate_id")?;

        ShippingRate::delete(
            &self.pool,
            &shipping_rate_uuid,
            &offer_uuid,
            &user_id,
        )
        .await?;

        Ok(Response::new(RemoveShippingRateFromOfferResponse {}))
    }
}
