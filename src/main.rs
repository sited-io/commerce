use commerce::api::sited_io::commerce::v1::offer_service_server::OfferServiceServer;
use commerce::websites::WebsitesSubscriber;
use http::header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE};
use http::{HeaderName, Method};
use tonic::transport::Server;
use tower_http::cors::{AllowOrigin, CorsLayer};
use tower_http::trace::TraceLayer;

use commerce::api::sited_io::commerce::v1::shop_service_server::ShopServiceServer;
use commerce::db::{init_db_pool, migrate};
use commerce::images::ImageService;
use commerce::logging::{LogOnFailure, LogOnRequest, LogOnResponse};
use commerce::{
    get_env_var, init_jwks_verifier, OfferService, ShippingRateService,
    ShopCustomizationService, ShopDomainService, ShopService,
};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // initialize logging
    tracing_subscriber::fmt::init();

    // get required environment variables
    let host = get_env_var("HOST");

    let jwks_url = get_env_var("JWKS_URL");
    let jwks_host = get_env_var("JWKS_HOST");

    let allowed_min_platform_fee_percent: u32 =
        get_env_var("ALLOWED_MIN_PLATFORM_FEE_PERCENT")
            .parse()
            .unwrap();
    let allowed_min_minimum_platform_fee_cent: u32 =
        get_env_var("ALLOWED_MIN_MINIMUM_PLATFORM_FEE_CENT")
            .parse()
            .unwrap();

    // initialize database connection and migrate
    let db_pool = init_db_pool(
        get_env_var("DB_HOST"),
        get_env_var("DB_PORT").parse().unwrap(),
        get_env_var("DB_USER"),
        get_env_var("DB_PASSWORD"),
        get_env_var("DB_DBNAME"),
        std::env::var("DB_ROOT_CERT").ok(),
    )?;
    migrate(&db_pool).await?;

    // initialize s3 bucket
    let image_service = ImageService::new(
        get_env_var("BUCKET_NAME"),
        get_env_var("BUCKET_ENDPOINT"),
        get_env_var("BUCKET_ACCESS_KEY_ID"),
        get_env_var("BUCKET_SECRET_ACCESS_KEY"),
        get_env_var("BUCKET_URL"),
        get_env_var("IMAGE_MAX_SIZE").parse().unwrap(),
    )
    .await;

    // initialize NATS client
    let nats_client = async_nats::ConnectOptions::new()
        .user_and_password(
            get_env_var("NATS_USER"),
            get_env_var("NATS_PASSWORD"),
        )
        .connect(get_env_var("NATS_HOST"))
        .await?;

    // initialize and run website subscriber
    let website_subscriber = WebsitesSubscriber::new(
        nats_client,
        db_pool.clone(),
        allowed_min_platform_fee_percent,
        allowed_min_minimum_platform_fee_cent,
    );

    // configure gRPC health reporter
    let (mut health_reporter, health_service) =
        tonic_health::server::health_reporter();
    health_reporter
        .set_serving::<ShopServiceServer<ShopService>>()
        .await;
    health_reporter
        .set_serving::<OfferServiceServer<OfferService>>()
        .await;

    // configure gRPC reflection service
    let reflection_service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(
            tonic_health::pb::FILE_DESCRIPTOR_SET,
        )
        .register_encoded_file_descriptor_set(
            commerce::api::sited_io::FILE_DESCRIPTOR_SET,
        )
        .build()
        .unwrap();

    let shop_service = ShopService::build(
        db_pool.clone(),
        init_jwks_verifier(&jwks_host, &jwks_url)?,
        image_service.clone(),
        allowed_min_platform_fee_percent,
        allowed_min_minimum_platform_fee_cent,
    );

    let shop_customization_service = ShopCustomizationService::build(
        db_pool.clone(),
        init_jwks_verifier(&jwks_host, &jwks_url)?,
        image_service.clone(),
    );

    let shop_domain_service = ShopDomainService::build(
        db_pool.clone(),
        init_jwks_verifier(&jwks_host, &jwks_url)?,
    );

    let offer_service = OfferService::build(
        db_pool.clone(),
        init_jwks_verifier(&jwks_host, &jwks_url)?,
        image_service,
    );

    let shipping_rate_service = ShippingRateService::build(
        db_pool,
        init_jwks_verifier(&jwks_host, &jwks_url)?,
    );

    tracing::log::info!("gRPC+web server listening on {}", host);

    let res = tokio::join!(
        tokio::spawn(async move {
            website_subscriber.subscribe().await;
        }),
        Server::builder()
            .layer(
                TraceLayer::new_for_grpc()
                    .on_request(LogOnRequest::default())
                    .on_response(LogOnResponse::default())
                    .on_failure(LogOnFailure::default()),
            )
            .layer(
                CorsLayer::new()
                    .allow_headers([
                        AUTHORIZATION,
                        ACCEPT,
                        CONTENT_TYPE,
                        HeaderName::from_static("grpc-status"),
                        HeaderName::from_static("grpc-message"),
                        HeaderName::from_static("x-grpc-web"),
                        HeaderName::from_static("x-user-agent"),
                    ])
                    .allow_methods([Method::POST])
                    .allow_origin(AllowOrigin::any())
                    .allow_private_network(true),
            )
            .accept_http1(true)
            .add_service(tonic_web::enable(reflection_service))
            .add_service(tonic_web::enable(health_service))
            .add_service(tonic_web::enable(shop_service))
            .add_service(tonic_web::enable(shop_customization_service))
            .add_service(tonic_web::enable(shop_domain_service))
            .add_service(tonic_web::enable(offer_service))
            .add_service(tonic_web::enable(shipping_rate_service))
            .serve(host.parse().unwrap())
    );

    res.0?;
    res.1?;

    Ok(())
}
