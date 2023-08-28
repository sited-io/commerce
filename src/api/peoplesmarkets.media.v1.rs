#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MediaUpload {
    #[prost(string, tag = "1")]
    pub name: ::prost::alloc::string::String,
    #[prost(enumeration = "MediaUploadEncoding", tag = "2")]
    pub encoding: i32,
    #[prost(string, tag = "3")]
    pub data: ::prost::alloc::string::String,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum MediaUploadEncoding {
    Unspecified = 0,
    Base64 = 1,
}
impl MediaUploadEncoding {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            MediaUploadEncoding::Unspecified => "MEDIA_UPLOAD_ENCODING_UNSPECIFIED",
            MediaUploadEncoding::Base64 => "MEDIA_UPLOAD_ENCODING_BASE64",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "MEDIA_UPLOAD_ENCODING_UNSPECIFIED" => Some(Self::Unspecified),
            "MEDIA_UPLOAD_ENCODING_BASE64" => Some(Self::Base64),
            _ => None,
        }
    }
}
