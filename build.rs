use std::io::Result;

fn main() -> Result<()> {
    const PROTOS: &[&str] = &[
        "service-apis/proto/peoplesmarkets/commerce/v1/shop.proto",
        "service-apis/proto/peoplesmarkets/commerce/v1/shop_customization.proto",
        "service-apis/proto/peoplesmarkets/commerce/v1/shop_domain.proto",
        "service-apis/proto/peoplesmarkets/commerce/v1/offer.proto",
        "service-apis/proto/peoplesmarkets/commerce/v1/shipping_rate.proto",
        ];
    const INCLUDES: &[&str] = &["service-apis/proto"];

    tonic_build::configure()
        .out_dir("src/api")
        .protoc_arg("--experimental_allow_proto3_optional")
        .file_descriptor_set_path("src/api/FILE_DESCRIPTOR_SET")
        .build_client(false)
        .build_server(true)
        .compile(PROTOS, INCLUDES)?;

    Ok(())
}
