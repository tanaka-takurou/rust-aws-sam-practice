# Rust AWS SAM Practice
Practices of AWS SAM with rust code.


## Dependencies
 - [aws-lambda-rust-runtime](https://github.com/awslabs/aws-lambda-rust-runtime)
 - [aws-sdk-rust](https://github.com/awslabs/aws-sdk-rust)

## Setting

##### Config
 - [samconfig.toml](https://docs.aws.amazon.com/serverless-application-model/latest/developerguide/serverless-sam-cli-config.html)


##### Build
```
cargo lambda build --release --arm64
```

##### Deploy
```
sam deploy --guided
```
