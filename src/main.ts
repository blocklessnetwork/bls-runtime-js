// load host wasm module (isomorphic bls runtime/extensions)
import init, { Blockless } from "../bls-runtime-wasm/pkg";
await init();

const bls = new Blockless({
    env: {},
    args: [],
});

const wasmPath = "../target/wasm32-unknown-unknown/release/rust_sdk.wasm";
// const { instance, module } = await WebAssembly.instantiateStreaming(fetch(wasmPath), {});
const module = await WebAssembly.compileStreaming(fetch(wasmPath));
bls.instantiate(module, {
    // blockless: {
    //     host_log: (ptr: number, len: number) => {
    //         console.log("host_log");
    //     }
    // }
});

const exitCode = bls.start();
console.log("Exit code: " + exitCode);


// // specify imports for the guest wasm module
// const guestImports = {
//     blockless: {
//         host_call: (ptr: number, len: number) => {
//             // NOTE: uses instance from below (global)
//             const memory = new Uint8Array((instance.exports.memory as any).buffer, ptr, len);
//             // const data = new TextDecoder().decode(memory);
//             // console.log(data);
//             host_call(memory);
//         },
//     },
// };

// // const wasmPath = "../simple.wasm";
// const wasmPath = "../target/wasm32-unknown-unknown/release/rust_sdk.wasm";
// const { instance, module } = await WebAssembly.instantiateStreaming(fetch(wasmPath), guestImports);
// (instance.exports as any).start();

// // arraySum([1, 2, 3, 4, 5], instance);

// Invoke the `array_sum` exported method and
// log the result to the console
function arraySum(array: Uint8Array, instance: WebAssembly.Instance) {
    // copy the contents of `array` into the
    // module's memory and get the offset
    const ptr = copyMemory(instance, array);
    // invoke the module's `array_sum` exported function
    // and log the result
    const res = (instance.exports as any).array_sum(ptr, array.length);
    console.log("Result: " + res);
}

function upper(input: string, instance: WebAssembly.Instance) {
    // transform the input string into its UTF-8
    // representation
    const bytes = new TextEncoder().encode(input);
    // copy the contents of the string into
    // the module's memory
    const ptr = copyMemory(instance, bytes);
    // call the module's `upper` function and
    // get the offset into the memory where the
    // module wrote the result string
    const res_ptr = (instance.exports as any).upper(ptr, bytes.length);
    // read the string from the module's memory,
    // store it, and log it to the console
    const result = readString(res_ptr, bytes.length, instance);
    console.log(result);
    // the JavaScript runtime took ownership of the
    // data returned by the module, which did not
    // deallocate it - so we need to clean it up
    deallocGuestMemory(res_ptr, bytes.length, instance);
}

// Read a string from the instance's memory.
function readString(ptr: number, len: number, instance: WebAssembly.Instance) {
    const m = new Uint8Array((instance.exports.memory as any).buffer, ptr, len);
    const decoder = new TextDecoder("utf-8");
    // return a slice of size `len` from the module's
    // memory, starting at offset `ptr`
    return decoder.decode(m.slice(0, len));
}

function deallocGuestMemory(ptr: number, len: number, instance: WebAssembly.Instance) {
    // call the module's `dealloc` function
    (instance.exports as any).dealloc(ptr, len);
}

// Copy `data` into the `instance` exported memory buffer.
function copyMemory(instance: WebAssembly.Instance, data: Uint8Array) {
    // the `alloc` function returns an offset in
    // the module's memory to the start of the block
    const ptr = (instance.exports as any).alloc(data.length);

    // create a typed `ArrayBuffer` at `ptr` of proper size
    const mem = new Uint8Array((instance.exports.memory as any).buffer, ptr, data.length);
    // copy the content of `data` into the memory buffer
    mem.set(new Uint8Array(data));
    // return the pointer
    return ptr;
}
