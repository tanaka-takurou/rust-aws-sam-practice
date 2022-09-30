use aws_lambda_events::event::apigw::{ApiGatewayProxyResponse, ApiGatewayProxyRequest, ApiGatewayProxyRequestContext };
use http::{header, HeaderMap};
use lambda_runtime::{service_fn, LambdaEvent, Error};
use serde_json::{json, Value};

async fn function_handler(event: LambdaEvent<Value>) -> Result<ApiGatewayProxyResponse, Error> {
    let request: ApiGatewayProxyRequest = serde_json::from_value(event.payload).expect("Error event.payload");
    let param_str = request.body.unwrap_or("".to_string());
    let param_json: Value = serde_json::from_str(&param_str).unwrap_or(Value::Null);
    match param_json.get("hostname") {
        Some(hostname) => {
            println!("hostname: {}", hostname);
        },
        None => (),
    };
    match param_json.get("pathname") {
        Some(pathname) => {
            println!("pathname: {}", pathname);
        },
        None => (),
    };
    let context: ApiGatewayProxyRequestContext = request.request_context;
    let resp_json = json!({
        "source_ip": context.identity.source_ip.unwrap_or("".to_string()),
        "request_time": context.request_time.unwrap_or("".to_string())
    });
    let mut map = HeaderMap::new();
    map.append(header::CONTENT_LENGTH, resp_json.to_string().parse().unwrap());
    map.append(header::CONTENT_TYPE, "application/json".parse().unwrap());
    map.append(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*".parse().unwrap());
    map.append(header::ACCESS_CONTROL_ALLOW_HEADERS, "Content-Type".parse().unwrap());
    map.append(header::ACCESS_CONTROL_ALLOW_METHODS, "POST".parse().unwrap());
    Ok(
        ApiGatewayProxyResponse {
            status_code: 200,
            body: Some(resp_json.to_string().into()),
            is_base64_encoded: None,
            headers: map,
            multi_value_headers: HeaderMap::new()
        }
    )
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let func = service_fn(function_handler);
    lambda_runtime::run(func).await?;

    Ok(())
}
