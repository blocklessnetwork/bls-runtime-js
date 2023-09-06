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

    // reads data from the guest's memory into u8 array - based on the given pointer and length
    let memory_array: Uint8Array = Uint8Array::new(&memory.buffer()).subarray(ptr, ptr + len);
    memory_array.to_vec()
}

/// Encodes the given data into the guest's memory and returns the pointer to the data.
/// The returned pointer is the offset from the start of the guest's memory.
/// The first 4 bytes (u32) at the returned pointer is the length of the data (max length of u32).
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

    // NOTE: first 4 bytes represent the length of the result
    let result_ptr = alloc_func
        .call1(&JsValue::undefined(), &JsValue::from((data.len() as u32) + 4))
        .unwrap()
        .as_f64()
        .expect("failed to convert return value from `alloc`") as u32;

    // copy the returned result into guest's memory
    let mem_array = Uint8Array::new(&memory.buffer());
    let length_array = (data.len() as u32).to_le_bytes(); // convert to little-endian bytes
    for (i, &byte) in length_array.iter().enumerate() {
        mem_array.set_index(result_ptr + i as u32, byte);
    }
    mem_array.set(&Uint8Array::from(data), result_ptr + 4);

    result_ptr
}