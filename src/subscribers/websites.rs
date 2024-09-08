use deadpool_postgres::Pool;
use futures::StreamExt;
use prost::Message;

use crate::api::sited_io::websites::v1::WebsiteResponse;
use crate::model::Shop;

pub struct WebsitesSubscriber {
    nats_client: async_nats::Client,
    pool: Pool,
    allowed_min_platform_fee_percent: u32,
    allowed_min_minimum_platform_fee_cent: u32,
}

impl WebsitesSubscriber {
    pub fn new(
        nats_client: async_nats::Client,
        pool: Pool,
        allowed_min_platform_fee_percent: u32,
        allowed_min_minimum_platform_fee_cent: u32,
    ) -> Self {
        Self {
            nats_client,
            pool,
            allowed_min_platform_fee_percent,
            allowed_min_minimum_platform_fee_cent,
        }
    }

    pub async fn subscribe(&self) {
        let mut subscriber = self
            .nats_client
            .queue_subscribe(
                "websites.website.>",
                "websites.website".to_string(),
            )
            .await
            .unwrap();

        while let Some(message) = subscriber.next().await {
            let action: &str =
                message.subject.split('.').last().unwrap_or_default();

            if action == "upsert" || action == "delete" {
                if let Ok(website) = WebsiteResponse::decode(message.payload) {
                    if let Ok(shop) =
                        Shop::get_by_website_id(&self.pool, &website.website_id)
                            .await
                    {
                        if action == "upsert" && shop.is_none() {
                            tracing::log::info!("[WebsitesSubscriber::subscrbe] create shop for website {}", website.website_id);

                            if let Err(err) = Shop::create_internal(
                                &self.pool,
                                &website.user_id,
                                &website.website_id,
                                &website.name,
                                &website.website_id,
                                self.allowed_min_platform_fee_percent,
                                self.allowed_min_minimum_platform_fee_cent,
                            )
                            .await
                            {
                                tracing::log::error!(
                                    "[WebsitesSubscriber::subscrbe] {:?}",
                                    err
                                );
                            };
                        }

                        if action == "delete" && shop.is_some() {
                            tracing::log::info!("[WebsitesSubscriber::subscrbe] delete shop for website {}", website.website_id);

                            if let Err(err) = Shop::delete_for_website_id(
                                &self.pool,
                                &website.website_id,
                            )
                            .await
                            {
                                tracing::log::error!(
                                    "[WebsitesSubscriber::subscrbe] {:?}",
                                    err
                                );
                            }
                        }
                    }
                }
            }
        }
    }
}
