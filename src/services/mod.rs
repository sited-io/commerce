mod market_booth;
mod offer;

pub use market_booth::MarketBoothService;
pub use offer::OfferService;

use tonic::Status;
use uuid::Uuid;

use crate::api::peoplesmarkets::pagination::v1::Pagination;

pub fn uuid_err_to_grpc_status(field: &str) -> Status {
    Status::invalid_argument(format!("field {field} is not a valid UUID v4"))
}

pub fn parse_uuid(uuid_string: &str, field: &str) -> Result<Uuid, Status> {
    uuid_string
        .parse()
        .map_err(|_| uuid_err_to_grpc_status(field))
}

/**
 * Returns limit and offset for requested Pagination or defaults.
 */
fn paginate(request: Option<Pagination>) -> (u64, u64, Pagination) {
    let mut limit = 10;
    let mut offset = 0;
    let mut pagination = Pagination {
        page: 1,
        size: limit,
    };

    if let Some(request) = request {
        limit = request.size;
        offset = (request.page - 1) * request.size;
        pagination = request;
    }

    (limit, offset, pagination)
}
