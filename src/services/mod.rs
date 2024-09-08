mod offer;
mod shipping_rate;
mod shop;
mod shop_customization;
mod shop_domain;

pub use offer::OfferService;
pub use shipping_rate::ShippingRateService;
pub use shop::ShopService;
pub use shop_customization::ShopCustomizationService;
pub use shop_domain::ShopDomainService;

use tonic::Status;
use uuid::Uuid;

use crate::api::sited_io::types::v1::{PaginationRequest, PaginationResponse};

pub fn uuid_err_to_grpc_status(field: &str) -> Status {
    Status::invalid_argument(format!("field {field} is not a valid UUID v4"))
}

pub fn parse_uuid(uuid_string: &str, field: &str) -> Result<Uuid, Status> {
    uuid_string
        .parse()
        .map_err(|_| uuid_err_to_grpc_status(field))
}

/// Returns limit and offset from PaginationRequest
fn get_limit_offset_from_pagination(
    request: Option<PaginationRequest>,
) -> Result<(u32, u32, PaginationResponse), Status> {
    let mut limit = 10;
    let mut offset = 0;
    let mut pagination = PaginationResponse {
        page: 1,
        size: limit,
        total_elements: 0,
    };

    if let Some(request) = request {
        if request.page < 1 {
            return Err(Status::invalid_argument(
                "pagination.page less than 1",
            ));
        }
        limit = request.size;
        offset = (request.page - 1) * request.size;
        pagination.page = request.page;
        pagination.size = request.size;
    }

    Ok((limit, offset, pagination))
}
