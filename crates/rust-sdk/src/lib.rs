use std::cell::RefCell;
use bls_common::{
    http::{Method, HttpRequest, HttpResponse},
    types::{ModuleCall, ModuleCallResponse},
};

mod utils;

#[link(wasm_import_module = "blockless")]
extern "C" {
    #[link_name = "host_log"]
    pub fn host_log(ptr: u32, len: u32);
}

#[link(wasm_import_module = "blockless")]
extern "C" {
    #[link_name = "host_call"]
    pub fn host_call(ptr: u32, len: u32) -> u32;
}

#[link(wasm_import_module = "blockless")]
extern "C" {
    #[link_name = "http_call"]
    pub fn http_call(ptr: u32, len: u32) -> u32;
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
pub fn blockless_callback(result_ptr: usize) -> *const u8 {
    let serialized = read_data_from_ptr(result_ptr);
    let module_call_response: ModuleCallResponse = serde_json::from_slice(&serialized[..]).unwrap(); // TODO: handle error

    // call the callback function
    MODULE_CALLBACK.with(|callback| {
        if let Some(func) = *callback.borrow() {
            func(module_call_response);
        }
    });

    return 0 as *const u8;
}

#[no_mangle]
pub fn http_callback(result_ptr: usize) -> *const u8 {
    let serialized = read_data_from_ptr(result_ptr);
    let http_call_response: Result<HttpResponse, String> = serde_json::from_slice(&serialized[..]).unwrap(); // TODO: handle error

    // call the callback function
    HTTP_CALLBACK.with(|callback| {
        if let Some(func) = *callback.borrow() {
            func(http_call_response);
        }
    });

    return 0 as *const u8;
}

fn read_data_from_ptr(result_ptr: usize) -> Vec<u8> {
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
    static MODULE_CALLBACK: RefCell<Option<fn(ModuleCallResponse)>> = RefCell::new(None);
    static HTTP_CALLBACK: RefCell<Option<fn(Result<HttpResponse, String>)>> = RefCell::new(None);
}

pub fn dispatch_host_call(module_call: ModuleCall, callback_fn: fn(ModuleCallResponse)) {
    let data = serde_json::to_vec(&module_call).unwrap();
    let ptr = data.as_ptr() as u32;
    let len = data.len() as u32;

    let result_ptr = unsafe { host_call(ptr, len) };
    if result_ptr == 0 {
        // only register callback if the host call was successful (early return)
        MODULE_CALLBACK.with(|callback| {
            *callback.borrow_mut() = Some(callback_fn);
        });
        return;
    }
    let error_response = read_data_from_ptr(result_ptr as usize);

    // let input_str = String::from_utf8(error_response).unwrap();
    // println!("input_str: {}", input_str);

    let deserialized: ModuleCallResponse = serde_json::from_slice(&error_response[..]).unwrap();
    callback_fn(deserialized);
}

pub fn dispatch_http_call(module_call: HttpRequest, callback_fn: fn(Result<HttpResponse, String>)) {
    let data = serde_json::to_vec(&module_call).unwrap();
    let ptr = data.as_ptr() as u32;
    let len = data.len() as u32;

    let result_ptr = unsafe { http_call(ptr, len) };
    if result_ptr == 0 {
        HTTP_CALLBACK.with(|callback| {
            *callback.borrow_mut() = Some(callback_fn);
        });
        return;
    }
    let error_response = read_data_from_ptr(result_ptr as usize);
    let str_error_response = String::from_utf8(error_response).unwrap();

    callback_fn(Err(str_error_response));
}

#[no_mangle]
pub fn _start() {
    let req = HttpRequest::new("https://jsonplaceholder.typicode.com/todos/1", Method::Get);

    // let http_call = ModuleCall::Http(req);
    // dispatch_host_call(http_call, |response| {
    //     log!("callback hit!");
    //     log!("{}", response);
    // });

    dispatch_http_call(req, |response| {
        log!("callback hit!");
        log!("{:?}", response);
    });
}
