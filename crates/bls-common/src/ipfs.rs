
use crate::impl_display;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IpfsGetParams {
  pub cid: u64,
}

impl_display!(IpfsGetParams);

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IpfsResponse {
  pub cid: u64,
  pub data: String,
}

impl_display!(IpfsResponse);
