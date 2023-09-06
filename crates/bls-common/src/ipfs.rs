
use crate::impl_display_via_json;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IpfsGetParams {
  pub cid: u64,
}

impl_display_via_json!(IpfsGetParams);

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IpfsResponse {
  pub cid: u64,
  pub data: String,
}

impl_display_via_json!(IpfsResponse);
