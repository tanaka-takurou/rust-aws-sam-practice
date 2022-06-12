use lambda_http::{run, service_fn, Error, IntoResponse, Request, Response};
use aws_config::meta::region::RegionProviderChain;
use aws_sdk_apigateway::{Client, Region};

/// This is the main body for the function.
/// Write your code inside it.
/// There are some code example in the following URLs:
/// - https://github.com/awslabs/aws-lambda-rust-runtime/tree/main/lambda-http/examples
async fn function_handler(_event: Request) -> Result<impl IntoResponse, Error> {
    // Extract some useful information from the request
    let region_provider = RegionProviderChain::first_try(Region::new("ap-northeast-1"));
    let shared_config = aws_config::from_env().region(region_provider).load().await;
    let client = Client::new(&shared_config);
    let resp = client.get_rest_apis().send().await?;

    let mut api_names:Vec<String>=Vec::new();
    for api in resp.items().unwrap_or_default() {
        api_names.push(api.name().unwrap_or_default().to_string());
    }

    // Return something that implements IntoResponse.
    // It will be serialized to the right response event automatically by the runtime
    let resp = Response::builder()
        .status(200)
        .header("content-type", "text/html")
        .body(format!("Hello AWS Lambda HTTP request. Current API Gateway REST API Names: {:?}", api_names))
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
