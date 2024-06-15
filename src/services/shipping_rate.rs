use deadpool_postgres::Pool;
use jwtk::jwk::RemoteJwksVerifier;
use tonic::{async_trait, Request, Response, Status};

use crate::api::sited_io::commerce::v1::shipping_rate_service_server::{
    self, ShippingRateServiceServer,
};
use crate::api::sited_io::commerce::v1::{
    Currency, DeleteShippingRateRequest, DeleteShippingRateResponse,
    GetShippingRateRequest, GetShippingRateResponse, PutShippingRateRequest,
    PutShippingRateResponse, ShippingRateResponse,
};
use crate::auth::get_user_id;
use crate::model::ShippingRate;
use crate::parse_uuid;

pub struct ShippingRateService {
    pool: Pool,
    verifier: RemoteJwksVerifier,
}

impl ShippingRateService {
    const COUNTRIES_SEPARATOR: &'static str = ",";

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
            amount: shipping_rate.amount,
            currency,
            all_countries: shipping_rate.all_countries,
            specific_countries: Self::decode_countries(
                shipping_rate.specific_countries,
            ),
        })
    }

    fn encode_countries(country_codes: Vec<i32>) -> Option<String> {
        if country_codes.is_empty() {
            None
        } else {
            Some(
                country_codes
                    .iter()
                    .map(|c| c.to_string())
                    .collect::<Vec<String>>()
                    .join(Self::COUNTRIES_SEPARATOR),
            )
        }
    }

    fn decode_countries(countries: Option<String>) -> Vec<i32> {
        if let Some(countries) = countries {
            countries
                .split(Self::COUNTRIES_SEPARATOR)
                .map(|c| c.parse().unwrap())
                .collect()
        } else {
            Vec::with_capacity(0)
        }
    }
}

#[async_trait]
impl shipping_rate_service_server::ShippingRateService for ShippingRateService {
    async fn put_shipping_rate(
        &self,
        request: Request<PutShippingRateRequest>,
    ) -> Result<Response<PutShippingRateResponse>, Status> {
        let user_id = get_user_id(request.metadata(), &self.verifier).await?;

        let PutShippingRateRequest {
            offer_id,
            amount,
            currency,
            all_countries,
            specific_countries,
        } = request.into_inner();

        let offer_uuid = parse_uuid(&offer_id, "offer_id")?;

        let currency = Currency::from_i32(currency)
            .ok_or(Status::invalid_argument("currency"))?
            .as_str_name();

        let specific_countries = Self::encode_countries(specific_countries);

        ShippingRate::put(
            &self.pool,
            &offer_uuid,
            &user_id,
            amount,
            currency,
            all_countries,
            specific_countries,
        )
        .await?;

        Ok(Response::new(PutShippingRateResponse {}))
    }

    async fn get_shipping_rate(
        &self,
        request: Request<GetShippingRateRequest>,
    ) -> Result<Response<GetShippingRateResponse>, Status> {
        let GetShippingRateRequest { offer_id } = request.into_inner();

        let offer_id = offer_id.ok_or(Status::invalid_argument("offer_id"))?;
        let offer_uuid = parse_uuid(&offer_id, "offer_id")?;

        let found_shipping_rate =
            ShippingRate::get_by_offer_id(&self.pool, &offer_uuid)
                .await?
                .ok_or(Status::not_found(&offer_id))?;

        Ok(Response::new(GetShippingRateResponse {
            shipping_rate: Some(self.to_response(found_shipping_rate)?),
        }))
    }

    async fn delete_shipping_rate(
        &self,
        request: Request<DeleteShippingRateRequest>,
    ) -> Result<Response<DeleteShippingRateResponse>, Status> {
        let user_id = get_user_id(request.metadata(), &self.verifier).await?;

        let DeleteShippingRateRequest { shipping_rate_id } =
            request.into_inner();

        let shipping_rate_uuid =
            parse_uuid(&shipping_rate_id, "shipping_rate_id")?;

        ShippingRate::delete(&self.pool, &shipping_rate_uuid, &user_id).await?;

        Ok(Response::new(DeleteShippingRateResponse {}))
    }
}
