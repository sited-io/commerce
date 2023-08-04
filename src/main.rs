use std::time::Duration;

use jwtk::jwk::RemoteJwksVerifier;
use tonic::{service::interceptor, transport::Server};
use tonic_web::GrpcWebLayer;
use tower_http::cors::CorsLayer;

use commerce::db::{init_db_pool, migrate};
use commerce::logging::intercept_log;
use commerce::{get_env_var, ShopsService};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let host = get_env_var("HOST");
    let jwks_url = get_env_var("JWKS_URL");

    let db_pool = init_db_pool()?;

    migrate(&db_pool).await?;

    let jwt_verifier =
        RemoteJwksVerifier::new(jwks_url, None, Duration::from_secs(120));

    tracing::log::info!("Web server listening on {}", host);

    Server::builder()
        .accept_http1(true)
        .layer(CorsLayer::very_permissive())
        .layer(GrpcWebLayer::new())
        .layer(interceptor(intercept_log))
        .add_service(ShopsService::build(db_pool, jwt_verifier))
        .serve(host.parse().unwrap())
        .await?;

    Ok(())
}
