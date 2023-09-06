
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::impl_display_via_json;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HttpReqParams {
  pub url: String,
  pub opts: HttpReqOpts,
}

impl HttpReqParams {
  #[cfg(feature = "use_wasm_bindgen")]
  pub async fn request(&self) -> Result<reqwest::Response, &'static str> {
    let mut headers = reqwest::header::HeaderMap::new();
    if let Some(header_map) = &self.opts.headers {
      headers = header_map.try_into().expect("valid headers");
    };

    let mut client_builder = reqwest::ClientBuilder::new();

    if let Some(ct) = self.opts.connect_timeout {
      // client_builder = client_builder.connect_timeout(std::time::Duration::from_secs(ct));
    }
    if let Some(rt) = self.opts.read_timeout {
      // client_builder = client_builder.timeout(std::time::Duration::from_secs(rt));
    }
    let client = client_builder.build().unwrap();
    // TODO: use client.execute - rather than manually building the request
    // TODO: can we simply use the `Request` struct from the execute in the common types?
    let req_builder = match self.opts.method.to_lowercase().as_str() {
      "head" => client.head(&self.url),
      "get" => client.get(&self.url),
      "post" => client.post(&self.url),
      "delete" => client.delete(&self.url),
      "put" => client.put(&self.url),
      "patch" => client.patch(&self.url),
      _ => return Err("request method not supported"),
    };
    let resp = req_builder
      .headers(headers)
      .body(self.opts.body.clone().unwrap_or_default())
      .send()
      .await
      .map_err(|e| {
          // error!("request send error, {}", e);
          "request send error"
      })?;
    Ok(resp)
  }
}

impl_display_via_json!(HttpReqParams);

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HttpReqOpts {
  pub method: String,
  pub headers: Option<HashMap<String, String>>, // This could be a HashMap<String, String> for more structured headers
  pub body: Option<String>,
  #[serde(rename = "connectTimeout")]
  pub connect_timeout: Option<u64>,
  #[serde(rename = "readTimeout")]
  pub read_timeout: Option<u64>,
}

impl_display_via_json!(HttpReqOpts);

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HttpResponse {
  pub status: u16,
  pub headers: HashMap<String, String>,
  pub body: String,
}

impl HttpResponse {
  #[cfg(feature = "use_wasm_bindgen")]
  pub async fn from_reqwest(resp: reqwest::Response) -> Result<Self, String> {
    let mut headers = HashMap::new();
    for (key, value) in resp.headers().iter() {
      headers.insert(key.to_string(), value.to_str().unwrap_or_default().to_string());
    }
    let status = resp.status().as_u16();
    let body = resp.bytes().await.map_err(|_| "response body error")?;
    Ok(HttpResponse {
      status,
      headers,
      body: String::from_utf8(body.to_vec()).map_err(|_| "response body error")?,
    })
  }
}

impl_display_via_json!(HttpResponse);
