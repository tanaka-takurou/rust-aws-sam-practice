use aws_config::meta::region::RegionProviderChain;
use lambda_http::{run, service_fn, Error, IntoResponse, Request, Response, Body};
use aws_config::Region;
use aws_sdk_s3::Client as S3Client;
use aws_sdk_s3::primitives::ByteStream;
use aws_sdk_s3::types::ObjectCannedAcl;
use aws_sdk_sfn::Client as StepFunctionClient;
use chrono::Utc;
use serde::Serialize;
use serde_json::Value;
use base64::decode;
use std::env;

#[derive(Serialize, Debug)]
pub struct StepFunctionInputParameter {
    key: String,
}

#[derive(Serialize, Debug)]
struct ResponseData {
    message: String,
}

/// This is the main body for the function.
/// Write your code inside it.
/// There are some code example in the following URLs:
/// - https://github.com/awslabs/aws-lambda-rust-runtime/tree/main/lambda-http/examples
async fn function_handler(event: Request) -> Result<impl IntoResponse, Error> {
    let mut res_json = "{}".to_string();
    if let Body::Text(param_str) = event.body() {
        let param: Value = serde_json::from_str(param_str).unwrap();
        let mut filename =  param["filename"].to_string();
        filename.retain(|c| c != '"');
        let mut filedata =  param["filedata"].to_string();
        filedata.retain(|c| c != '"');

        let env_bucket = env::var("BUCKET_NAME").unwrap();
        let env_region = env::var("REGION").unwrap();
        let env_state_machine_arn = env::var("STATE_MACHINE_ARN").unwrap();
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
        let extension =
            match split_filename.get(split_filename.len() - 1) {
                Some(v) => (*v).trim_end().to_string(),
                None    => "".to_string(),
            }
        ;
        if !base64data.is_empty() && !extension.is_empty() {
            let content_type = format!("image/{}", extension);
            let utc_now = Utc::now();
            let name = format!("{}", utc_now.timestamp_millis()).to_string();
            let key = format!("{}/{}.{}", utc_now.timestamp(), name.clone(), extension.clone()).to_string();
            let s3_client = S3Client::new(&shared_config);
            s3_client
                .put_object()
                .acl(ObjectCannedAcl::PublicRead)
                .bucket(&env_bucket)
                .key(&key)
                .body(ByteStream::from(decode(base64data).unwrap()))
                .content_type(content_type)
                .send()
                .await?;
            println!("https://{}.s3-{}.amazonaws.com/{}", env_bucket.to_string(), env_region.to_string(), key);

            let input = StepFunctionInputParameter {
                key: key,
            };
            let input_json = serde_json::to_string(&input).unwrap();
            let sfn_client = StepFunctionClient::new(&shared_config);
            let _ = sfn_client
                .start_execution()
                .state_machine_arn(&env_state_machine_arn)
                .input(&input_json)
                .name(&name)
                .send()
                .await?;

            let res = ResponseData {
                message: name,
            };
            res_json = serde_json::to_string(&res).unwrap();
        } else {
            println!("base64data or extension is empty.");
        }
    }

    let resp = Response::builder()
        .status(200)
        .header("content-type", "application/json")
        .body(res_json)
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
