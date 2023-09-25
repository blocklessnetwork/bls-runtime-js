// load host wasm module (isomorphic bls runtime/extensions)
import init, { Blockless } from "../bls-runtime-wasm/pkg";
await init();

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
    // fs,
});

// const wasmPath = "../simple.wasm";
// const wasmPath = "../basics.wasm";
// const wasmPath = "../release.wasm";
// const wasmPath = "../crates/rust-sdk/target/wasm32-unknown-unknown/release/rust_sdk.wasm";
const wasmPath = "../crates/rust-sdk/target/wasm32-wasi/release/rust_sdk.wasm";
const wasmModule = await WebAssembly.compileStreaming(fetch(wasmPath));
const _wasmInstance = bls.instantiate(wasmModule, {});
const exitCode = bls.start();
const stdout = bls.getStdoutString();

console.info(`${stdout}\n(exit code: ${exitCode})`);
writeOutputToDOM(stdout);

function writeOutputToDOM(out: string) {
    const parser = new DOMParser();
    const htmlDoc = parser.parseFromString(out, "text/html");
    const parsedHead = htmlDoc.head; // get the <head> element from the parsed document
    const mainDocumentHead = document.head; // get the main document's <head> element

    // move all children from parsedHead to mainDocumentHead
    while (parsedHead.firstChild) {
        mainDocumentHead.appendChild(parsedHead.firstChild);
    }

    const parsedBody = htmlDoc.body; // get the <body> element from the parsed document
    const mainDocumentBody = document.body; // get the main document's <body> element

    // move all children from parsedBody to mainDocumentBody
    while (parsedBody.firstChild) {
        mainDocumentBody.appendChild(parsedBody.firstChild);
    }
}

// import WASI from "wasi-js";
// import browserBindings from "wasi-js/dist/bindings/browser";
// // setup browser based WASI
// import { fs } from 'memfs';
// // override default `writeSync` behaviour to parse html and append to document
// (fs as any).writeSync = (fd, buffer, offset, length, position, callback) => {
//     const string = new TextDecoder().decode(buffer);
//     writeOutputToDOM(string);
//     if (callback) {
//         callback(null, length);
//     }
// };
// const wasi = new WASI({
//     args: ["--my-arg"],
//     env: {
//         FOO: "FOO",
//         BAR: "BAR",
//         BLS_REQUEST_METHOD: "GET",
//         BLS_REQUEST_PATH: "/",
//         BLS_REQUEST_QUERY: "",
//     },
//     bindings: { ...browserBindings, fs },
// });
// const wasiImports = wasi.getImports(wasmModule);
// const imports = {
//     ...wasiImports,
//     // wasi_snapshot_preview1: wasi.wasiImport,
//     browser: {
//         run_reqwest: (ptr: number, len: number) => {
//             const instance = (globalThis as any).instance; // global variable value set by bls.instantiate
//             console.info("run_reqwest from browser import");
//             const memory = new Uint8Array((instance.exports.memory as any).buffer, ptr, len);
//             const url = new TextDecoder().decode(memory);
//             console.info(url);
//         },
//     },
// };
// const { instance: inst, module } = await WebAssembly.instantiateStreaming(fetch(wasmPath), imports);
// const wasmInstance = await WebAssembly.instantiate(wasmModule, imports);
// (wasmInstance.exports as any)._start();


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
