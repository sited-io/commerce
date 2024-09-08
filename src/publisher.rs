use prost::Message;

use crate::api::sited_io::commerce::v1::{
    OfferResponse, ShippingRateResponse, ShopResponse,
};

#[derive(Debug, Clone)]
pub struct Publisher {
    client: async_nats::Client,
}

impl Publisher {
    const SHOP_UPSERT_SUBJECT: &'static str = "commerce.shop.upsert";
    const SHOP_DELETE_SUBJECT: &'static str = "commerce.shop.delete";

    const OFFER_UPSERT_SUBJECT: &'static str = "commerce.offer.upsert";
    const OFFER_DELETE_SUBJECT: &'static str = "commerce.offer.delete";

    const SHIPPING_RATE_UPSERT_SUBJECT: &'static str =
        "commerce.shipping_rate.upsert";
    const SHIPPING_RATE_DELETE_SUBJECT: &'static str =
        "commerce.shipping_rate.delete";

    pub fn new(client: async_nats::Client) -> Self {
        Self { client }
    }

    pub async fn flush(
        &self,
    ) -> Result<(), async_nats::error::Error<async_nats::client::FlushErrorKind>>
    {
        self.client.flush().await
    }

    pub async fn publish_upsert_shop(&self, shop: &ShopResponse) {
        if let Err(err) = self
            .client
            .publish(Self::SHOP_UPSERT_SUBJECT, shop.encode_to_vec().into())
            .await
        {
            tracing::error!("[Publisher.publish_upsert_shop]: {err}");
        }
    }

    pub async fn publish_delete_shop(&self, shop: &ShopResponse) {
        if let Err(err) = self
            .client
            .publish(Self::SHOP_DELETE_SUBJECT, shop.encode_to_vec().into())
            .await
        {
            tracing::error!("[Publisher.publish_delete_shop]: {err}");
        }
    }

    pub async fn publish_upsert_offer(&self, offer: &OfferResponse) {
        if let Err(err) = self
            .client
            .publish(Self::OFFER_UPSERT_SUBJECT, offer.encode_to_vec().into())
            .await
        {
            tracing::error!("[Publisher.publish_upsert_offer]: {err}");
        }
    }

    pub async fn publish_delete_offer(&self, offer: &OfferResponse) {
        if let Err(err) = self
            .client
            .publish(Self::OFFER_DELETE_SUBJECT, offer.encode_to_vec().into())
            .await
        {
            tracing::error!("[Publisher.publish_delete_offer]: {err}")
        }
    }

    pub async fn publish_upsert_shipping_rate(
        &self,
        shipping_rate: &ShippingRateResponse,
    ) {
        if let Err(err) = self
            .client
            .publish(
                Self::SHIPPING_RATE_UPSERT_SUBJECT,
                shipping_rate.encode_to_vec().into(),
            )
            .await
        {
            tracing::error!("[Publisher.publish_upsert_offer]: {err}");
        }
    }

    pub async fn publish_delete_shipping_rate(
        &self,
        shipping_rate: &ShippingRateResponse,
    ) {
        if let Err(err) = self
            .client
            .publish(
                Self::SHIPPING_RATE_DELETE_SUBJECT,
                shipping_rate.encode_to_vec().into(),
            )
            .await
        {
            tracing::error!("[Publisher.publish_delete_offer]: {err}")
        }
    }
}
