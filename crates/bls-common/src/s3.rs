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

  pub async fn list(&self, opts: S3ListOpts) -> Result<Vec<u8>, &'static str> {
    // build the request to list objects in the S3 bucket
    let mut request = http::Request::builder()
      .method("GET")
      .uri(&opts.config.endpoint)
      .header("Accept", "application/json") 
      .body(Default::default())
      .unwrap();

    // sign the request
    let _ = self.sign_request(&opts.config, &mut request)?;

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

    // let bucket = Bucket::new(&opts.bucket_name, self.region.as_ref().unwrap().clone(), self.credentials.as_ref().unwrap().clone())
    //   .map_err(|e| {
    //     println!("list bucket error: {:?}", e);
    //     "bucket error"
    //   })?;
    // let list = bucket.list(opts.prefix, opts.delimiter)
    //   .await
    //   .map_err(|e| {
    //     println!("list bucket error: {:?}", e);
    //     "list bucket error"
    //   })?;
    // let results = list
    //   .into_iter()
    //   .map(|item| {
    //     let item_content = item.contents
    //       .into_iter()
    //       .map(|c| {
    //         S3ListItemResponseContent {
    //           last_modified: c.last_modified,
    //           e_tag: c.e_tag,
    //           storage_class: c.storage_class,
    //           key: c.key,
    //           size: c.size,
    //         }
    //       })
    //       .collect::<Vec<_>>();
    //     S3ListItemResponse {
    //       name: item.name,
    //       is_truncated: item.is_truncated,
    //       prefix: item.prefix,
    //       contents: item_content,
    //     }
    //   })
    //   .collect::<Vec<_>>();
    // Ok(results)
  }

  pub async fn create(&mut self, opts: &S3CreateOpts) -> Result<Vec<u8>, &'static str> {
    let mut request = http::Request::builder()
        .method("PUT")
        .uri(format!("{}/{}", &opts.config.endpoint, opts.bucket_name))
        .header("Accept", "application/json")
        .body(Default::default())
        .unwrap();

    // sign the request
    let _ = self.sign_request(&opts.config, &mut request)?;

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

    // let bucket_config = BucketConfiguration::default();
    //   BucketConfiguration {
    //     acl: CannedBucketAcl::Private,
    //     object_lock_enabled: false,
    //     grant_full_control: None,
    //     grant_read: None,
    //     grant_read_acp: None,
    //     grant_write: None,
    //     grant_write_acp: None,
    //     location_constraint: None,
    // };
    // let mut config = config;
    // config.set_region(region.clone());
    // let command = Command::CreateBucket { config };
    // let bucket = Bucket::new(name, region, credentials)?;
    // let request = RequestImpl::new(&bucket, "", command)?;
    // let response_data = request.response_data(false).await?;
    // let response_text = response_data.as_str()?;
    // Ok(CreateBucketResponse {
    //     bucket,
    //     response_text: response_text.to_string(),
    //     response_code: response_data.status_code(),
    // })

    // pub struct Reqwest<'a> {
    //   pub bucket: &'a Bucket,
    //   pub path: &'a str,
    //   pub command: Command<'a>,
    // }
    
    // let create_response = Bucket::create(&opts.bucket_name, self.region.as_ref().unwrap().clone(), self.credentials.as_ref().unwrap().clone(), bucket_config)
    //   .await  
    //   .map_err(|e| {
    //     println!("create bucket error: {:?}", e);
    //     "create bucket error"
    //   })?;
    // Ok(S3Response {
    //   headers: Default::default(),
    //   status_code: create_response.response_code,
    //   bytes: create_response.response_text.as_bytes().to_vec(),
    // })
  }

  pub async fn get(&self, opts: &S3GetOpts) -> Result<Vec<u8>, &'static str> {
    // build the request to list objects in the S3 bucket
    let mut request = http::Request::builder()
        .method("GET")
        .uri(format!("{}/{}?prefix={}", opts.config.endpoint, opts.bucket_name, opts.path))
        .header("Accept", "application/json")
        .body(Default::default())
        .map_err(|_| "failed to build request")?;

    // sign the request
    let _ = self.sign_request(&opts.config, &mut request)?;

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
 
    // let bucket = Bucket::new(&opts.bucket_name, self.region.as_ref().unwrap().clone(), self.credentials.as_ref().unwrap().clone())
    //   .map_err(|e| {
    //     println!("get bucket error: {:?}", e);
    //     "bucket error"
    //   })?;
    // let get_response = bucket.get_object(&opts.path)
    //   .await
    //   .map_err(|e| {
    //     println!("get object error: {:?}", e);
    //     "get object error"
    //   })?;
    // Ok(S3Response {
    //   status_code: get_response.status_code(),
    //   headers: get_response.headers(),
    //   bytes: get_response.to_vec(),
    // })
  }

  pub async fn put(&self, opts: &S3PutOpts) -> Result<Vec<u8>, &'static str> {
    let mut request = http::Request::builder()
        .method("PUT")
        .uri(format!("{}/{}/{}", opts.config.endpoint, opts.bucket_name, opts.path))
        .header("Accept", "application/json")
        .header("Content-Type", "text/plain")
        .header("Content-Length", opts.content.len().to_string())
        .body(opts.content.clone())
        .unwrap();

     // sign the request
     let _ = self.sign_request(&opts.config, &mut request)?;

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

    // let bucket = Bucket::new(&opts.bucket_name, self.region.as_ref().unwrap().clone(), self.credentials.as_ref().unwrap().clone())
    //   .map_err(|e| {
    //     println!("put bucket error: {:?}", e);
    //     "bucket error"
    //   })?;
    // let put_response = bucket.put_object(&opts.path, opts.content.as_slice())
    //   .await
    //   .map_err(|e| {
    //     println!("put object error: {:?}", e);
    //     "put object error"
    //   })?;
    // Ok(S3Response {
    //   status_code: put_response.status_code(),
    //   headers: put_response.headers(),
    //   bytes: put_response.to_vec(),
    // })
  }

  pub async fn delete(&self, opts: &S3DeleteOpts) -> Result<Vec<u8>, &'static str> {
    let mut request = http::Request::builder()
        .method("DELETE")
        .uri(format!("{}/{}/{}", opts.config.endpoint, opts.bucket_name, opts.path))
        .header("Accept", "application/json")
        .body(Default::default())
        .unwrap();

     // sign the request
     let _ = self.sign_request(&opts.config, &mut request)?;

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

    // let bucket = Bucket::new(&opts.bucket_name, self.region.as_ref().unwrap().clone(), self.credentials.as_ref().unwrap().clone())
    //   .map_err(|e| {
    //     println!("delete bucket error: {:?}", e);
    //     "bucket error"
    //   })?;
    // let delete_response = bucket.delete_object(&opts.path)
    //   .await
    //   .map_err(|e| {
    //     println!("delete object error: {:?}", e);
    //     "delete object error"
    //   })?;
    // Ok(S3Response {
    //   status_code: delete_response.status_code(),
    //   headers: delete_response.headers(),
    //   bytes: delete_response.to_vec(),
    // })
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
      S3Command::S3List(opts) => client.list(opts.clone()).await?,
      S3Command::S3Create(opts) => client.create(opts).await?,
      S3Command::S3Get(opts) => client.get(opts).await?,
      S3Command::S3Put(opts) => client.put(opts).await?,
      S3Command::S3Delete(opts) => client.delete(opts).await?,
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct S3GetOpts {
  pub config: S3Config,

  pub bucket_name: String,
  pub path: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct S3PutOpts {
  pub config: S3Config,

  pub bucket_name: String,
  pub path: String,
  pub content: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct S3DeleteOpts {
  pub config: S3Config,

  pub bucket_name: String,
  pub path: String,
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