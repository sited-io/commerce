use tonic::{Request, Status};

pub fn intercept_log(req: Request<()>) -> Result<Request<()>, Status> {
    tracing::log::info!(target: "grpc-web", "{:?}", req);
    Ok(req)
}
