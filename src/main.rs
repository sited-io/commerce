use std::time::Duration;

use jwtk::jwk::RemoteJwksVerifier;
use tonic::transport::Server;
use tonic_health::server::HealthReporter;
use tower_http::trace::TraceLayer;

use commerce::api::peoplesmarkets::commerce::v1::market_booth_service_server::MarketBoothServiceServer;
use commerce::db::{init_db_pool, migrate};
use commerce::logging::{LogOnFailure, LogOnRequest, LogOnResponse};
use commerce::{get_env_var, MarketBoothService};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let host = get_env_var("HOST");
    let jwks_url = get_env_var("JWKS_URL");

    let db_pool = init_db_pool()?;

    migrate(&db_pool).await?;

    let jwt_verifier =
        RemoteJwksVerifier::new(jwks_url, None, Duration::from_secs(120));

    let (mut health_reporter, health_service) =
        tonic_health::server::health_reporter();
    health_reporter
        .set_serving::<MarketBoothServiceServer<MarketBoothService>>()
        .await;

    tracing::log::info!("gRPC server listening on {}", host);

    Server::builder()
        .layer(
            TraceLayer::new_for_grpc()
                .on_request(LogOnRequest::default())
                .on_response(LogOnResponse::default())
                .on_failure(LogOnFailure::default()),
        )
        .add_service(health_service)
        .add_service(MarketBoothService::build(db_pool, jwt_verifier))
        .serve(host.parse().unwrap())
        .await?;

    Ok(())
}
