use crate::impl_display;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use s3::{creds::Credentials, Bucket, BucketConfiguration, Region};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct S3Config {
  pub access_key: String,
  pub secret_key: String,
  pub endpoint: String,
  pub region: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct S3Client {
  credentials: Option<Credentials>,
  region: Option<Region>,
}

impl S3Client {
  pub fn set_config(&mut self, config: S3Config) {
    self.credentials = Some(Credentials::new(
      Some(&config.access_key),
      Some(&config.secret_key),
      None,
      None,
      None,
    ).expect("aws s3 credentials"));
    self.region = Some(Region::Custom {
      region: config.region.unwrap_or("us-east-1".to_string()).into(),
      endpoint: config.endpoint,
    });
  }

  fn validate_config(&self) -> Result<(), &'static str> {
    if self.credentials.is_none() {
      return Err("credentials not set");
    }
    if self.region.is_none() {
      return Err("region not set");
    }
    Ok(())
  }

  pub fn list(&self, opts: S3ListOpts) -> Result<Vec<S3ListItemResponse>, &'static str> {
    self.validate_config()?;

    let bucket = Bucket::new(&opts.bucket_name, self.region.as_ref().unwrap().clone(), self.credentials.as_ref().unwrap().clone())
      .map_err(|e| {
        println!("list bucket error: {:?}", e);
        "bucket error"
      })?;
    let list = bucket.list(opts.prefix, opts.delimiter)
      .map_err(|e| {
        println!("list bucket error: {:?}", e);
        "list bucket error"
      })?;
    let results = list
      .into_iter()
      .map(|item| {
        let item_content = item.contents
          .into_iter()
          .map(|c| {
            S3ListItemResponseContent {
              last_modified: c.last_modified,
              e_tag: c.e_tag,
              storage_class: c.storage_class,
              key: c.key,
              size: c.size,
            }
          })
          .collect::<Vec<_>>();
        S3ListItemResponse {
          name: item.name,
          is_truncated: item.is_truncated,
          prefix: item.prefix,
          contents: item_content,
        }
      })
      .collect::<Vec<_>>();
    Ok(results)
  }

  pub fn create(&self, opts: &S3CreateOpts) -> Result<S3Response, &'static str> {
    self.validate_config()?;
 
    let bucket_config = BucketConfiguration::default();
    let create_response = Bucket::create(&opts.bucket_name, self.region.as_ref().unwrap().clone(), self.credentials.as_ref().unwrap().clone(), bucket_config)
      .map_err(|e| {
        println!("create bucket error: {:?}", e);
        "create bucket error"
      })?;
    Ok(S3Response {
      headers: Default::default(),
      status_code: create_response.response_code,
      bytes: create_response.response_text.as_bytes().to_vec(),
    })
  }

  pub fn get(&self, opts: &S3GetOpts) -> Result<S3Response, &'static str> {
    self.validate_config()?;
 
    let bucket = Bucket::new(&opts.bucket_name, self.region.as_ref().unwrap().clone(), self.credentials.as_ref().unwrap().clone())
      .map_err(|e| {
        println!("get bucket error: {:?}", e);
        "bucket error"
      })?;
    let get_response = bucket.get_object(&opts.path)
      .map_err(|e| {
        println!("get object error: {:?}", e);
        "get object error"
      })?;
    Ok(S3Response {
      status_code: get_response.status_code(),
      headers: get_response.headers(),
      bytes: get_response.to_vec(),
    })
  }

  pub fn put(&self, opts: &S3PutOpts) -> Result<S3Response, &'static str> {
    self.validate_config()?;

    let bucket = Bucket::new(&opts.bucket_name, self.region.as_ref().unwrap().clone(), self.credentials.as_ref().unwrap().clone())
      .map_err(|e| {
        println!("put bucket error: {:?}", e);
        "bucket error"
      })?;
    let put_response = bucket.put_object(&opts.path, opts.content.as_slice())
      .map_err(|e| {
        println!("put object error: {:?}", e);
        "put object error"
      })?;
    Ok(S3Response {
      status_code: put_response.status_code(),
      headers: put_response.headers(),
      bytes: put_response.to_vec(),
    })
  }

  pub fn delete(&self, opts: &S3DeleteOpts) -> Result<S3Response, &'static str> {
    self.validate_config()?;
  
    let bucket = Bucket::new(&opts.bucket_name, self.region.as_ref().unwrap().clone(), self.credentials.as_ref().unwrap().clone())
      .map_err(|e| {
        println!("delete bucket error: {:?}", e);
        "bucket error"
      })?;
    let delete_response = bucket.delete_object(&opts.path)
      .map_err(|e| {
        println!("delete object error: {:?}", e);
        "delete object error"
      })?;
    Ok(S3Response {
      status_code: delete_response.status_code(),
      headers: delete_response.headers(),
      bytes: delete_response.to_vec(),
    })
  }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum S3Command {
  S3SetConfig(S3Config),
  S3Create(S3CreateOpts),
  S3List(S3ListOpts),
  S3Get(S3GetOpts),
  S3Put(S3PutOpts),
  S3Delete(S3DeleteOpts),
}
impl_display!(S3Command);

impl S3Command {
  pub fn exec(&self, client: &mut S3Client) -> Result<Vec<u8>, &'static str> {
    let res = match self {
      S3Command::S3SetConfig(opts) => serde_json::to_vec(&client.set_config(opts.clone()))
        .map_err(|_| "failed to serialize response")?,
      S3Command::S3List(opts) => serde_json::to_vec(&client.list(opts.clone())?)
        .map_err(|_| "failed to serialize response")?,
      S3Command::S3Create(opts) => serde_json::to_vec(&client.create(opts)?)
        .map_err(|_| "failed to serialize response")?,
      S3Command::S3Get(opts) => serde_json::to_vec(&client.get(opts)?)
        .map_err(|_| "failed to serialize response")?,
      S3Command::S3Put(opts) => serde_json::to_vec(&client.put(opts)?)
        .map_err(|_| "failed to serialize response")?,
      S3Command::S3Delete(opts) => serde_json::to_vec(&client.delete(opts)?)
        .map_err(|_| "failed to serialize response")?,
    };
    Ok(res)
  }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct S3ListOpts {
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
  pub bucket_name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct S3GetOpts {
  pub bucket_name: String,
  pub path: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct S3PutOpts {
  pub bucket_name: String,
  pub path: String,
  pub content: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct S3DeleteOpts {
  pub bucket_name: String,
  pub path: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct S3Response {
  bytes: Vec<u8>,
  status_code: u16,
  headers: HashMap<String, String>,
}
