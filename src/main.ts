// load host wasm module (isomorphic bls runtime/extensions)
import init, { Blockless } from "../bls-runtime-wasm/pkg";
import WASI from "wasi-js";
import browserBindings from "wasi-js/dist/bindings/browser";

await init();

// setup browser based WASI
import { fs } from 'memfs';
// override default `writeSync` behaviour to parse html and append to document
(fs as any).writeSync = (fd, buffer, offset, length, position, callback) => {
    const string = new TextDecoder().decode(buffer);
    writeOutputToDOM(string);
    if (callback) {
        callback(null, length);
    }
};
const wasi = new WASI({
    args: ["--my-arg"],
    env: {
        FOO: "FOO",
        BAR: "BAR",
        BLS_REQUEST_METHOD: "GET",
        BLS_REQUEST_PATH: "/",
        BLS_REQUEST_QUERY: "",
    },
    bindings: { ...browserBindings, fs },
});

const bls = new Blockless({
    env: {
        BLS_REQUEST_METHOD: "GET",
        BLS_REQUEST_PATH: "/",
        BLS_REQUEST_QUERY: "",
    },
    args: ["--my-arg"],
    // "fs_root_path": "/", 
    // "drivers_root_path": "/drivers", 
    // "runtime_logger": "runtime.log", 
    // "limited_fuel": 200000000,
    // "limited_memory": 30,
    // "debug_info": false,
    // "entry": "lib.wasm",
    permissions: [
        "https://jsonplaceholder.typicode.com/todos/1",
        "http://httpbin.org/anything",
        "file://a.go"
    ],
    fs,
});

// const wasmPath = "../simple.wasm";
// const wasmPath = "../basics.wasm";
// const wasmPath = "../release.wasm";
// const wasmPath = "../target/wasm32-unknown-unknown/release/rust_sdk.wasm";
const wasmPath = "../crates/rust-sdk/target/wasm32-wasi/release/rust_sdk.wasm";
const wasmModule = await WebAssembly.compileStreaming(fetch(wasmPath));

const wasiImports = wasi.getImports(wasmModule);

const imports = {
    ...wasiImports,
    // wasi_snapshot_preview1: wasi.wasiImport,
    // wasi_snapshot_preview1: {
    //     ...wasiImports.wasi_snapshot_preview1,
    //     // environ_sizes_get(){ return 0; },
    //     // environ_get() { return 0; },
    //     // proc_exit() { return 0; },
    //     // path_open() { return 0; },
    //     // fd_close() { return 0; },
    //     // fd_prestat_dir_name() { return 0; },
    //     // fd_prestat_get() { return 0; },
    //     // fd_write(fd, iovsPtr, iovsLength, bytesWrittenPtr){
    //     //     const iovs = new Uint32Array(instance.exports.memory.buffer, iovsPtr, iovsLength * 2);
    //     //     if(fd === 1) { //stdout
    //     //         let text = "";
    //     //         let totalBytesWritten = 0;
    //     //         const decoder = new TextDecoder();
    //     //         for(let i =0; i < iovsLength * 2; i += 2){
    //     //             const offset = iovs[i];
    //     //             const length = iovs[i+1];
    //     //             const textChunk = decoder.decode(new Int8Array(instance.exports.memory.buffer, offset, length));
    //     //             text += textChunk;
    //     //             totalBytesWritten += length;
    //     //         }
    //     //         const dataView = new DataView(instance.exports.memory.buffer);
    //     //         dataView.setInt32(bytesWrittenPtr, totalBytesWritten, true);
    //     //         console.log(text);
    //     //     }
    //     //     return 0;
    //     // },
    // },
    browser: {
        run_reqwest: (ptr: number, len: number) => {
            const instance = (globalThis as any).instance; // global variable value set by bls.instantiate
            console.info("run_reqwest from browser import");
            const memory = new Uint8Array((instance.exports.memory as any).buffer, ptr, len);
            const url = new TextDecoder().decode(memory);
            console.info(url);
        },
    },
};
// const { instance: inst, module } = await WebAssembly.instantiateStreaming(fetch(wasmPath), imports);

// const wasmInstance = await WebAssembly.instantiate(wasmModule, imports);
// (wasmInstance.exports as any)._start();

// const wasmInstance = await WebAssembly.instantiate(wasmModule, imports);
const wasmInstance = bls.instantiate(wasmModule, {});
// wasi.setMemory(wasmInstance.exports.memory as any); // set memory for wasi

// // wasi.start(wasmInstance);
const exitCode = bls.start();
const stdout = bls.getStdoutString();
console.log(`${stdout}\n(exit code: ${exitCode})`);
writeOutputToDOM(stdout);

// console.log("running!");

// const wasmInstance = bls.instantiate(wasmModule, {});
// const wasmerExitCode = bls.start();
// const stdout = bls.getStdoutString();
// console.log(`${stdout}\n(exit code: ${wasmerExitCode})`);
// writeOutputToDOM(stdout);

// import { init as initWasmer, WASI as WASIWasmer } from '@wasmer/wasi';
// import initWasmer, { WASI as WASIWasmer } from '../../../wasmer-js/pkg';

// await initWasmer();
// const wasiWasmer = new WASIWasmer({
//     env: {
//         BLS_REQUEST_METHOD: "GET",
//         BLS_REQUEST_PATH: "/",
//         BLS_REQUEST_QUERY: "",
//     },
//     // fs: fs as any,
//     // args: ["--my-arg"],
// });
// wasiWasmer.instantiate(wasmModule, {});
// const wasmerExitCode = wasiWasmer.start();
// const stdout = wasiWasmer.getStdoutString();
// console.log(`${stdout}(exit code: ${wasmerExitCode})`);
// writeOutputToDOM(stdout);





// import initWasmer, { initSync, run, Runtime } from '../../../wasmer-js/pkg';

// await initWasmer();
// const runtime = new Runtime(1);
// const instance = run(wasmModule, runtime, {
//     env: {
//         BLS_REQUEST_METHOD: "GET",
//         BLS_REQUEST_PATH: "/",
//         BLS_REQUEST_QUERY: "",
//     },
//     // fs: fs as any,
//     // args: ["--my-arg"],
// });
// const output = await instance.wait();


// Read a string from the instance's memory.
// function readString(ptr: number, len: number, instance: WebAssembly.Instance) {
//     const m = new Uint8Array((instance.exports.memory as any).buffer, ptr, len);
//     const decoder = new TextDecoder("utf-8");
//     // return a slice of size `len` from the module's
//     // memory, starting at offset `ptr`
//     return decoder.decode(m.slice(0, len));
// }

// function deallocGuestMemory(ptr: number, len: number, instance: WebAssembly.Instance) {
//     // call the module's `dealloc` function
//     (instance.exports as any).dealloc(ptr, len);
// }

// // Copy `data` into the `instance` exported memory buffer.
// function copyMemory(instance: WebAssembly.Instance, data: Uint8Array) {
//     // the `alloc` function returns an offset in
//     // the module's memory to the start of the block
//     const ptr = (instance.exports as any).alloc(data.length);

//     // create a typed `ArrayBuffer` at `ptr` of proper size
//     const mem = new Uint8Array((instance.exports.memory as any).buffer, ptr, data.length);
//     // copy the content of `data` into the memory buffer
//     mem.set(new Uint8Array(data));
//     // return the pointer
//     return ptr;
// }

function writeOutputToDOM(out: string) {
    const parser = new DOMParser();
    const htmlDoc = parser.parseFromString(out, "text/html");

    // Get the <head> element from the parsed document
    const parsedHead = htmlDoc.head;

    // Get the main document's <head> element
    const mainDocumentHead = document.head;

    // Move all children from parsedHead to mainDocumentHead
    while (parsedHead.firstChild) {
        mainDocumentHead.appendChild(parsedHead.firstChild);
    }

    // Get the <body> element from the parsed document
    const parsedBody = htmlDoc.body;

    // Get the main document's <body> element
    const mainDocumentBody = document.body;

    // Move all children from parsedBody to mainDocumentBody
    while (parsedBody.firstChild) {
        mainDocumentBody.appendChild(parsedBody.firstChild);
    }
}
