use std::io::Result;

fn main() -> Result<()> {
    tonic_build::configure()
        .out_dir("src/api")
        .protoc_arg("--experimental_allow_proto3_optional")
        .file_descriptor_set_path("src/api/FILE_DESCRIPTOR_SET")
        .build_client(false)
        .build_server(true)
        .compile(
            &[
                "service-apis/proto/peoplesmarkets/commerce/v1/market_booth.proto",
                "service-apis/proto/peoplesmarkets/commerce/v1/offer.proto",
            ], 
            &["service-apis/proto"],
        )?;

    Ok(())
}
