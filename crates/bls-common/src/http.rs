
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::impl_display;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Method {
  Get,
  Post,
  Put,
  Delete,
  Head,
  Trace,
}

impl Default for Method {
  fn default() -> Self { Method::Get }
}

impl AsRef<[u8]> for Method {
  fn as_ref(&self) -> &[u8] {
    match self {
      Method::Get => b"get",
      Method::Post => b"post",
      Method::Put => b"put",
      Method::Delete => b"delete",
      Method::Head => b"head",
      Method::Trace => b"trace",
    }
  }
}

// allow to parse from string
impl std::str::FromStr for Method {
  type Err = &'static str;
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s.to_lowercase().as_str() {
      "get" => Ok(Method::Get),
      "post" => Ok(Method::Post),
      "put" => Ok(Method::Put),
      "delete" => Ok(Method::Delete),
      "head" => Ok(Method::Head),
      "trace" => Ok(Method::Trace),
      _ => Err("Invalid method name"),
    }
  }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct HttpRequest {
  pub url: String,
  pub method: Method,
  pub headers: Option<HashMap<String, String>>,
  pub body: Option<String>,
  // pub timeout: Option<u64>, // NOTE: timeout does not work in wasm
}

impl HttpRequest {
  pub fn new(url: &str, method: Method) -> Self {
    Self {
      url: url.into(),
      method,
      ..Default::default()
    }
  }

  pub fn valid_permissions(&self, permissions: &[String]) -> bool {
    // return true if url (to lower) starts with permissions (lower case)
    let url = self.url.to_ascii_lowercase();
    permissions
      .iter()
      .any(|p| url.starts_with(&p.to_lowercase()))
  }

  #[cfg(feature = "use-wasm-bindgen")]
  pub async fn request(&self) -> Result<reqwest::Response, &'static str> {
    let request: reqwest::Request = self.to_owned().try_into()?;
    let resp = reqwest::Client::new().execute(request).await.map_err(|_e| {
      // error!("request send error, {}", _e);
      "request send error"
    })?;
    Ok(resp)
  }
}

#[cfg(feature = "use-wasm-bindgen")]
impl TryInto<reqwest::Request> for HttpRequest {
  type Error = &'static str;

  fn try_into(self) -> Result<reqwest::Request, Self::Error> {
    let url = reqwest::Url::parse(&self.url).map_err(|_| "invalid url")?;
    let method = reqwest::Method::from_bytes(self.method.as_ref())
      .map_err(|_| "invalid method")?;

    let mut request = reqwest::Request::new(method, url);

    if let Some(headers) = &self.headers {
      let header_map: reqwest::header::HeaderMap = headers.try_into().expect("valid headers");
      request.headers_mut().extend(header_map.into_iter());
    };
    if let Some(body) = self.body {
      request.body_mut().replace(body.into());
    }

    // NOTE: timeout does not work in wasm
    // if let Some(timeout) = self.timeout {
    //   request.timeout_mut().replace(core::time::Duration::from_secs(timeout));
    // }

    Ok(request)
  }
}

impl_display!(HttpRequest);

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HttpResponse {
  pub status: u16,
  pub headers: HashMap<String, String>,
  pub body: Vec<u8>,
}

impl HttpResponse {
  // NOTE: cannot use `impl TryFrom<reqwest::Response> for HttpResponse` because reading body is async
  #[cfg(feature = "use-wasm-bindgen")]
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
      body: body.to_vec(),
    })
  }
}

impl_display!(HttpResponse);
