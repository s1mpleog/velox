use aws_config::{BehaviorVersion, Region};
use aws_sdk_s3::{Client, config::Builder, primitives::ByteStream};

use crate::{error::VeloxError, utils::Utils};

pub async fn init_r2() -> Result<Client, VeloxError> {
    let endpoint_url =
        Utils::load_env("R2_ENDPOINT").map_err(|e| VeloxError::EnvError(e.to_string()))?;
    let access_key_id =
        Utils::load_env("R2_ACCESS_KEY_ID").map_err(|e| VeloxError::EnvError(e.to_string()))?;
    let secret_access_key =
        Utils::load_env("R2_SECRET_ACCESS_KEY").map_err(|e| VeloxError::EnvError(e.to_string()))?;

    let config = aws_config::defaults(BehaviorVersion::latest()).load().await;

    let region = Region::new("auto");

    let r2_config = Builder::from(&config)
        .credentials_provider(aws_sdk_s3::config::Credentials::new(
            access_key_id,
            secret_access_key,
            None,
            None,
            "R2",
        ))
        .region(region)
        .endpoint_url(endpoint_url)
        .build();

    let client = aws_sdk_s3::Client::from_conf(r2_config);

    Ok(client)
}

pub async fn upload(
    client: &aws_sdk_s3::Client,
    key: &str,
    content_type: &str,
    data: Vec<u8>,
) -> Result<(), VeloxError> {
    let body = ByteStream::from(data);

    client
        .put_object()
        .bucket(Utils::load_env("R2_BUCKET_NAME")?)
        .key(key)
        .body(body)
        .content_type(content_type)
        .send()
        .await
        .map_err(|_| VeloxError::InternalError)?;

    Ok(())
}
