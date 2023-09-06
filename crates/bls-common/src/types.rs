use crate::http::{HttpReqParams, HttpResponse};
use crate::ipfs::{IpfsGetParams, IpfsResponse};
use crate::impl_display_via_json;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "module", content = "params")]
pub enum ModuleCall {
  Http(HttpReqParams),
  Ipfs(IpfsGetParams),
  // TODO: Add more modules here
}

impl ModuleCall {
  // TODO: validate module permissions based on config
  // - use dependency injection or some generic approach to get the config (permissions) - must be isomorphic
  pub fn validate_permissions(&self) -> bool {
    match self {
      ModuleCall::Http(HttpReqParams { url, .. }) => {
        // TODO: validate permissions logic
      },
      ModuleCall::Ipfs(IpfsGetParams { cid }) => {
        // TODO: validate permissions logic?
      },
    }
    true
  }
}

impl_display_via_json!(ModuleCall);

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "module", content = "response")]
pub enum ModuleCallResponse {
  Http(Result<HttpResponse, String>),
  Ipfs(Result<IpfsResponse, String>),
}

impl_display_via_json!(ModuleCallResponse);
