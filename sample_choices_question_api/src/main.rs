use aws_lambda_events::event::apigw::{ApiGatewayProxyResponse, ApiGatewayProxyRequest};
use http::{header, HeaderMap};
use lambda_runtime::{service_fn, LambdaEvent, Error};
use rand::prelude::*;
use serde_json::Value;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Deserialize)]
struct RequestData {
    order: Vec<u32>,
}

#[derive(Deserialize, Serialize)]
struct ResponseData {
    index: u32,
    questions: Vec<QuestionsConfig>,
}

#[derive(Deserialize, Serialize)]
struct Config {
    title: Option<String>,
    random: Option<bool>,
    questions: Vec<QuestionsConfig>,
}

#[derive(Clone, Deserialize, Serialize)]
struct QuestionsConfig {
    question: String,
    explanation: String,
    answers: Vec<u32>,
    choices: Vec<String>,
}

const DEFAULT_TOML_STR: &str = r#"title = "Choices question about numbers"

[[questions]]
question = "Select ALL prime numbers."
explanation = "Prime numbers are numbers greater than 1. They only have two factors, 1 and the number itself."
answers = [2, 3, 5, 7]
choices = [
"1",
"2",
"3",
"4",
"5",
"6",
"7",
"8",
"9",
"10"
]

[[questions]]
question = "Select nearest integer to &pi; (pi)."
explanation = "The number &pi; is approximately equal to 3.14159."
answers = [3]
choices = [
"1",
"2",
"3",
"4",
"5",
"6",
"7",
"8",
"9",
"10"
]

[[questions]]
question = "Select integer part of <i>e</i> (Euler's number)."
explanation = "The number <i>e</i> is approximately equal to 2.71828."
answers = [2]
choices = [
"1",
"2",
"3",
"4",
"5",
"6",
"7",
"8",
"9",
"10"
]
"#;

fn vect_difference(v1: &Vec<u32>, v2: &Vec<u32>) -> Vec<u32> {
    let s1: HashSet<u32> = v1.iter().cloned().collect();
    let s2: HashSet<u32> = v2.iter().cloned().collect();
    (&s1 - &s2).iter().cloned().collect()
}

fn get_config() -> Config {
    let config: Config = toml::from_str(DEFAULT_TOML_STR).unwrap_or(
        Config{
            title: Some(String::new()),
            random: Some(false),
            questions: vec![],
        }
    );
    config
}

fn get_next_index(order: Vec<u32>, length: u32) -> u32 {
    let nums: Vec<u32> = (0u32..length).collect();
    let mut diff = vect_difference(&nums, &order);
    let mut rng = rand::thread_rng();
    diff.shuffle(&mut rng);
    diff[0]
}

fn get_masked_data(q: Vec<QuestionsConfig>, next_index: u32) -> Vec<QuestionsConfig> {
    let question_config = QuestionsConfig{
        question:    q[next_index as usize].question.to_string(),
        explanation: String::new(),
        answers:     vec![0; q[next_index as usize].answers.len()],
        choices:     q[next_index as usize].choices.to_vec(),
    };
    vec![question_config]
}

fn put_in_order(q: Vec<QuestionsConfig>, order: Vec<u32>) -> Vec<QuestionsConfig> {
    let mut res: Vec<QuestionsConfig> = Vec::new();
    for i in 0..q.len() {
        res.push(QuestionsConfig{
            question:    q[order[i] as usize].question.to_string(),
            explanation: q[order[i] as usize].explanation.to_string(),
            answers:     q[order[i] as usize].answers.to_vec(),
            choices:     q[order[i] as usize].choices.to_vec(),
        });
    }
    res
}

async fn function_handler(event: LambdaEvent<Value>) -> Result<ApiGatewayProxyResponse, Error> {
    let request: ApiGatewayProxyRequest = serde_json::from_value(event.payload).expect("Error event.payload");
    let param_str = request.body.unwrap_or("".to_string());
    let req: RequestData = serde_json::from_str(&param_str).unwrap_or(RequestData{order: vec![]});

    let config = get_config();
    let data = match req.order.len() < config.questions.len() {
        true => {
            let next_index = get_next_index(req.order.to_vec(), config.questions.len() as u32);
            ResponseData {
                index: next_index,
                questions: get_masked_data(config.questions.to_vec(), next_index),
            }
        },
        _ => {
            ResponseData {
                index: u32::MAX,
                questions: put_in_order(config.questions.to_vec(), req.order.to_vec()),
            }
        },
    };
    let json_string = serde_json::to_string(&data).unwrap_or(String::new());

    let mut map = HeaderMap::new();
    map.append(header::CONTENT_LENGTH, json_string.to_string().parse().unwrap());
    map.append(header::CONTENT_TYPE, "application/json".parse().unwrap());
    map.append(header::ACCESS_CONTROL_ALLOW_ORIGIN, "https://www.example.com".parse().unwrap());
    map.append(header::ACCESS_CONTROL_ALLOW_HEADERS, "Content-Type".parse().unwrap());
    map.append(header::ACCESS_CONTROL_ALLOW_METHODS, "POST".parse().unwrap());
    Ok(
        ApiGatewayProxyResponse {
            status_code: 200,
            body: Some(json_string.to_string().into()),
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
