/// SOURCE: https://radu-matei.com/blog/practical-guide-to-wasm-memory/

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

// #[link(wasm_import_module = "blockless")]
// extern "C" {
//     #[link_name = "host_request"]
//     pub fn host_request(ptr: u32, len: u32) -> u32;
// }

// #[link(wasm_import_module = "blockless")]
// extern "C" {
//     #[link_name = "host_query"]
//     pub fn host_query(request_id: u32) -> u32;
// }

#[link(wasm_import_module = "browser")]
extern "C" {
    #[link_name = "run_reqwest"]
    pub fn run_reqwest(ptr: u32, len: u32) -> u32;
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

// /// Given a pointer to the start of a byte array and
// /// its length, return the sum of its elements.
// #[no_mangle]
// pub unsafe fn array_sum(ptr: *mut u8, len: usize) -> u8 {
//     // create a Vec<u8> from the pointer to the
//     // linear memory and the length
//     let data = Vec::from_raw_parts(ptr, len, len);
//     // actually compute the sum and return it
//     data.iter().sum()
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
pub unsafe fn upper(ptr: *mut u8, len: usize) -> *mut u8 {
    // create a `Vec<u8>` from the pointer and length
    // here we could also use Rust's excellent FFI
    // libraries to read a string, but for simplicity,
    // we are using the same method as for plain byte arrays
    let data = Vec::from_raw_parts(ptr, len, len);
    // read a Rust `String` from the byte array,
    let input_str = String::from_utf8(data).unwrap();
    // transform the string to uppercase, then turn it into owned bytes
    let mut upper = input_str.to_ascii_uppercase().as_bytes().to_owned();
    let ptr = upper.as_mut_ptr();
    // take ownership of the memory block where the result string
    // is written and ensure its destructor is not
    // called whe the object goes out of scope
    // at the end of the function
    std::mem::forget(upper);
    // return the pointer to the uppercase string
    // so the runtime can read data from this offset
    ptr
}

// #[no_mangle]
// pub fn _start() {
//     let url = "https://reqres.in/api/products";
//     // let url_bytes = url.as_bytes();

//     let ptr = url.as_ptr() as u32;
//     let len = url.len() as u32;
//     unsafe {
//         let request_id = host_request(ptr, len);
//         let result_ptr = host_query(request_id);
//         while result_ptr == 0 {
//             // wait for the result
//         }
//         let result_len = *(result_ptr as *const u8);
//         host_log(result_ptr + 1, result_len as u32);
//     }
// }

#[no_mangle]
pub fn _start() {
    let url = "https://reqres.in/api/products";
    let data = url.as_bytes();

    let ptr = data.as_ptr() as u32;
    let len = data.len() as u32;
    unsafe {
        let result_ptr = host_call(ptr, len);
        // 1st byte at result_ptr is the length of the result
        let result_len = *(result_ptr as *const u8);
        host_log(result_ptr + 1, result_len as u32);
    }
}

// // Entrypoint for the wasm module
// #[no_mangle]
// pub extern "C" fn _start() {
//     pub fn main() {
//         // println!("Hello, world!");
//         unsafe {
//             host_call(1);
//         }
//     }
// }
