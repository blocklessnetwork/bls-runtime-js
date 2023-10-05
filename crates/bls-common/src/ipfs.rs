
use crate::{impl_display, impl_query_string_conversions};
use crate::http::{HttpRequest, HttpResponse, Method};
use std::str::FromStr;
use serde::{Deserialize, Serialize};

#[cfg(feature = "use-wasm-bindgen")]
pub struct IPFSClient {
  client: reqwest::Client,
  url: reqwest::Url,
}

#[cfg(feature = "use-wasm-bindgen")]
impl Default for IPFSClient {
  fn default() -> Self {
    IPFSClient {
      client: reqwest::Client::new(),
      url: reqwest::Url::parse("http://127.0.0.1:5001").unwrap(),
    }
  }
}

#[cfg(feature = "use-wasm-bindgen")]
impl IPFSClient {
  pub fn new(url: reqwest::Url) -> Self {
    IPFSClient {
      client: reqwest::Client::new(),
      url,
    }
  }

  pub fn api_url(&self) -> String {
    format!("{}api/v0", self.url)
  }

  pub async fn post(&self, command: &impl ToString) -> Result<Vec<u8>, String> {
    let url = format!("{}/{}", &self.api_url(), command.to_string());
    let response = self.client
        .post(url)
        .send()
        .await
        .map_err(|e| format!("Error sending request: {:?}", e))?;
    if response.status() != 200 {
      return Err(format!("Error post response: {:?}", response.status()));
    }
    Ok(response.bytes().await.map_err(|_| "response body error")?.to_vec())
  }

  pub async fn post_form(&self, command: &impl ToString, file_field_name: &str, file_data: Vec<u8>) -> Result<Vec<u8>, String> {
    let url = format!("{}/{}", &self.api_url(), command.to_string());

    let mut form = reqwest::multipart::Form::new();

    // add file data
    let part = reqwest::multipart::Part::bytes(file_data).file_name("file");
    form = form.part(file_field_name.to_owned(), part);

    // perform the request
    let response = self.client
      .post(&url)
      .multipart(form)
      .send()
      .await
      .map_err(|e| format!("Error sending request: {:?}", e))?;

    if response.status().as_u16() != 200 {
      return Err(format!("Error post_form response: {:?}", response.status()));
    }

    Ok(response.bytes().await.map_err(|_| "response body error")?.to_vec())
  }
}

#[cfg(feature = "use-wasm-bindgen")]
#[async_trait::async_trait(?Send)]
pub trait IPFSCommand {
  async fn exec(&self, client: &IPFSClient) -> Result<Vec<u8>, String>
    where
      Self: Sized + ToString,
  {
    Ok(client.post(self).await?)
  }
}

// https://docs.ipfs.tech/reference/kubo/rpc/#api-v0-files-chcid
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FilesChCid {
  pub arg: String,
  #[serde(rename = "cid-version")]
  pub cid_version: Option<u64>,
  pub hash: Option<String>,
}
impl Default for FilesChCid {
  fn default() -> Self {
    FilesChCid{ arg: "/".into(), cid_version: None, hash: None }
  }
}
#[cfg(feature = "use-wasm-bindgen")]
impl IPFSCommand for FilesChCid {}
impl_query_string_conversions!("files/chcid?", FilesChCid);

// https://docs.ipfs.tech/reference/kubo/rpc/#api-v0-files-cp
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FilesCp {
  pub arg: String,
  #[serde(rename = "arg")]
  pub dest: String,
  pub parents: Option<bool>,
}
#[cfg(feature = "use-wasm-bindgen")]
impl IPFSCommand for FilesCp {}
impl_query_string_conversions!("files/cp?", FilesCp);

// https://docs.ipfs.tech/reference/kubo/rpc/#api-v0-files-ls
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FilesLs {
  pub arg: String,
  pub long: Option<bool>,
  pub u: Option<bool>,
}
impl Default for FilesLs {
  fn default() -> Self {
    FilesLs{ arg: "/".into(), long: None, u: None }
  }
}
#[cfg(feature = "use-wasm-bindgen")]
impl IPFSCommand for FilesLs {}
impl_query_string_conversions!("files/ls?", FilesLs);

// https://docs.ipfs.tech/reference/kubo/rpc/#api-v0-files-mkdir
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FilesMkdir {
  pub arg: String,
  pub parents: Option<bool>,
  #[serde(rename = "cid-version")]
  pub cid_version: Option<u64>,
  pub hash: Option<String>,
}
#[cfg(feature = "use-wasm-bindgen")]
impl IPFSCommand for FilesMkdir {}
impl_query_string_conversions!("files/mkdir?", FilesMkdir);

// https://docs.ipfs.tech/reference/kubo/rpc/#api-v0-files-mv
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FilesMv {
  pub source: String,
  pub dest: String,
}
#[cfg(feature = "use-wasm-bindgen")]
impl IPFSCommand for FilesMv {}
impl ToString for FilesMv {
  fn to_string(&self) -> String {
    format!("files/mv?arg={}&arg={}", self.source, self.dest)
  }
}
impl FromStr for FilesMv {
  type Err = &'static str;
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let mut parts = s
      .strip_prefix("files/mv?")
      .ok_or("Prefix mismatch")?
      .split('&')
      .map(|p| p.strip_prefix("arg=").ok_or("Invalid format"))
      .collect::<Result<Vec<_>, _>>()?;

    if parts.len() != 2 {
      return Err("Invalid number of arguments");
    }

    Ok(FilesMv {
      source: parts.remove(0).to_string(),
      dest: parts.remove(0).to_string(),
    })
  }
}

// https://docs.ipfs.tech/reference/kubo/rpc/#api-v0-files-read
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FilesRead {
  pub arg: String,
  pub offset: Option<u64>,
  pub count: Option<u64>,
}
#[cfg(feature = "use-wasm-bindgen")]
impl IPFSCommand for FilesRead {}
impl_query_string_conversions!("files/read?", FilesRead);

// https://docs.ipfs.tech/reference/kubo/rpc/#api-v0-files-rm
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FilesRm {
  pub arg: String,
  pub recursive: Option<bool>,
  pub force: Option<bool>,
}
#[cfg(feature = "use-wasm-bindgen")]
impl IPFSCommand for FilesRm {}
impl_query_string_conversions!("files/rm?", FilesRm);

// https://docs.ipfs.tech/reference/kubo/rpc/#api-v0-files-stat
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FilesStat {
  pub arg: String,
  pub format: Option<String>,
  pub hash: Option<bool>,
  pub size: Option<bool>,
  #[serde(rename = "with-local")]
  pub with_local: Option<bool>,
}
#[cfg(feature = "use-wasm-bindgen")]
impl IPFSCommand for FilesStat {}
impl_query_string_conversions!("files/stat?", FilesStat);

// https://docs.ipfs.tech/reference/kubo/rpc/#api-v0-files-write
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FilesWrite {
  pub arg: String,
  pub offset: Option<u64>,
  pub create: Option<bool>,
  pub truncate: Option<bool>,
  pub count: Option<u64>,
  #[serde(rename = "raw-leaves")]
  pub raw_leaves: Option<bool>,
  pub cid_version: Option<u64>,
  pub hash: Option<String>,
  #[serde(skip)]
  pub file_data: Vec<u8>,
}
#[cfg(feature = "use-wasm-bindgen")]
#[async_trait::async_trait(?Send)]
impl IPFSCommand for FilesWrite {
  async fn exec(&self, client: &IPFSClient) -> Result<Vec<u8>, String> where Self: Sized + ToString {
    Ok(client.post_form(self, "file", self.file_data.clone()).await?)
  }
}
impl_query_string_conversions!("files/write?", FilesWrite);

// https://docs.ipfs.tech/reference/kubo/rpc/#api-v0-version
#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub struct Version {
  number: Option<bool>,
  commit: Option<bool>,
  repo: Option<bool>,
  all: Option<bool>,
}
#[cfg(feature = "use-wasm-bindgen")]
impl IPFSCommand for Version {}
impl_query_string_conversions!("version?", Version);

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_files_ch_cid_to_query_string() {
    let mut files_ch_cid = FilesChCid::default();
    assert_eq!(files_ch_cid.to_string(), "files/chcid?arg=%2F");

    files_ch_cid.arg = "/test".into();
    assert_eq!(files_ch_cid.to_string(), "files/chcid?arg=%2Ftest");

    files_ch_cid.cid_version = Some(1);
    assert_eq!(files_ch_cid.to_string(), "files/chcid?arg=%2Ftest&cid-version=1");

    files_ch_cid.hash = Some("sha2-256".into());
    assert_eq!(files_ch_cid.to_string(), "files/chcid?arg=%2Ftest&cid-version=1&hash=sha2-256");
    
    files_ch_cid.cid_version = None;
    assert_eq!(files_ch_cid.to_string(), "files/chcid?arg=%2Ftest&hash=sha2-256");
  }

  #[test]
  fn test_files_ch_cid_from_query_string() {
    let files_ch_cid = FilesChCid::from_str("files/chcid?arg=%2Ftest&cid-version=1&hash=sha2-256").unwrap();
    assert_eq!(files_ch_cid.arg, "/test");
    assert_eq!(files_ch_cid.cid_version, Some(1));
    assert_eq!(files_ch_cid.hash, Some("sha2-256".into()));
  }

  #[test]
  fn test_files_mv_to_query_string() {
    let mut files_mv = FilesMv {
      source: "/".into(),
      dest: "/".into(),
    };
    assert_eq!(files_mv.to_string(), "files/mv?arg=/&arg=/");

    files_mv.source = "/test".into();
    assert_eq!(files_mv.to_string(), "files/mv?arg=/test&arg=/");

    files_mv.dest = "/test2".into();
    assert_eq!(files_mv.to_string(), "files/mv?arg=/test&arg=/test2");
  }

  #[test]
  fn test_files_mv_from_query_string() {
    let files_mv: FilesMv = "files/mv?arg=/test&arg=/test2".parse().unwrap();
    assert_eq!(files_mv.source, "/test");
    assert_eq!(files_mv.dest, "/test2");
  }

  #[tokio::test]
  async fn test_files_commands_local_node() {
    // VERSION
    // curl -X POST "http://127.0.0.1:5001/api/v0/version?"

    // WRITE
    // curl -X POST -F file=@test.txt "http://127.0.0.1:5001/api/v0/files/write?arg=/test.txt&create=true"

    // STAT
    // curl -X POST "http://127.0.0.1:5001/api/v0/files/stat?arg=/test.txt"

    // CHCID
    // curl -X POST "http://127.0.0.1:5001/api/v0/files/chcid?arg=/test.txt"

    // READ
    // curl -X POST "http://127.0.0.1:5001/api/v0/files/read?arg=/test.txt"

    // CP
    // curl -X POST "http://127.0.0.1:5001/api/v0/files/cp?arg=/test.txt&arg=/test2.txt"

    // MKDIR
    // curl -X POST "http://127.0.0.1:5001/api/v0/files/mkdir?arg=/new-dir"

    // MV
    // curl -X POST "http://127.0.0.1:5001/api/v0/files/mv?arg=/test2.txt&arg=/new-dir/test2.txt"

    // RM
    // curl -X POST "http://127.0.0.1:5001/api/v0/files/rm?arg=/new-dir&force=true"

    // LS
    // curl -X POST "http://127.0.0.1:5001/api/v0/files/ls?arg=/"

    let client = IPFSClient::default();

    let version = Version::default();
    assert_eq!(version.to_string(), "version?");
    let _ = version.exec(&client).await.unwrap();

    let files_create = FilesWrite {
      arg: "/test.txt".into(),
      file_data: "hello world!".as_bytes().to_vec(),
      offset: None,
      create: Some(true),
      truncate: None,
      count: None,
      raw_leaves: None,
      cid_version: None,
      hash: None,
    };
    assert_eq!(files_create.to_string(), "files/write?arg=%2Ftest.txt&create=true");
    let _ = files_create.exec(&client).await.unwrap();

    let files_stat = FilesStat {
      arg: "/test.txt".into(),
      format: None,
      hash: None,
      size: None,
      with_local: None,
    };
    assert_eq!(files_stat.to_string(), "files/stat?arg=%2Ftest.txt");
    let _ = files_stat.exec(&client).await.unwrap();

    let files_ch_cid = FilesChCid {
      arg: "/test.txt".into(),
      cid_version: None,
      hash: None,
    };
    assert_eq!(files_ch_cid.to_string(), "files/chcid?arg=%2Ftest.txt");
    let _ = files_ch_cid.exec(&client).await.unwrap();

    let files_read = FilesRead {
      arg: "/test.txt".into(),
      offset: None,
      count: None,
    };
    assert_eq!(files_read.to_string(), "files/read?arg=%2Ftest.txt");
    let res = files_read.exec(&client).await.unwrap();
    let res_str = String::from_utf8(res).unwrap();
    assert_eq!(res_str, "hello world!");
    
    let files_cp = FilesCp {
      arg: "/test.txt".into(),
      dest: "/test2.txt".into(),
      parents: None,
    };
    assert_eq!(files_cp.to_string(), "files/cp?arg=%2Ftest.txt&arg=%2Ftest2.txt");
    let _ = files_cp.exec(&client).await.unwrap();

    let files_mkdir = FilesMkdir {
      arg: "/new-dir".into(),
      parents: None,
      cid_version: None,
      hash: None,
    };
    assert_eq!(files_mkdir.to_string(), "files/mkdir?arg=%2Fnew-dir");
    let _ = files_mkdir.exec(&client).await.unwrap();

    let files_mv = FilesMv {
      source: "/test2.txt".into(),
      dest: "/new-dir/test2.txt".into(),
    };
    assert_eq!(files_mv.to_string(), "files/mv?arg=/test2.txt&arg=/new-dir/test2.txt");
    let _ = files_mv.exec(&client).await.unwrap();

    let files_rm = FilesRm {
      arg: "/new-dir".into(),
      recursive: None,
      force: Some(true),
    };
    assert_eq!(files_rm.to_string(), "files/rm?arg=%2Fnew-dir&force=true");
    let _ = files_rm.exec(&client).await.unwrap();

    let files_ls = FilesLs::default();
    assert_eq!(files_ls.to_string(), "files/ls?arg=%2F");
    let res = files_ls.exec(&client).await.unwrap();
    let res_str = String::from_utf8(res).unwrap();
    assert_eq!(res_str, "{\"Entries\":[{\"Name\":\"test.txt\",\"Type\":0,\"Size\":0,\"Hash\":\"\"}]}\n");
  }
}