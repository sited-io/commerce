use std::time::Duration;

use http::header::{ACCEPT, AUTHORIZATION};
use http::{HeaderName, Method};
use jwtk::jwk::RemoteJwksVerifier;
use tonic::transport::Server;
use tower_http::cors::{AllowOrigin, CorsLayer};
use tower_http::trace::TraceLayer;

use commerce::api::peoplesmarkets::commerce::v1::market_booth_service_server::MarketBoothServiceServer;
use commerce::db::{init_db_pool, migrate};
use commerce::logging::{LogOnFailure, LogOnRequest, LogOnResponse};
use commerce::{get_env_var, MarketBoothService, OfferService};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // initialize logging
    tracing_subscriber::fmt::init();

    // get required environment variables
    let host = get_env_var("HOST");
    let jwks_url = get_env_var("JWKS_URL");
    let jwks_host = get_env_var("JWKS_HOST");

    // initialize database connection and migrate
    let db_pool = init_db_pool()?;
    migrate(&db_pool).await?;

    // initialize client for JWT verification against public JWKS
    //   adding host header in order to work in private network
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(
        reqwest::header::HOST,
        reqwest::header::HeaderValue::from_str(&jwks_host)?,
    );
    let client = reqwest::Client::builder()
        .default_headers(headers)
        .build()?;

    // configure gRPC health reporter
    let (mut health_reporter, health_service) =
        tonic_health::server::health_reporter();
    health_reporter
        .set_serving::<MarketBoothServiceServer<MarketBoothService>>()
        .await;

    // configure gRPC reflection service
    let reflection_service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(
            tonic_health::pb::FILE_DESCRIPTOR_SET,
        )
        .register_encoded_file_descriptor_set(
            commerce::api::peoplesmarkets::FILE_DESCRIPTOR_SET,
        )
        .build()
        .unwrap();

    let market_booth_service = MarketBoothService::build(
        db_pool.clone(),
        RemoteJwksVerifier::new(
            jwks_url.clone(),
            Some(client.clone()),
            Duration::from_secs(120),
        ),
    );
    let offer_service = OfferService::build(
        db_pool,
        RemoteJwksVerifier::new(
            jwks_url,
            Some(client),
            Duration::from_secs(120),
        ),
    );

    tracing::log::info!("gRPC-web server listening on {}", host);

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
                    HeaderName::from_static("grpc-status"),
                    HeaderName::from_static("grpc-message"),
                ])
                .allow_methods([Method::POST])
                .allow_origin(AllowOrigin::any())
                .allow_private_network(true),
        )
        .accept_http1(true)
        .add_service(tonic_web::enable(reflection_service))
        .add_service(tonic_web::enable(health_service))
        .add_service(tonic_web::enable(market_booth_service))
        .add_service(tonic_web::enable(offer_service))
        .serve(host.parse().unwrap())
        .await?;

    Ok(())
}
