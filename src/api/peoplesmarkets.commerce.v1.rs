// @generated
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MarketBoothResponse {
    #[prost(string, tag="1")]
    pub market_booth_id: ::prost::alloc::string::String,
    #[prost(string, tag="2")]
    pub user_id: ::prost::alloc::string::String,
    #[prost(int64, tag="3")]
    pub created_at: i64,
    #[prost(int64, tag="4")]
    pub updated_at: i64,
    #[prost(string, tag="5")]
    pub name: ::prost::alloc::string::String,
    #[prost(string, tag="6")]
    pub description: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreateMarketBoothRequest {
    #[prost(string, tag="1")]
    pub name: ::prost::alloc::string::String,
    #[prost(string, optional, tag="2")]
    pub description: ::core::option::Option<::prost::alloc::string::String>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreateMarketBoothResponse {
    #[prost(message, optional, tag="1")]
    pub market_booth: ::core::option::Option<MarketBoothResponse>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetMarketBoothRequest {
    #[prost(string, tag="1")]
    pub market_booth_id: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetMarketBoothResponse {
    #[prost(message, optional, tag="1")]
    pub market_booth: ::core::option::Option<MarketBoothResponse>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MarketBoothsOrderBy {
    #[prost(enumeration="MarketBoothsOrderByField", tag="1")]
    pub field: i32,
    #[prost(enumeration="super::super::ordering::v1::Direction", tag="2")]
    pub direction: i32,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MarketBoothsFilter {
    #[prost(enumeration="MarketBoothsFilterField", tag="1")]
    pub field: i32,
    #[prost(string, tag="2")]
    pub filter: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ListMarketBoothsRequest {
    #[prost(string, optional, tag="1")]
    pub user_id: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(message, optional, tag="2")]
    pub pagination: ::core::option::Option<super::super::pagination::v1::Pagination>,
    #[prost(message, optional, tag="3")]
    pub order_by: ::core::option::Option<MarketBoothsOrderBy>,
    #[prost(message, optional, tag="4")]
    pub filter: ::core::option::Option<MarketBoothsFilter>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ListMarketBoothsResponse {
    #[prost(message, repeated, tag="1")]
    pub market_booths: ::prost::alloc::vec::Vec<MarketBoothResponse>,
    #[prost(message, optional, tag="2")]
    pub pagination: ::core::option::Option<super::super::pagination::v1::Pagination>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UpdateMarketBoothRequest {
    #[prost(string, tag="1")]
    pub market_booth_id: ::prost::alloc::string::String,
    #[prost(string, optional, tag="2")]
    pub name: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag="3")]
    pub description: ::core::option::Option<::prost::alloc::string::String>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UpdateMarketBoothResponse {
    #[prost(message, optional, tag="1")]
    pub market_booth: ::core::option::Option<MarketBoothResponse>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DeleteMarketBoothRequest {
    #[prost(string, tag="1")]
    pub market_booth_id: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DeleteMarketBoothResponse {
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum MarketBoothsOrderByField {
    Unspecified = 0,
    CreatedAt = 1,
    UpdatedAt = 2,
    Name = 3,
}
impl MarketBoothsOrderByField {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            MarketBoothsOrderByField::Unspecified => "MARKET_BOOTHS_ORDER_BY_FIELD_UNSPECIFIED",
            MarketBoothsOrderByField::CreatedAt => "MARKET_BOOTHS_ORDER_BY_FIELD_CREATED_AT",
            MarketBoothsOrderByField::UpdatedAt => "MARKET_BOOTHS_ORDER_BY_FIELD_UPDATED_AT",
            MarketBoothsOrderByField::Name => "MARKET_BOOTHS_ORDER_BY_FIELD_NAME",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "MARKET_BOOTHS_ORDER_BY_FIELD_UNSPECIFIED" => Some(Self::Unspecified),
            "MARKET_BOOTHS_ORDER_BY_FIELD_CREATED_AT" => Some(Self::CreatedAt),
            "MARKET_BOOTHS_ORDER_BY_FIELD_UPDATED_AT" => Some(Self::UpdatedAt),
            "MARKET_BOOTHS_ORDER_BY_FIELD_NAME" => Some(Self::Name),
            _ => None,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum MarketBoothsFilterField {
    Unspecified = 0,
    Name = 1,
}
impl MarketBoothsFilterField {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            MarketBoothsFilterField::Unspecified => "MARKET_BOOTHS_FILTER_FIELD_UNSPECIFIED",
            MarketBoothsFilterField::Name => "MARKET_BOOTHS_FILTER_FIELD_NAME",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "MARKET_BOOTHS_FILTER_FIELD_UNSPECIFIED" => Some(Self::Unspecified),
            "MARKET_BOOTHS_FILTER_FIELD_NAME" => Some(Self::Name),
            _ => None,
        }
    }
}
include!("peoplesmarkets.commerce.v1.tonic.rs");
// @@protoc_insertion_point(module)