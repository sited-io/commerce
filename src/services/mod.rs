mod market_booth;

pub use market_booth::MarketBoothService;
use tonic::Status;

pub fn uuid_err_to_grpc_status(field: &str) -> Status {
    Status::invalid_argument(format!("field {field} is not a valid UUID v4"))
}
