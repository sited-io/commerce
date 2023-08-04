use tonic::metadata::MetadataMap;

pub fn get_auth_token(metadata: &MetadataMap) -> Option<String> {
    let auth_header = metadata
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|header_value| header_value.split_once(' '))
        .map(|(_, token)| token.to_string());

    auth_header
}
