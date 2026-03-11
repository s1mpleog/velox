use aws_config::{BehaviorVersion, Region};
use aws_sdk_s3::{
    Client,
    config::Builder,
    primitives::ByteStream,
    types::{CompletedMultipartUpload, CompletedPart},
};
use bytes::Bytes;
use futures_util::{Stream, StreamExt};

use crate::{error::VeloxError, utils::Utils};
const MIN_PART_SIZE: usize = 5 * 1024 * 1024;

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

pub async fn upload_streaming(
    client: &aws_sdk_s3::Client,
    key: &str,
    content_type: &str,
    stream: impl Stream<Item = Result<Bytes, VeloxError>> + Unpin + Send,
) -> Result<i64, VeloxError> {
    let bucket = Utils::load_env("R2_BUCKET_NAME")?;
    let create = client
        .create_multipart_upload()
        .bucket(&bucket)
        .key(key)
        .content_type(content_type)
        .send()
        .await
        .map_err(|_| VeloxError::InternalError)?;
    let upload_id = create.upload_id().ok_or(VeloxError::InternalError)?;

    match do_upload(client, &bucket, key, upload_id, stream).await {
        Ok(total_bytes) => Ok(total_bytes),
        Err(e) => {
            let _ = client
                .abort_multipart_upload()
                .bucket(&bucket)
                .key(key)
                .upload_id(upload_id)
                .send()
                .await;
            Err(e)
        }
    }
}

async fn do_upload(
    client: &aws_sdk_s3::Client,
    bucket: &str,
    key: &str,
    upload_id: &str,
    mut stream: impl Stream<Item = Result<Bytes, VeloxError>> + Unpin + Send,
) -> Result<i64, VeloxError> {
    let mut parts: Vec<CompletedPart> = Vec::new();
    let mut part_number = 1i32;
    let mut buffer: Vec<u8> = Vec::with_capacity(MIN_PART_SIZE);
    let mut total_bytes: i64 = 0;

    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        total_bytes += chunk.len() as i64;
        buffer.extend_from_slice(&chunk);
        if buffer.len() >= MIN_PART_SIZE {
            let data = buffer.split_off(0);
            let part = upload_part(client, bucket, key, upload_id, part_number, data).await?;
            parts.push(part);
            part_number += 1;
        }
    }

    if !buffer.is_empty() {
        let part = upload_part(client, bucket, key, upload_id, part_number, buffer).await?;
        parts.push(part);
    }

    let completed = CompletedMultipartUpload::builder()
        .set_parts(Some(parts))
        .build();

    client
        .complete_multipart_upload()
        .bucket(bucket)
        .key(key)
        .upload_id(upload_id)
        .multipart_upload(completed)
        .send()
        .await
        .map_err(|_| VeloxError::InternalError)?;

    Ok(total_bytes)
}

async fn upload_part(
    client: &aws_sdk_s3::Client,
    bucket: &str,
    key: &str,
    upload_id: &str,
    part_number: i32,
    data: Vec<u8>,
) -> Result<CompletedPart, VeloxError> {
    let response = client
        .upload_part()
        .bucket(bucket)
        .key(key)
        .upload_id(upload_id)
        .part_number(part_number)
        .body(ByteStream::from(data))
        .send()
        .await
        .map_err(|_| VeloxError::InternalError)?;

    let etag = response
        .e_tag()
        .ok_or(VeloxError::InternalError)?
        .to_string();

    Ok(CompletedPart::builder()
        .part_number(part_number)
        .e_tag(etag)
        .build())
}

pub async fn delete_from_r2(client: &aws_sdk_s3::Client, key: &str) -> Result<(), VeloxError> {
    client
        .delete_object()
        .bucket(Utils::load_env("R2_BUCKET_NAME")?)
        .key(key)
        .send()
        .await
        .map_err(|_| VeloxError::InternalError)?;

    Ok(())
}
