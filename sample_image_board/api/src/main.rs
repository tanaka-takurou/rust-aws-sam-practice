use aws_config::meta::region::RegionProviderChain;
use lambda_http::{run, service_fn, Error, IntoResponse, Request, Response, Body};
use aws_config::Region;
use aws_sdk_dynamodb::types::AttributeValue;
use aws_sdk_dynamodb::Client as DynamodbClient;
use aws_sdk_s3::Client as S3Client;
use aws_sdk_s3::primitives::ByteStream;
use aws_sdk_s3::types::ObjectCannedAcl;
use chrono::Utc;
use serde_json::Value;
use base64::decode;
use std::env;

/// This is the main body for the function.
/// Write your code inside it.
/// There are some code example in the following URLs:
/// - https://github.com/awslabs/aws-lambda-rust-runtime/tree/main/lambda-http/examples
async fn function_handler(event: Request) -> Result<impl IntoResponse, Error> {
    if let Body::Text(param_str) = event.body() {
        let param: Value = serde_json::from_str(param_str).unwrap();
        let mut filename =  param["filename"].to_string();
        filename.retain(|c| c != '"');
        let mut filedata =  param["filedata"].to_string();
        filedata.retain(|c| c != '"');

        let env_table = env::var("IMG_TABLE_NAME").unwrap();
        let env_bucket = env::var("BUCKET_NAME").unwrap();
        let env_region = env::var("REGION").unwrap();
        let region_provider = RegionProviderChain::first_try(Region::new(env_region.clone()));
        let shared_config = aws_config::from_env().region(region_provider).load().await;

        // Upload Image to S3
        let split_filedata: Vec<&str> = filedata.split(',').collect();
        let base64data =
            match split_filedata.get(1) {
                Some(v) => (*v).trim_end().to_string(),
                None    => "".to_string(),
            }
        ;
        let split_filename: Vec<&str> = filename.split('.').collect();
        let content_type =
            match split_filename.get(split_filename.len() - 1) {
                Some(v) => format!("image/{}", (*v).trim_end().to_string()),
                None    => "".to_string(),
            }
        ;
        if base64data.len() > 0 && content_type.len() > 0 {
            let s3_client = S3Client::new(&shared_config);
            s3_client
                .put_object()
                .acl(ObjectCannedAcl::PublicRead)
                .bucket(&env_bucket)
                .key(&filename)
                .body(ByteStream::from(decode(base64data).unwrap()))
                .content_type(content_type)
                .send()
                .await?;
            let image_uri = format!("https://{}.s3-{}.amazonaws.com/{}", env_bucket.to_string(), env_region.to_string(), filename).to_string();

            // Put Image Url to Dynamodb
            let dynamodb_client = DynamodbClient::new(&shared_config);
            let dynamodb_res: Result<Vec<_>, _> = dynamodb_client
                .scan()
                .table_name(env_table.clone())
                .into_paginator()
                .items()
                .send()
                .collect()
                .await;
            let items: Vec<_> = dynamodb_res.unwrap_or(vec![]);
            let image_id = format!("{}", items.len() + 1).to_string();
            let created = format!("{}", Utc::now()).to_string();
            let id_av = AttributeValue::N(image_id);
            let url_av = AttributeValue::S(image_uri);
            let created_av = AttributeValue::S(created);

            let request = dynamodb_client
                .put_item()
                .table_name(env_table.clone())
                .item("img_id", id_av)
                .item("url", url_av)
                .item("created", created_av);
            request.send().await?;
        } else {
            println!("base64data is empty.");
        }
    }

    let resp = Response::builder()
        .status(200)
        .header("content-type", "application/json")
        .body("{}".to_string())
        .map_err(Box::new)?;
    Ok(resp)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        // disabling time is handy because CloudWatch will add the ingestion time.
        .without_time()
        .init();

    run(service_fn(function_handler)).await
}
