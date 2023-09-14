use aws_credential_types::Credentials;
use aws_sdk_s3::config::Region;
use aws_sdk_s3::primitives::ByteStream;
use aws_sdk_s3::Client;
use base64::Engine;
use tonic::Status;

#[derive(Debug, Clone)]
pub struct ImageService {
    client: Client,
    bucket_name: String,
    base_url: String,
    max_size: usize,
}

impl ImageService {
    pub async fn new(
        bucket_name: String,
        bucket_endpoint: String,
        access_key_id: String,
        secret_access_key: String,
        base_url: String,
        max_size: usize,
    ) -> Self {
        let credentials =
            Credentials::from_keys(access_key_id, secret_access_key, None);

        let config = aws_config::from_env()
            .credentials_provider(credentials)
            .region(Region::new("auto"))
            .endpoint_url(bucket_endpoint)
            .load()
            .await;

        let client = Client::new(&config);

        Self {
            client,
            bucket_name,
            base_url,
            max_size,
        }
    }

    pub fn get_image_url(&self, image_path: &String) -> String {
        format!("{}/{}", self.base_url, image_path)
    }

    pub fn get_opt_image_url(
        &self,
        image_path: Option<String>,
    ) -> Option<String> {
        image_path.map(|p| self.get_image_url(&p))
    }

    pub fn decode_base64(image_string: &String) -> Result<Vec<u8>, Status> {
        base64::engine::general_purpose::STANDARD
            .decode(image_string)
            .map_err(|_| Status::invalid_argument("image"))
    }

    pub fn validate_image(&self, image_data: &[u8]) -> Result<(), Status> {
        let image_size_ok = image_data.len() < self.max_size;
        let image_mime_ok = infer::image::is_jpeg(image_data)
            || infer::image::is_jpeg2000(image_data)
            || infer::image::is_png(image_data)
            || infer::image::is_webp(image_data);

        if image_size_ok && image_mime_ok {
            Ok(())
        } else if !image_size_ok {
            Err(Status::resource_exhausted("image"))
        } else {
            Err(Status::invalid_argument("image"))
        }
    }

    pub async fn put_image(
        &self,
        image_path: &String,
        image_data: &[u8],
    ) -> Result<(), Status> {
        let img = image::load_from_memory(image_data).map_err(|err| {
            tracing::log::error!("[ImageService.put_image]: {err}");
            Status::internal("")
        })?;
        let encoder = webp::Encoder::from_image(&img).map_err(|err| {
            tracing::log::error!("[ImageService.put_image]: {err}");
            Status::internal("")
        })?;
        let img_webp = encoder.encode_lossless().to_owned();

        self.client
            .put_object()
            .bucket(&self.bucket_name)
            .key(image_path)
            .content_type("image/webp")
            .body(ByteStream::from(img_webp))
            .send()
            .await
            .map_err(|err| {
                tracing::log::error!("[ImageService.put_image]: {err}");
                Status::internal("")
            })?;

        Ok(())
    }

    pub async fn remove_image(
        &self,
        image_path: &String,
    ) -> Result<(), Status> {
        self.client
            .delete_object()
            .bucket(&self.bucket_name)
            .key(image_path)
            .send()
            .await
            .map_err(|err| {
                tracing::log::error!("[ImageService.remove_image]: {err}");
                Status::internal(err.to_string())
            })?;

        Ok(())
    }
}
