mod api;
mod auth;
pub mod db;
pub mod logging;
mod model;
mod services;

pub use services::*;
use tonic::metadata::MetadataMap;

pub fn get_env_var(var: &str) -> String {
    std::env::var(var)
        .expect(&format!("ERROR: Missing environment variable '{var}'"))
}

