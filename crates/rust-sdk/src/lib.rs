#![feature(noop_waker)]

use std::collections::HashMap;
use std::cell::{Cell, RefCell};
use std::fmt::Debug;
use std::sync::atomic::AtomicU64;
use bls_common::s3::{S3CreateOpts, S3PutOpts, S3DeleteOpts};
use serde::Serialize;
use serde::de::DeserializeOwned;
use futures::channel::oneshot;

use bls_common::{
    http::{Method, HttpRequest, HttpResponse},
    s3::{S3Command, S3Config, S3ListOpts, S3GetOpts},
    ipfs::{IPFSCommand, FilesLsOpts},
};

mod executor;
mod utils;

#[link(wasm_import_module = "blockless")]
extern "C" {
    #[link_name = "host_log"]
    pub fn host_log(ptr: u32, len: u32);
}

#[link(wasm_import_module = "blockless")]
extern "C" {
    #[link_name = "http_call"]
    pub fn http_call(ptr: u32, len: u32, callback_id: u64) -> u32;
}

#[link(wasm_import_module = "blockless")]
extern "C" {
    #[link_name = "s3_call"]
    pub fn s3_call(ptr: u32, len: u32, callback_id: u64) -> u32;
}

#[link(wasm_import_module = "blockless")]
extern "C" {
    #[link_name = "ipfs_call"]
    pub fn ipfs_call(ptr: u32, len: u32, callback_id: u64) -> u32;
}

// /// Allocate memory into the module's linear memory
// /// and return the offset to the start of the block.
// #[no_mangle]
// pub fn alloc(len: usize) -> *mut u8 {
//     // create a new mutable buffer with capacity `len`
//     let mut buf = Vec::with_capacity(len);
//     // take a mutable pointer to the buffer
//     let ptr = buf.as_mut_ptr();
//     // take ownership of the memory block and
//     // ensure that its destructor is not
//     // called when the object goes out of scope
//     // at the end of the function
//     std::mem::forget(buf);
//     // return the pointer so the runtime
//     // can write data at this offset
//     return ptr;
// }

#[no_mangle]
pub unsafe fn alloc(size: usize) -> *mut u8 {
    let align = std::mem::align_of::<usize>();
    let layout = std::alloc::Layout::from_size_align_unchecked(size, align);
    std::alloc::alloc(layout)
}

#[no_mangle]
pub unsafe fn dealloc(ptr: *mut u8, size: usize) {
    let align = std::mem::align_of::<usize>();
    let layout = std::alloc::Layout::from_size_align_unchecked(size, align);
    std::alloc::dealloc(ptr, layout);
}

#[no_mangle]
pub fn http_callback(result_ptr: usize, callback_id: u64) -> *const u8 {
    let serialized = decode_from_ptr(result_ptr);
    let http_call_response: Result<Vec<u8>, String> = serde_json::from_slice(&serialized[..]).unwrap(); // TODO: handle error

    PENDING_CALLS.with(|calls| {
        if let Some(sender) = calls.borrow_mut().remove(&callback_id) {
            sender.send(http_call_response).expect("Failed to send http_callback result");
        }
    });
    executor::EXECUTOR.with(|e| e.borrow_mut().run());
    0 as *const u8
}

#[no_mangle]
pub fn s3_callback(result_ptr: usize, callback_id: u64) -> *const u8 {
    let serialized = decode_from_ptr(result_ptr);
    let s3_call_response: Result<Vec<u8>, String> = serde_json::from_slice(&serialized[..]).unwrap(); // TODO: handle error

    PENDING_CALLS.with(|calls| {
        if let Some(sender) = calls.borrow_mut().remove(&callback_id) {
            sender.send(s3_call_response).expect("Failed to send s3_callback result");
        }
    });
    executor::EXECUTOR.with(|e| e.borrow_mut().run());
    0 as *const u8
}

#[no_mangle]
pub fn ipfs_callback(result_ptr: usize, callback_id: u64) -> *const u8 {
    let serialized = decode_from_ptr(result_ptr);
    let ipfs_call_response: Result<Vec<u8>, String> = serde_json::from_slice(&serialized[..]).unwrap(); // TODO: handle error

    IPFS_CALLBACKS.with(|callbacks| {
        if let Some(func) = callbacks.borrow_mut().remove(&callback_id) {
            func(ipfs_call_response);
        }
    });

    0 as *const u8
}

fn decode_from_ptr(result_ptr: usize) -> Vec<u8> {
    let serialized = unsafe {
         // first 4 bytes at result_ptr represent the length of the result (as u32)
        let result_len = *(result_ptr as *const u32); // directly dereference to u32

        // assuming the host and guest have the same endianness
        let result_len = u32::from_le(result_len);

        // log the result; data starts at `result_ptr + 4` because the first 4 bytes are used to store the length
        let pointer = result_ptr + 4;
        Vec::from_raw_parts(pointer as *mut u8, result_len as usize, result_len as usize)
    };
    return serialized;
}

static NEXT_CALLBACK_ID: AtomicU64 = AtomicU64::new(0);

// global mutable variables (since this is a single-threaded runtime)
thread_local! {
    static PENDING_CALLS: RefCell<HashMap<u64, oneshot::Sender<Result<Vec<u8>, String>>>> = RefCell::new(HashMap::new());
    static IPFS_CALLBACKS: RefCell<HashMap<u64, fn(Result<Vec<u8>, String>)>> = RefCell::new(HashMap::new());
}

pub async fn dispatch_host_call(
    data: impl Serialize,
    host_call_fn: unsafe extern "C" fn(u32, u32, u64) -> u32,
) -> Result<Vec<u8>, &'static str> {
    let (sender, receiver) = oneshot::channel();

    let callback_id = NEXT_CALLBACK_ID.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    PENDING_CALLS.with(|calls| calls.borrow_mut().insert(callback_id, sender));

    let data = serde_json::to_vec(&data).map_err(|_| "Failed to serialize request")?;

    // Call the FFI function.
    let result_ptr = unsafe { host_call_fn(data.as_ptr() as u32, data.len() as u32, callback_id) };

    // If early return value is non-zero, an error must have ocurred in host runtime.
    if result_ptr != 0 {
        Err("Failed to dispatch the call")?;
    }

    let response = receiver.await
        .map_err(|_| "Failed to receive response")?
        .map_err(|_| "Failed to retrieve data")?;
    Ok(response)
}

pub async fn dispatch_http_call(request: HttpRequest) -> Result<HttpResponse, &'static str> {
    let response = dispatch_host_call(request, http_call).await;
    response.map(|response| serde_json::from_slice::<HttpResponse>(&response[..]).map_err(|_| "Failed to deserialize HttpResponse"))?
}

pub async fn dispatch_s3_call(request: S3Command) -> Result<Vec<u8>, &'static str> {
    dispatch_host_call(request, s3_call).await
}

pub fn dispatch_ipfs_call(module_call: IPFSCommand, callback_fn: fn(Result<Vec<u8>, String>)) -> Result<(), &'static str> {
    let data = serde_json::to_vec(&module_call).map_err(|_| "Failed to serialize request")?;
    let ptr = data.as_ptr() as u32;
    let len = data.len() as u32;

    let callback_id = NEXT_CALLBACK_ID.fetch_add(1, std::sync::atomic::Ordering::SeqCst);

    let result_ptr = unsafe { ipfs_call(ptr, len, callback_id) };
    if result_ptr == 0 {
        IPFS_CALLBACKS.with(|callbacks| {
            callbacks.borrow_mut().insert(callback_id, callback_fn);
        });
        return Ok(());
    }

    let error_response = decode_from_ptr(result_ptr as usize);
    let str_error_response = String::from_utf8(error_response).map_err(|_| "Failed to convert error response to string")?;

    callback_fn(Err(str_error_response));
    Ok(())
}

#[no_mangle]
pub fn _start() {
    executor::spawn_local(async {
        let config = S3Config {
            access_key: "test".to_string(),
            secret_key: "test".to_string(),
            endpoint: "http://localhost:4566".to_string(),
            region: None,
        };

        let result = dispatch_s3_call(S3Command::S3Create(S3CreateOpts {
            config: config.clone(),
            bucket_name: "my-new-bucket".to_string(),
        })).await;
        log!("s3 S3Create callback hit!");
        let res_str = String::from_utf8(result.unwrap()).unwrap();
        log!("{:?}", res_str);

        let result = dispatch_s3_call(S3Command::S3Put(S3PutOpts {
            config: config.clone(),
            bucket_name: "my-new-bucket".to_string(),
            path: "some/path/hello.txt".to_string(),
            content: "hello world".as_bytes().to_vec(),
        })).await;
        log!("s3 S3Put callback hit!");
        let res_str = String::from_utf8(result.unwrap()).unwrap();
        log!("{:?}", res_str);

        let result = dispatch_s3_call(S3Command::S3Get(S3GetOpts {
            config: config.clone(),
            bucket_name: "my-new-bucket".to_string(),
            path: "some/path/hello.txt".to_string(),
        })).await;
        log!("s3 S3Get callback hit!");
        let res_str = String::from_utf8(result.unwrap()).unwrap();
        log!("{:?}", res_str);

        let result = dispatch_s3_call(S3Command::S3Delete(S3DeleteOpts {
            config,
            bucket_name: "my-new-bucket".to_string(),
            path: "some/path/hello.txt".to_string(),
        })).await;
        log!("s3 S3Delete callback hit!");
        let res_str = String::from_utf8(result.unwrap()).unwrap();
        log!("{:?}", res_str);
    });
        
    // executor::spawn_local(async {
    //     let sum = add(1, 5).await;
    //     log!("sum: {}", sum);
        
    //     let response1 = dispatch_http_call(HttpRequest::new("https://jsonplaceholder.typicode.com/todos/1", Method::Get)).await;
    //     log!("First http callback hit!");
    //     log!("{:?}", response1);

    //     let response2 = dispatch_http_call(HttpRequest::new("https://jsonplaceholder.typicode.com/todos/2", Method::Get)).await;
    //     log!("Second http callback hit!");
    //     log!("{:?}", response2);

    //     let response3 = dispatch_http_call(HttpRequest::new("https://jsonplaceholder.typicode.com/todos/3", Method::Get)).await;
    //     log!("Third http callback hit!");
    //     log!("{:?}", response3);

    //     let response4 = dispatch_http_call(HttpRequest::new("https://jsonplaceholder.typicode.com/todos/4", Method::Get)).await;
    //     log!("Fourth http callback hit!");
    //     log!("{:?}", response4);

    //     let response5 = dispatch_http_call(HttpRequest::new("https://jsonplaceholder.typicode.com/todos/5", Method::Get)).await;
    //     log!("Fifth http callback hit!");
    //     log!("{:?}", response5);

    //     let response6 = dispatch_http_call(HttpRequest::new("https://jsonplaceholder.typicode.com/todos/6", Method::Get)).await;
    //     log!("Sixth http callback hit!");
    //     log!("{:?}", response6);

    //     let response7 = dispatch_http_call(HttpRequest::new("https://jsonplaceholder.typicode.com/todos/7", Method::Get)).await;
    //     log!("Seventh http callback hit!");
    //     log!("{:?}", response7);
    // });
}


#[cfg(test)]
mod tests {
    use super::*;

    // requires wasm32-wasi target
    #[test]
    fn test_env_vars() {
        // get env vars
        let env_vars = std::env::vars().collect::<Vec<(String, String)>>();
        log!("{:?}", env_vars);
    }

    // requires wasm32-wasi target
    #[test]
    fn test_args() {
        // get passed in args
        let args = std::env::args().collect::<Vec<String>>();
        let args = args.iter().map(|s| s.as_str()).collect::<Vec<&str>>();
        log!("{:?}", args);
    }

    // #[test]
    // fn test_http_call() {
    //     let req = HttpRequest::new("https://jsonplaceholder.typicode.com/todos/1", Method::Get);
    //     dispatch_http_call(req, |response| {
    //         log!("callback hit!");
    //         log!("{:?}", response);
    //     });
    // }

    #[test]
    fn test_ipfs_call() {
        let files_ls = IPFSCommand::FilesLs(FilesLsOpts::default());
        dispatch_ipfs_call(files_ls, |response| {
            log!("ipfs callback hit!");
            let response_str = String::from_utf8(response.unwrap()).unwrap();
            log!("{:?}", response_str);
        });
    }

    #[test]
    fn test_wasm32_wasi_file_write() {
        // // write hello world to a file (my-file.txt)
        // let file = std::fs::File::create("my-file.txt").unwrap();
        // let mut writer = std::io::BufWriter::new(file);
        // writer.write_all(b"hello world").unwrap();
        // writer.flush().unwrap();

        // // read hello world from a file (my-file.txt)
        // let file = std::fs::File::open("my-file.txt").unwrap();
        // let mut reader = std::io::BufReader::new(file);
        // let mut buffer = String::new();
        // reader.read_line(&mut buffer).unwrap();

        // // log hello world to console
        // log!("buffer: {}", buffer);
    }

}
