pub mod api;
mod auth;
pub mod db;
pub mod logging;
mod model;
mod services;

pub use services::*;

pub fn get_env_var(var: &str) -> String {
    std::env::var(var).unwrap_or_else(|_| {
        panic!("ERROR: Missing environment variable '{var}'")
    })
}
