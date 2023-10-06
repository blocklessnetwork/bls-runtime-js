
use crate::{impl_display, impl_query_string_conversions};
use crate::http::{HttpRequest, HttpResponse, Method};
use std::str::FromStr;
use serde::{Deserialize, Serialize};

/// declare IPFS client behind feature flag - since reqwest is not supported in wasm32-unknown-unknown targets
#[cfg(feature = "use-wasm-bindgen")]
pub mod client {
  use super::*;

  pub struct IPFSClient {
    client: reqwest::Client,
    url: reqwest::Url,
  }

  impl Default for IPFSClient {
    fn default() -> Self {
      IPFSClient {
        client: reqwest::Client::new(),
        url: reqwest::Url::parse("http://127.0.0.1:5001").unwrap(),
      }
    }
  }

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
}

pub enum IPFSCommands {
  FilesChCid(FilesChCidOpts),
  FilesCp(FilesCpOpts),
  FilesLs(FilesLsOpts),
  FilesMkdir(FilesMkdirOpts),
  FilesMv(FilesMvOpts),
  FilesRead(FilesReadOpts),
  FilesRm(FilesRmOpts),
  FilesStat(FilesStatOpts),
  FilesWrite(FilesWriteOpts),
  Version(VersionOpts),
}

#[cfg(feature = "use-wasm-bindgen")]
impl IPFSCommands {
  pub async fn exec(&self, client: &crate::ipfs::client::IPFSClient) -> Result<Vec<u8>, String> {
    match self {
      IPFSCommands::FilesChCid(opts) => client.post(opts).await,
      IPFSCommands::FilesCp(opts) => client.post(opts).await,
      IPFSCommands::FilesLs(opts) => client.post(opts).await,
      IPFSCommands::FilesMkdir(opts) => client.post(opts).await,
      IPFSCommands::FilesMv(opts) => client.post(opts).await,
      IPFSCommands::FilesRead(opts) => client.post(opts).await,
      IPFSCommands::FilesRm(opts) => client.post(opts).await,
      IPFSCommands::FilesStat(opts) => client.post(opts).await,
      IPFSCommands::FilesWrite(opts) => client.post_form(opts, "file", opts.file_data.clone()).await,
      IPFSCommands::Version(opts) => client.post(opts).await,
    }
  }
}

// https://docs.ipfs.tech/reference/kubo/rpc/#api-v0-files-chcid
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FilesChCidOpts {
  pub arg: String,
  #[serde(rename = "cid-version")]
  pub cid_version: Option<u64>,
  pub hash: Option<String>,
}
impl Default for FilesChCidOpts {
  fn default() -> Self {
    FilesChCidOpts{ arg: "/".into(), cid_version: None, hash: None }
  }
}
impl_query_string_conversions!("files/chcid?", FilesChCidOpts);

// https://docs.ipfs.tech/reference/kubo/rpc/#api-v0-files-cp
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FilesCpOpts {
  pub arg: String,
  #[serde(rename = "arg")]
  pub dest: String,
  pub parents: Option<bool>,
}
impl_query_string_conversions!("files/cp?", FilesCpOpts);

// https://docs.ipfs.tech/reference/kubo/rpc/#api-v0-files-ls
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FilesLsOpts {
  pub arg: String,
  pub long: Option<bool>,
  pub u: Option<bool>,
}
impl Default for FilesLsOpts {
  fn default() -> Self {
    FilesLsOpts{ arg: "/".into(), long: None, u: None }
  }
}
impl_query_string_conversions!("files/ls?", FilesLsOpts);

// https://docs.ipfs.tech/reference/kubo/rpc/#api-v0-files-mkdir
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FilesMkdirOpts {
  pub arg: String,
  pub parents: Option<bool>,
  #[serde(rename = "cid-version")]
  pub cid_version: Option<u64>,
  pub hash: Option<String>,
}
impl_query_string_conversions!("files/mkdir?", FilesMkdirOpts);

// https://docs.ipfs.tech/reference/kubo/rpc/#api-v0-files-mv
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FilesMvOpts {
  pub source: String,
  pub dest: String,
}
impl ToString for FilesMvOpts {
  fn to_string(&self) -> String {
    format!("files/mv?arg={}&arg={}", self.source, self.dest)
  }
}
impl FromStr for FilesMvOpts {
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

    Ok(FilesMvOpts {
      source: parts.remove(0).to_string(),
      dest: parts.remove(0).to_string(),
    })
  }
}

// https://docs.ipfs.tech/reference/kubo/rpc/#api-v0-files-read
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FilesReadOpts {
  pub arg: String,
  pub offset: Option<u64>,
  pub count: Option<u64>,
}
impl_query_string_conversions!("files/read?", FilesReadOpts);

// https://docs.ipfs.tech/reference/kubo/rpc/#api-v0-files-rm
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FilesRmOpts {
  pub arg: String,
  pub recursive: Option<bool>,
  pub force: Option<bool>,
}
impl_query_string_conversions!("files/rm?", FilesRmOpts);

// https://docs.ipfs.tech/reference/kubo/rpc/#api-v0-files-stat
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FilesStatOpts {
  pub arg: String,
  pub format: Option<String>,
  pub hash: Option<bool>,
  pub size: Option<bool>,
  #[serde(rename = "with-local")]
  pub with_local: Option<bool>,
}
impl_query_string_conversions!("files/stat?", FilesStatOpts);

// https://docs.ipfs.tech/reference/kubo/rpc/#api-v0-files-write
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FilesWriteOpts {
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
impl_query_string_conversions!("files/write?", FilesWriteOpts);

// https://docs.ipfs.tech/reference/kubo/rpc/#api-v0-version
#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub struct VersionOpts {
  number: Option<bool>,
  commit: Option<bool>,
  repo: Option<bool>,
  all: Option<bool>,
}
impl_query_string_conversions!("version?", VersionOpts);

#[cfg(test)]
mod tests {
  use super::*;
  use crate::ipfs::client::IPFSClient;

  #[test]
  fn test_files_ch_cid_to_query_string() {
    let mut files_ch_cid = FilesChCidOpts::default();
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
    let files_ch_cid = FilesChCidOpts::from_str("files/chcid?arg=%2Ftest&cid-version=1&hash=sha2-256").unwrap();
    assert_eq!(files_ch_cid.arg, "/test");
    assert_eq!(files_ch_cid.cid_version, Some(1));
    assert_eq!(files_ch_cid.hash, Some("sha2-256".into()));
  }

  #[test]
  fn test_files_mv_to_query_string() {
    let mut files_mv = FilesMvOpts {
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
    let files_mv: FilesMvOpts = "files/mv?arg=/test&arg=/test2".parse().unwrap();
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

    let version = VersionOpts::default();
    assert_eq!(version.to_string(), "version?");
    let _ = IPFSCommands::Version(version).exec(&client).await.unwrap();

    let files_create = FilesWriteOpts {
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
    let _ = IPFSCommands::FilesWrite(files_create).exec(&client).await.unwrap();

    let files_stat = FilesStatOpts {
      arg: "/test.txt".into(),
      format: None,
      hash: None,
      size: None,
      with_local: None,
    };
    assert_eq!(files_stat.to_string(), "files/stat?arg=%2Ftest.txt");
    let _ = IPFSCommands::FilesStat(files_stat).exec(&client).await.unwrap();

    let files_ch_cid = FilesChCidOpts {
      arg: "/test.txt".into(),
      cid_version: None,
      hash: None,
    };
    assert_eq!(files_ch_cid.to_string(), "files/chcid?arg=%2Ftest.txt");
    let _ = IPFSCommands::FilesChCid(files_ch_cid).exec(&client).await.unwrap();

    let files_read = FilesReadOpts {
      arg: "/test.txt".into(),
      offset: None,
      count: None,
    };
    assert_eq!(files_read.to_string(), "files/read?arg=%2Ftest.txt");
    let res = IPFSCommands::FilesRead(files_read).exec(&client).await.unwrap();
    let res_str = String::from_utf8(res).unwrap();
    assert_eq!(res_str, "hello world!");
    
    let files_cp = FilesCpOpts {
      arg: "/test.txt".into(),
      dest: "/test2.txt".into(),
      parents: None,
    };
    assert_eq!(files_cp.to_string(), "files/cp?arg=%2Ftest.txt&arg=%2Ftest2.txt");
    let _ = IPFSCommands::FilesCp(files_cp).exec(&client).await.unwrap();

    let files_mkdir = FilesMkdirOpts {
      arg: "/new-dir".into(),
      parents: None,
      cid_version: None,
      hash: None,
    };
    assert_eq!(files_mkdir.to_string(), "files/mkdir?arg=%2Fnew-dir");
    let _ = IPFSCommands::FilesMkdir(files_mkdir).exec(&client).await.unwrap();

    let files_mv = FilesMvOpts {
      source: "/test2.txt".into(),
      dest: "/new-dir/test2.txt".into(),
    };
    assert_eq!(files_mv.to_string(), "files/mv?arg=/test2.txt&arg=/new-dir/test2.txt");
    let _ = IPFSCommands::FilesMv(files_mv).exec(&client).await.unwrap();

    let files_rm = FilesRmOpts {
      arg: "/new-dir".into(),
      recursive: None,
      force: Some(true),
    };
    assert_eq!(files_rm.to_string(), "files/rm?arg=%2Fnew-dir&force=true");
    let _ = IPFSCommands::FilesRm(files_rm).exec(&client).await.unwrap();

    let files_ls = FilesLsOpts::default();
    assert_eq!(files_ls.to_string(), "files/ls?arg=%2F");
    let res = IPFSCommands::FilesLs(files_ls).exec(&client).await.unwrap();
    let res_str = String::from_utf8(res).unwrap();
    assert_eq!(res_str, "{\"Entries\":[{\"Name\":\"test.txt\",\"Type\":0,\"Size\":0,\"Hash\":\"\"}]}\n");
  }
}