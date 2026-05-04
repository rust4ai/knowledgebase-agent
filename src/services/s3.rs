use s3::bucket::Bucket;
use s3::creds::Credentials;
use s3::Region;

use crate::config::AppConfig;
use crate::error::{AppError, AppResult};

pub fn create_bucket(config: &AppConfig) -> AppResult<Box<Bucket>> {
    let region = if let Some(ref endpoint) = config.s3_endpoint {
        Region::Custom {
            region: config.s3_region.clone(),
            endpoint: endpoint.clone(),
        }
    } else {
        Region::Custom {
            region: config.s3_region.clone(),
            endpoint: format!(
                "https://{}.digitaloceanspaces.com",
                config.s3_region
            ),
        }
    };

    let credentials = Credentials::new(
        Some(&config.s3_access_key),
        Some(&config.s3_secret_key),
        None,
        None,
        None,
    )
    .map_err(|e| AppError::S3(e.to_string()))?;

    let bucket = Bucket::new(&config.s3_bucket, region, credentials)
        .map_err(|e| AppError::S3(e.to_string()))?
        .with_path_style();

    Ok(bucket)
}

pub async fn upload_bytes(
    bucket: &Bucket,
    key: &str,
    body: &[u8],
    content_type: &str,
) -> AppResult<()> {
    bucket
        .put_object_with_content_type(key, body, content_type)
        .await
        .map_err(|e| AppError::S3(e.to_string()))?;
    Ok(())
}

pub async fn download_bytes(bucket: &Bucket, key: &str) -> AppResult<Vec<u8>> {
    let response = bucket
        .get_object(key)
        .await
        .map_err(|e| AppError::S3(e.to_string()))?;
    Ok(response.to_vec())
}

pub async fn delete_object(bucket: &Bucket, key: &str) -> AppResult<()> {
    bucket
        .delete_object(key)
        .await
        .map_err(|e| AppError::S3(e.to_string()))?;
    Ok(())
}
