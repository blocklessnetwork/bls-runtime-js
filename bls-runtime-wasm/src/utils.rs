use js_sys::{Function, Reflect, Uint8Array, WebAssembly::{self, Instance}};
use wasm_bindgen::prelude::{JsCast, JsValue};

/// Decodes the data from the guest's memory and returns it as a Rust Vec<u8>.
/// The given pointer is the offset from the start of the guest's memory.
pub fn decode_data_from_memory(instance: &Instance, ptr: u32, len: u32) -> Vec<u8> {
    let exports = instance.exports();
    let memory = Reflect::get(&exports, &"memory".into())
        .expect("memory export wasn't found")
        .dyn_into::<WebAssembly::Memory>()
        .expect("memory export wasn't a `WebAssembly.Memory`");

    // create a Uint8Array to hold the data
    let memory_array: Uint8Array = Uint8Array::new(&memory.buffer()).subarray(ptr, ptr + len);

    // convert Uint8Array to Rust Vec<u8>
    let mut vec = vec![0; len as usize];
    memory_array.copy_to(&mut vec);
    vec
}

/// Encodes the given data into the guest's memory and returns the pointer to the data.
/// The returned pointer is the offset from the start of the guest's memory.
/// The first byte at the returned pointer is the length of the data.
/// The rest of the bytes are the data itself.
/// NOTE: the caller is responsible for deallocating the memory.
pub fn encode_data_to_memory(instance: &Instance, data: &[u8]) -> u32 {
    let exports = instance.exports();
    let memory = Reflect::get(&exports, &"memory".into())
        .expect("memory export wasn't found")
        .dyn_into::<WebAssembly::Memory>()
        .expect("memory export wasn't a `WebAssembly.Memory`");

    let alloc_func = Reflect::get(&exports, &"alloc".into())
        .expect("alloc export wasn't found")
        .dyn_into::<Function>()
        .expect("alloc export wasn't a `Function`");

    // NOTE: 1st byte represents the length of the result
    let result_ptr = alloc_func
        .call1(&JsValue::undefined(), &JsValue::from((data.len() + 1) as u32))
        .unwrap()
        .as_f64()
        .expect("failed to convert return value from `alloc`") as u32;

    // copy the returned result into guest's memory
    let mem_array = Uint8Array::new(&memory.buffer());

    // NOTE: 1st byte represents the length of the result
    mem_array.set_index(result_ptr as u32, data.len() as u8);

    for (i, &byte) in data.iter().enumerate() {
        mem_array.set_index(1 + (result_ptr as u32) + (i as u32), byte);
    }
    result_ptr
}
