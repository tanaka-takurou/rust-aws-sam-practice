use aws_config::meta::region::RegionProviderChain;
use lambda_http::{run, service_fn, Error, IntoResponse, Request, Response, Body};
use aws_config::Region;
use aws_sdk_sfn::{Client, types::ExecutionStatus};
use serde::Serialize;
use serde_json::Value;
use std::env;

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
        let mut exe_name =  param["id"].to_string();
        exe_name.retain(|c| c != '"');

        let env_region = env::var("REGION").unwrap();
        let env_state_machine_arn = env::var("STATE_MACHINE_ARN").unwrap();
        let region_provider = RegionProviderChain::first_try(Region::new(env_region.clone()));
        let shared_config = aws_config::from_env().region(region_provider).load().await;
        let client = Client::new(&shared_config);

        let mut exe_status = "";
        let status_vec = vec!["RUNNING", "SUCCEEDED"];
        for status in status_vec.into_iter() {
            let resp = client
                .list_executions()
                .state_machine_arn(&env_state_machine_arn)
                .status_filter(ExecutionStatus::from(status))
                .send()
                .await?;
            for execution in resp.executions().into_iter() {
                if execution.name().to_string() == exe_name {
                    exe_status = status;
                    break;
                }
            }
        }

        let res = ResponseData {
            message: exe_status.to_string(),
        };
        res_json = serde_json::to_string(&res).unwrap();
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
