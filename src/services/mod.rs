mod market_booth;

pub use market_booth::MarketBoothService;
use tonic::Status;
use uuid::Uuid;

pub fn uuid_err_to_grpc_status(field: &str) -> Status {
    Status::invalid_argument(format!("field {field} is not a valid UUID v4"))
}

pub fn parse_uuid(uuid_string: String, field: &str) -> Result<Uuid, Status> {
    uuid_string
        .parse()
        .map_err(|_| uuid_err_to_grpc_status(field))
}
