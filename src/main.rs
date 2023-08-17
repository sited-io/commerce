use std::time::Duration;

use jwtk::jwk::RemoteJwksVerifier;
use tonic::transport::Server;
use tower_http::trace::TraceLayer;

use commerce::api::peoplesmarkets::commerce::v1::market_booth_service_server::MarketBoothServiceServer;
use commerce::db::{init_db_pool, migrate};
use commerce::logging::{LogOnFailure, LogOnRequest, LogOnResponse};
use commerce::{get_env_var, MarketBoothService};

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
    let jwt_verifier = RemoteJwksVerifier::new(
        jwks_url,
        Some(client),
        Duration::from_secs(120),
    );

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
            commerce::api::peoplesmarkets::ordering::v1::FILE_DESCRIPTOR_SET,
        )
        .register_encoded_file_descriptor_set(
            commerce::api::peoplesmarkets::pagination::v1::FILE_DESCRIPTOR_SET,
        )
        .register_encoded_file_descriptor_set(
            commerce::api::peoplesmarkets::commerce::v1::FILE_DESCRIPTOR_SET,
        )
        .build()
        .unwrap();

    tracing::log::info!("gRPC server listening on {}", host);

    let market_booth_service = MarketBoothService::build(db_pool, jwt_verifier);

    Server::builder()
        .layer(
            TraceLayer::new_for_grpc()
                .on_request(LogOnRequest::default())
                .on_response(LogOnResponse::default())
                .on_failure(LogOnFailure::default()),
        )
        .add_service(reflection_service)
        .add_service(health_service)
        .add_service(market_booth_service)
        .serve(host.parse().unwrap())
        .await?;

    Ok(())
}
