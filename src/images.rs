use base64::Engine;
use s3::Bucket;
use tonic::Status;

#[derive(Debug, Clone)]
pub struct ImageService {
    pub bucket: Bucket,
    pub base_url: String,
    pub max_size: usize,
}

impl ImageService {
    pub fn new(
        bucket_name: String,
        account_id: String,
        access_key_id: String,
        secret_access_key: String,
        base_url: String,
        max_size: usize,
    ) -> Self {
        Self {
            bucket: s3::Bucket::new(
                &bucket_name,
                s3::Region::R2 { account_id },
                s3::creds::Credentials {
                    access_key: Some(access_key_id),
                    secret_key: Some(secret_access_key),
                    security_token: None,
                    session_token: None,
                    expiration: None,
                },
            )
            .unwrap(),
            base_url,
            max_size,
        }
    }

    pub fn get_image_url(&self, image_path: &String) -> String {
        format!("{}{}", self.base_url, image_path)
    }
    pub fn get_opt_image_url(
        &self,
        image_path: Option<String>,
    ) -> Option<String> {
        image_path.map(|p| format!("{}{}", self.base_url, p))
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
            || infer::image::is_png(image_data);

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
        let img = image::load_from_memory(image_data).unwrap();
        let encoder = webp::Encoder::from_image(&img).unwrap();
        let img_webp = encoder.encode(100.0).to_owned();

        tracing::log::info!("{:?}", img);

        self.bucket
            .put_object_with_content_type(image_path, &img_webp, "image/webp")
            .await
            .map_err(|err| {
                tracing::log::error!("{err}");
                Status::internal(err.to_string())
            })?;

        Ok(())
    }

    pub async fn remove_image(
        &self,
        image_path: &String,
    ) -> Result<(), Status> {
        self.bucket.delete_object(image_path).await.map_err(|err| {
            tracing::log::error!("{err}");
            Status::internal(err.to_string())
        })?;

        Ok(())
    }
}
