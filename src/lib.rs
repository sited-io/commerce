pub mod api;
mod auth;
pub mod db;
pub mod images;
pub mod logging;
mod model;
mod publisher;
mod services;
pub mod subscribers;

pub use auth::init_jwks_verifier;
pub use publisher::Publisher;
pub use services::*;
use tonic::Status;

pub fn get_env_var(var: &str) -> String {
    std::env::var(var).unwrap_or_else(|_| {
        panic!("ERROR: Missing environment variable '{var}'")
    })
}

pub fn i64_to_i32(i: i64) -> Result<i32, Status> {
    i.try_into().map_err(|err| {
        tracing::error!("{:?}", err);
        Status::internal("")
    })
}
