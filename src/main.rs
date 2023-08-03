use commerce::{get_env_var, logging::intercept_log, ShopsService};
use tonic::{service::interceptor, transport::Server};
use tonic_web::GrpcWebLayer;
use tower_http::cors::CorsLayer;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let host = get_env_var("HOST");

    tracing::log::info!("Web server listening on {}", host);

    Server::builder()
        .accept_http1(true)
        .layer(CorsLayer::very_permissive())
        .layer(GrpcWebLayer::new())
        .layer(interceptor(intercept_log))
        .add_service(ShopsService::build())
        .serve(host.parse().unwrap())
        .await?;

    Ok(())
}
