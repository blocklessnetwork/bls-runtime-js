use std::collections::HashMap;
use std::cell::{Cell, RefCell};

use bls_common::{
    http::{Method, HttpRequest, HttpResponse},
    ipfs::{IPFSCommand, FilesLsOpts},
};

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
    let http_call_response: Result<HttpResponse, String> = serde_json::from_slice(&serialized[..]).unwrap(); // TODO: handle error

    HTTP_CALLBACKS.with(|callbacks| {
        if let Some(func) = callbacks.borrow_mut().remove(&callback_id) {
            func(http_call_response);
        }
    });

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



// global mutable variables (since this is a single-threaded runtime)
thread_local! {
    static NEXT_CALLBACK_ID: Cell<u64> = Cell::new(0);
    static HTTP_CALLBACKS: RefCell<HashMap<u64, fn(Result<HttpResponse, String>)>> = RefCell::new(HashMap::new());
    static IPFS_CALLBACKS: RefCell<HashMap<u64, fn(Result<Vec<u8>, String>)>> = RefCell::new(HashMap::new());
}

pub fn dispatch_http_call(module_call: HttpRequest, callback_fn: fn(Result<HttpResponse, String>)) -> Result<(), &'static str> {
    let data = serde_json::to_vec(&module_call).map_err(|_| "Failed to serialize request")?;
    let ptr = data.as_ptr() as u32;
    let len = data.len() as u32;

    let callback_id = NEXT_CALLBACK_ID.with(|id| {
        let next_id = id.get() + 1;
        id.set(next_id);
        next_id
    });

    let result_ptr = unsafe { http_call(ptr, len, callback_id) };
    if result_ptr == 0 {
        HTTP_CALLBACKS.with(|callbacks| {
            callbacks.borrow_mut().insert(callback_id, callback_fn);
        });
        return Ok(());
    }

    let error_response = decode_from_ptr(result_ptr as usize);
    let str_error_response = String::from_utf8(error_response).map_err(|_| "Failed to convert error response to string")?;

    callback_fn(Err(str_error_response));
    Ok(())
}

pub fn dispatch_ipfs_call(module_call: IPFSCommand, callback_fn: fn(Result<Vec<u8>, String>)) -> Result<(), &'static str> {
    let data = serde_json::to_vec(&module_call).map_err(|_| "Failed to serialize request")?;
    let ptr = data.as_ptr() as u32;
    let len = data.len() as u32;

    let callback_id = NEXT_CALLBACK_ID.with(|id| {
        let next_id = id.get() + 1;
        id.set(next_id);
        next_id
    });

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
    let req = HttpRequest::new("https://jsonplaceholder.typicode.com/todos/1", Method::Get);
    dispatch_http_call(req, |response| {
        log!("http callback hit!");
        log!("{:?}", response);
    });

    let files_ls = IPFSCommand::FilesLs(FilesLsOpts::default());
    dispatch_ipfs_call(files_ls, |response| {
        log!("ipfs callback hit!");
        let response_str = String::from_utf8(response.unwrap()).unwrap();
        log!("{:?}", response_str);
    });
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_http_call() {
        let req = HttpRequest::new("https://jsonplaceholder.typicode.com/todos/1", Method::Get);

        dispatch_http_call(req, |response| {
            log!("callback hit!");
            log!("{:?}", response);
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