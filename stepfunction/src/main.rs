use lambda_http::{run, service_fn, Error, IntoResponse, Request, Response};
use aws_config::meta::region::RegionProviderChain;
use aws_config::Region;
use aws_sdk_sfn::Client;

/// This is the main body for the function.
/// Write your code inside it.
/// There are some code example in the following URLs:
/// - https://github.com/awslabs/aws-lambda-rust-runtime/tree/main/lambda-http/examples
async fn function_handler(_event: Request) -> Result<impl IntoResponse, Error> {
    // Extract some useful information from the request
    let region_provider = RegionProviderChain::first_try(Region::new("ap-northeast-1"));
    let shared_config = aws_config::from_env().region(region_provider).load().await;
    let client = Client::new(&shared_config);
    let resp = client.list_state_machines().send().await?;
    let state_machines = resp.state_machines();

    // Return something that implements IntoResponse.
    // It will be serialized to the right response event automatically by the runtime
    let resp = Response::builder()
        .status(200)
        .header("content-type", "text/html")
        .body(format!("Hello AWS Lambda HTTP request. Current StepFunction State Machines: {:?}", state_machines))
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
