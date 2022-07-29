use aws_config::meta::region::RegionProviderChain;
use aws_sdk_s3::{Region, Client as S3Client};
use aws_sdk_s3::types::ByteStream;
use aws_sdk_s3::model::ObjectCannedAcl;
use bytes::Buf;
use image::{DynamicImage, imageops, ImageOutputFormat};
use image_convert::{ImageResource, ICOConfig, JPGConfig, PNGConfig, WEBPConfig,
    to_ico, to_jpg, to_png, to_webp};
use lambda_http::{run, service_fn, Error, IntoResponse, Request, Response, Body};
use serde::Serialize;
use serde_json::Value;
use std::env;
use std::io::Cursor;

#[derive(Serialize, Debug)]
struct ResponseData {
    key: String,
}

fn convert_to_jpg(image_bytes: Vec<u8>) -> Vec<u8> {
    let mut config = JPGConfig::new();
    config.remain_profile = false;

    let input = ImageResource::from_reader(image_bytes.reader()).unwrap();
    let mut output = ImageResource::Data(Vec::new());
    to_jpg(&mut output, &input, &config).unwrap();

    output.into_vec().unwrap()
}

fn convert_to_png(image_bytes: Vec<u8>) -> Vec<u8> {
    let mut config = PNGConfig::new();
    config.remain_profile = false;

    let input = ImageResource::from_reader(image_bytes.reader()).unwrap();
    let mut output = ImageResource::Data(Vec::new());
    to_png(&mut output, &input, &config).unwrap();

    output.into_vec().unwrap()
}

fn convert_to_webp(image_bytes: Vec<u8>) -> Vec<u8> {
    let mut config = WEBPConfig::new();
    config.remain_profile = false;

    let input = ImageResource::from_reader(image_bytes.reader()).unwrap();
    let mut output = ImageResource::Data(Vec::new());
    to_webp(&mut output, &input, &config).unwrap();

    output.into_vec().unwrap()
}

fn convert_to_ico(image_bytes: Vec<u8>) -> Vec<u8> {
    let mut config = ICOConfig::new();
    config.remain_profile = false;
    config.size = vec![(16u16, 16u16)];

    let input = ImageResource::from_reader(image_bytes.reader()).unwrap();
    let mut output = ImageResource::Data(Vec::new());
    to_ico(&mut output, &input, &config).unwrap();

    output.into_vec().unwrap()
}

fn resize_for_thumbnail(image_bytes: Vec<u8>, width: u32, height: u32) -> Vec<u8> {
    let img = image::load_from_memory(&image_bytes[..]).unwrap();

    let resized = imageops::resize(&img, height, height, imageops::FilterType::Lanczos3);
    let mut base = DynamicImage::new_rgb8(width, height);

    imageops::overlay(&mut base, &resized, ((width - height)/2) as i64, 0);
    let mut result_bytes: Vec<u8> = Vec::new();
    base.write_to(&mut Cursor::new(&mut result_bytes), ImageOutputFormat::Png).unwrap();

    result_bytes
}

fn resize_for_icon(image_bytes: Vec<u8>, diameter: u32) -> Vec<u8> {
    let img = image::load_from_memory(&image_bytes[..]).unwrap();

    let mut resized = imageops::resize(&img, diameter, diameter, imageops::FilterType::Lanczos3);
    let radius = (diameter / 2) as f64;
    for (x, y, pixel) in resized.enumerate_pixels_mut() {
        if (x as f64 - radius).hypot(y as f64 - radius) > radius {
            *pixel = image::Rgba([0, 0, 0, 0]);
        }
    }
    let mut result_bytes: Vec<u8> = Vec::new();
    resized.write_to(&mut Cursor::new(&mut result_bytes), ImageOutputFormat::Png).unwrap();

    result_bytes
}

async fn upload_image(image_bytes: Vec<u8>, key: String) -> Result<(), Error> {
    let env_bucket = env::var("BUCKET_NAME").unwrap();
    let env_region = env::var("REGION").unwrap();
    let region_provider = RegionProviderChain::first_try(Region::new(env_region.clone()));
    let shared_config = aws_config::from_env().region(region_provider).load().await;

    // Upload Image to S3
    let split_key: Vec<&str> = key.split('.').collect();
    let extension =
        match split_key.get(split_key.len() - 1) {
            Some(v) => (*v).trim_end().to_string(),
            None    => "".to_string(),
        }
    ;
    if !image_bytes.is_empty() && !extension.is_empty() {
        let content_type = format!("image/{}", extension);
        let s3_client = S3Client::new(&shared_config);
        s3_client
            .put_object()
            .acl(ObjectCannedAcl::PublicRead)
            .bucket(&env_bucket)
            .key(&key)
            .body(ByteStream::from(image_bytes))
            .content_type(content_type)
            .send()
            .await?;
        println!("https://{}.s3-{}.amazonaws.com/{}", env_bucket.to_string(), env_region.to_string(), key);
    } else {
        println!("base64data or extension is empty.");
    }
    Ok(())
}

async fn get_image(key: String) -> Result<Vec<u8>, Error> {
    let env_bucket = env::var("BUCKET_NAME").unwrap();
    let env_region = env::var("REGION").unwrap();
    let region_provider = RegionProviderChain::first_try(Region::new(env_region.clone()));
    let shared_config = aws_config::from_env().region(region_provider).load().await;
    let s3_client = S3Client::new(&shared_config);
    let resp = s3_client
        .get_object()
        .bucket(&env_bucket)
        .key(&key)
        .send()
        .await?;
    let data = resp.body.collect().await;
    Ok(data.unwrap().into_bytes().to_vec())
}

fn create_new_key(key: String, suffix: String, new_extension: String) -> String {
    if suffix.is_empty() {
        return key;
    }
    let split_key: Vec<&str> = key.split('.').collect();
    let extension =
        match split_key.get(split_key.len() - 1) {
            Some(v) => (*v).trim_end().to_string(),
            None    => "".to_string(),
        }
    ;
    let name_len = key.len() - extension.len() - 1;
    return format!("{}_{}.{}", &key[..name_len], suffix, new_extension).to_string();
}

/// This is the main body for the function.
/// Write your code inside it.
/// There are some code example in the following URLs:
/// - https://github.com/awslabs/aws-lambda-rust-runtime/tree/main/lambda-http/examples
async fn function_handler(event: Request) -> Result<impl IntoResponse, Error> {
    let mut res_json = "{}".to_string();
    let mut status = 500;
    if let Body::Text(param_str) = event.body() {
        let param: Value = serde_json::from_str(param_str).unwrap();
        let key =  param["key"].to_string();
        println!("key: {}", key.clone());
        let image_bytes = get_image(key.clone()).await?;

        match param["action"].to_string().as_str() {
            "convert" => {
                println!("convert");
                match param["type"].to_string().as_str() {
                    "jpg" => {
                        let _ = upload_image(convert_to_jpg(image_bytes),
                                create_new_key(key.clone(), param["action"].to_string(), param["type"].to_string()));
                    },
                    "png" => {
                        let _ = upload_image(convert_to_png(image_bytes),
                                create_new_key(key.clone(), param["action"].to_string(), param["type"].to_string()));
                    },
                    "webp" => {
                        let _ = upload_image(convert_to_webp(image_bytes),
                                create_new_key(key.clone(), param["action"].to_string(), param["type"].to_string()));
                    },
                    "ico" => {
                        let _ = upload_image(convert_to_ico(image_bytes),
                                create_new_key(key.clone(), param["action"].to_string(), param["type"].to_string()));
                    },
                    _ => {
                        println!("Invalid type");
                    },
                };
            },
            "icon" => {
                println!("icon");
                let diameter: u32 = param["icon"]["diameter"].as_u64().unwrap() as u32;
                let _ = upload_image(resize_for_icon(image_bytes, diameter),
                    create_new_key(key.clone(), format!("{}_{}", param["action"],
                                                param["icon"]["diameter"]).to_string(),
                    "png".to_string()));
            },
            "thumbnail" => {
                println!("thumbnail");
                let width: u32 = param["thumbnail"]["width"].as_u64().unwrap() as u32;
                let height: u32 = param["thumbnail"]["height"].as_u64().unwrap() as u32;
                let _ = upload_image(resize_for_thumbnail(image_bytes, width, height),
                        create_new_key(key.clone(), format!("{}_{}_{}", param["action"],
                                            param["thumbnail"]["width"],
                                            param["thumbnail"]["height"]).to_string(),
                        "png".to_string()));
            },
            _ => {
                println!("Invalid action");
            },
        };

        let res = ResponseData {
            key: key.clone(),
        };
        res_json = serde_json::to_string(&res).unwrap();
        status = 200;
    }

    let resp = Response::builder()
        .status(status)
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
