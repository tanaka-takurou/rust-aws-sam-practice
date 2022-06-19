# Rust AWS SAM Practice
Practices of AWS SAM with rust code.
Sample Web Page.


## Dependencies
 - [aws-lambda-rust-runtime](https://github.com/awslabs/aws-lambda-rust-runtime)
 - [tera](https://github.com/Keats/tera)

## Setting
##### Code
 - front/src/main.rs

##### Config
 - [samconfig.toml](https://docs.aws.amazon.com/serverless-application-model/latest/developerguide/serverless-sam-cli-config.html)


##### Build
```
cd front
cargo lambda build --release --arm64
```

##### Deploy
```
sam deploy --guided
```
