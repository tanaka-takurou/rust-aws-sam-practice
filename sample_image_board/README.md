# Rust AWS SAM Practice
Practices of AWS SAM with rust code.
Sample Image Board Page.


## Dependencies
 - [aws-lambda-rust-runtime](https://github.com/awslabs/aws-lambda-rust-runtime)
 - [aws-sdk-rust](https://github.com/awslabs/aws-sdk-rust)
 - [tera](https://github.com/Keats/tera)

## Setting
##### Code
 - api/src/main.rs
 - front/src/main.rs

##### Config
 - [samconfig.toml](https://docs.aws.amazon.com/serverless-application-model/latest/developerguide/serverless-sam-cli-config.html)


##### Build
```
cd api
cargo lambda build --release --arm64
cd ../front
cargo lambda build --release --arm64
```

##### Deploy
```
sam deploy --guided
```
