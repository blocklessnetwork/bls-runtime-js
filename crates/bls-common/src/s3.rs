use crate::impl_display;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[cfg(feature = "use-wasm-bindgen")]
use aws_sigv4::http_request::{sign, SigningSettings, SigningParams, SignableRequest, };

#[cfg(feature = "use-wasm-bindgen")]
use wasm_timer::SystemTime;

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct S3Config {
  pub access_key: String,
  pub secret_key: String,
  pub endpoint: String,
  pub region: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct S3Client;

#[cfg(feature = "use-wasm-bindgen")]
impl S3Client {

  fn sign_request(&self, config: &S3Config, req: &mut http::Request<Vec<u8>>) -> Result<(), &'static str> {
    if config.access_key.is_empty() || config.secret_key.is_empty(){
      return Err("credentials not set");
    }

    // fix code in aws-sigv4 crate to convert to/from `OffsetDateTime`
    let (access_key, secret_key, region) = {
      (&config.secret_key, &config.access_key, &config.region.clone().unwrap_or("us-east-1".to_string()))
    };
    
    let signing_params = SigningParams::builder()
      .access_key(&access_key)
      .secret_key(&secret_key)
      .region(&region)
      .service_name("s3")
      .time(SystemTime::now())
      .settings(SigningSettings::default())
      .build()
      .unwrap();

    let signable_request = SignableRequest::from(&*req);
    let (signing_instructions, _signature) = sign(signable_request, &signing_params).unwrap().into_parts();
    signing_instructions.apply_to_request(req);
    Ok(())
  }

  pub async fn exec(&self, config: &S3Config, request: impl Into<http::Request<Vec<u8>>>) -> Result<Vec<u8>, &'static str> {
    let mut request = request.into();

    // sign the request
    let _ = self.sign_request(&config, &mut request)?;

    // perform the request
    let reqwest_request = reqwest::Request::try_from(request).unwrap();
    let response = reqwest::Client::new()
        .execute(reqwest_request)
        .await
        .map_err(|_| "failed to execute request")?;

    if !response.status().is_success() {
      return Err("failed to get response");
    }

    let got_text = response.text().await.map_err(|_| "failed to get response text")?;
    Ok(got_text.as_bytes().to_vec())
  }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum S3Command {
  S3Create(S3CreateOpts),
  S3List(S3ListOpts),
  S3Get(S3GetOpts),
  S3Put(S3PutOpts),
  S3Delete(S3DeleteOpts),
}
impl_display!(S3Command);

#[cfg(feature = "use-wasm-bindgen")]
impl S3Command {
  pub async fn exec(&self, client: &mut S3Client) -> Result<Vec<u8>, &'static str> {
    let res = match self {
      S3Command::S3List(opts) => client.exec(&opts.config, opts.clone()).await?,
      S3Command::S3Create(opts) => client.exec(&opts.config, opts.clone()).await?,
      S3Command::S3Get(opts) => client.exec(&opts.config, opts.clone()).await?,
      S3Command::S3Put(opts) => client.exec(&opts.config, opts.clone()).await?,
      S3Command::S3Delete(opts) => client.exec(&opts.config, opts.clone()).await?,
    };
    Ok(res)
  }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct S3ListOpts {
  pub config: S3Config,

  pub bucket_name: String,
  pub prefix: String,
  pub delimiter: Option<String>,
}
impl Into<http::Request<Vec<u8>>> for S3ListOpts {
  fn into(self) -> http::Request<Vec<u8>> {
    let request = http::Request::builder()
      .method("GET")
      .uri(&self.config.endpoint)
      .header("Accept", "application/json") 
      .body(Default::default())
      .unwrap();
    request
  }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct S3ListItemResponseContent {
  pub last_modified: String,
  pub e_tag: Option<String>,
  pub storage_class: Option<String>,
  pub key: String,
  pub size: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct S3ListItemResponse {
  pub name: String,
  pub is_truncated: bool,
  pub prefix: Option<String>,
  pub contents: Vec<S3ListItemResponseContent>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct S3CreateOpts {
  pub config: S3Config,

  pub bucket_name: String,
}
impl Into<http::Request<Vec<u8>>> for S3CreateOpts {
  fn into(self) -> http::Request<Vec<u8>> {
    let request = http::Request::builder()
      .method("PUT")
      .uri(format!("{}/{}", &self.config.endpoint, self.bucket_name))
      .header("Accept", "application/json")
      .body(Default::default())
      .unwrap();
    request
  }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct S3GetOpts {
  pub config: S3Config,

  pub bucket_name: String,
  pub path: String,
}
impl Into<http::Request<Vec<u8>>> for S3GetOpts {
  fn into(self) -> http::Request<Vec<u8>> {
    let request = http::Request::builder()
      .method("GET")
      .uri(format!("{}/{}?prefix={}", &self.config.endpoint, self.bucket_name, self.path))
      .header("Accept", "application/json")
      .body(Default::default())
      .unwrap();
    request
  }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct S3PutOpts {
  pub config: S3Config,

  pub bucket_name: String,
  pub path: String,
  pub content: Vec<u8>,
}
impl Into<http::Request<Vec<u8>>> for S3PutOpts {
  fn into(self) -> http::Request<Vec<u8>> {
    let request = http::Request::builder()
      .method("PUT")
      .uri(format!("{}/{}/{}", &self.config.endpoint, self.bucket_name, self.path))
      .header("Accept", "application/json")
      .header("Content-Type", "text/plain")
      .header("Content-Length", self.content.len().to_string())
      .body(self.content)
      .unwrap();
    request
  }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct S3DeleteOpts {
  pub config: S3Config,

  pub bucket_name: String,
  pub path: String,
}
impl Into<http::Request<Vec<u8>>> for S3DeleteOpts {
  fn into(self) -> http::Request<Vec<u8>> {
    let request = http::Request::builder()
      .method("DELETE")
      .uri(format!("{}/{}/{}", &self.config.endpoint, self.bucket_name, self.path))
      .header("Accept", "application/json")
      .body(Default::default())
      .unwrap();
    request
  }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct S3Response {
  bytes: Vec<u8>,
  status_code: u16,
  headers: HashMap<String, String>,
}

#[cfg(test)]
mod test {
  use super::*;

  fn sign_request(req: &mut http::Request<&str>) {
    let region = "us-east-1";
    let aws_access_key = "test";
    let aws_secret_key = "test";
    
    let signing_params = SigningParams::builder()
      .access_key(aws_access_key)
      .secret_key(aws_secret_key)
      .region(region)
      .service_name("s3")
      .time(std::time::SystemTime::now())
      .settings(SigningSettings::default())
      .build()
      .unwrap();

    let signable_request = SignableRequest::from(&*req);
    let (signing_instructions, _signature) = sign(signable_request, &signing_params).unwrap().into_parts();
    signing_instructions.apply_to_request(req);
  }

  #[tokio::test]
  async fn test_s3_list_buckets_localstack() {
    let endpoint_url = "http://localhost:4566";
    let mut request = http::Request::builder()
      .method("GET")
      .uri(endpoint_url)
      .header("Accept", "application/json") 
      .body("")
      .unwrap();

    sign_request(&mut request);

    // perform the request
    let reqwest_request = reqwest::Request::try_from(request).unwrap();
    let response = reqwest::Client::new()
      .execute(reqwest_request)
      .await
      .unwrap();

    assert!(response.status().is_success());

    let want_text= r#"<?xml version='1.0' encoding='utf-8'?>
<ListAllMyBucketsResult><Owner><DisplayName>webfile</DisplayName><ID>75aa57f09aa0c8caeab4f8c24e99d10f8e7faeebf76c078efc7c6caea54ba06a</ID></Owner><Buckets /></ListAllMyBucketsResult>
"#;

    let got_text = response.text().await.unwrap();
    assert_eq!(want_text, got_text);
  }

  #[tokio::test]
  async fn test_s3_create_bucket_localstack() {
    let endpoint_url = "http://localhost:4566";
    let bucket_name = "my-new-bucket";

    // Build the request to create a new S3 bucket
    let mut request = http::Request::builder()
        .method("PUT")
        .uri(format!("{}/{}", endpoint_url, bucket_name))
        .header("Accept", "application/json")
        .body("")
        .unwrap();

    // Sign the request
    sign_request(&mut request);

    // Perform the request
    let reqwest_request = reqwest::Request::try_from(request).unwrap();
    let response = reqwest::Client::new()
        .execute(reqwest_request)
        .await
        .unwrap();

    assert!(response.status().is_success());
  }

  #[tokio::test]
  async fn test_s3_list_objects_localstack() {
    let endpoint_url = "http://localhost:4566";
    let bucket_name = "my-new-bucket";
    let path = "some/path"; // optional path/prefix

    // Build the request to list objects in the S3 bucket
    let mut request = http::Request::builder()
        .method("GET")
        .uri(format!("{}/{}?prefix={}", endpoint_url, bucket_name, path))
        .header("Accept", "application/json")
        .body("")
        .unwrap();

    // Sign the request
    sign_request(&mut request);

    // Perform the request
    let reqwest_request = reqwest::Request::try_from(request).unwrap();
    let response = reqwest::Client::new()
        .execute(reqwest_request)
        .await
        .unwrap();

    // Assert that the request is successful
    assert!(response.status().is_success());

    let got_text = response.text().await.unwrap();
    assert_eq!("", got_text);
  }

  #[tokio::test]
  async fn test_s3_put_object_localstack() {
    let endpoint_url = "http://localhost:4566";
    let bucket_name = "my-new-bucket";
    let object_path = "some/path/to/new-object.txt";
    let content = "This is the content to write.";

    // Build the request to put an object into the S3 bucket
    let mut request = http::Request::builder()
        .method("PUT")
        .uri(format!("{}/{}/{}", endpoint_url, bucket_name, object_path))
        .header("Accept", "application/json")
        .header("Content-Type", "text/plain")
        .header("Content-Length", content.len().to_string())
        .body(content)
        .unwrap();

    // Sign the request
    sign_request(&mut request);

    // Perform the request
    let reqwest_request = reqwest::Request::try_from(request).unwrap();
    let response = reqwest::Client::new()
        .execute(reqwest_request)
        .await
        .unwrap();

    // Assert that the request is successful
    assert!(response.status().is_success());
  }

  #[tokio::test]
  async fn test_s3_get_object_localstack() {
    let endpoint_url = "http://localhost:4566";
    let bucket_name = "my-new-bucket";
    let object_path = "some/path/to/new-object.txt";

    let mut request = http::Request::builder()
        .method("GET")
        .uri(format!("{}/{}/{}", endpoint_url, bucket_name, object_path))
        .header("Accept", "application/json")
        .body("")
        .unwrap();

    sign_request(&mut request);

    let reqwest_request = reqwest::Request::try_from(request).unwrap();
    let response = reqwest::Client::new()
        .execute(reqwest_request)
        .await
        .unwrap();

    assert!(response.status().is_success());

    let got_content = response.text().await.unwrap();
    assert_eq!(got_content, "This is the content to write.");
  }

  #[tokio::test]
  async fn test_s3_delete_object_localstack() {
    let endpoint_url = "http://localhost:4566";
    let bucket_name = "my-new-bucket";
    let object_path = "some/path/to/new-object.txt";

    // Build the request to delete an object from the S3 bucket
    let mut request = http::Request::builder()
        .method("DELETE")
        .uri(format!("{}/{}/{}", endpoint_url, bucket_name, object_path))
        .header("Accept", "application/json")
        .body("")
        .unwrap();

    // Sign the request
    sign_request(&mut request);

    // Perform the request
    let reqwest_request = reqwest::Request::try_from(request).unwrap();
    let response = reqwest::Client::new()
        .execute(reqwest_request)
        .await
        .unwrap();

    // Assert that the request is successful
    assert!(response.status().is_success());
  }
}