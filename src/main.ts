import { createHelia } from 'helia'
import { mfs } from '@helia/mfs'
// import { IDBBlockstore } from 'blockstore-idb'
import { MemoryBlockstore } from 'blockstore-core'
import { MemoryDatastore } from 'datastore-core'
import { Buffer } from 'buffer'

// TODO: use persistent blockstore and datastore

// create a Helia node
const helia = await createHelia({
    datastore: new MemoryDatastore(),
    // blockstore: new IDBBlockstore("/ipfs-js"), // create a blockstore - IndexedDB
    blockstore: new MemoryBlockstore(),
})

console.info("running Helia node...");

const heliaFS = mfs(helia)
const encoder = new TextEncoder(); // turn strings into Uint8Arrays
await heliaFS.writeBytes(encoder.encode('Hello World 201'), "/abc.txt")
// for await (const entry of heliaFS.ls('/')) {
//     console.info(entry)
// }

// TODO: look at SharedWorker API: https://developer.mozilla.org/en-US/docs/Web/API/SharedWorker
navigator.serviceWorker.addEventListener('message', async (event) => {
    // handle the IPFS request using heliaFS
    if (event.data.type === 'IPFS_REQUEST') {
      const ipfsRequest = event.data.request;
      // reconstruct request
      const request = new Request(ipfsRequest.url, {
        method: ipfsRequest.method,
        // headers: ipfsRequest.headers,
        body: ipfsRequest.body,
      });
      try {
        const response = await handleIPFSRequest(request);
        event.ports[0].postMessage({ response: JSON.stringify(response) });
      } catch (error: any) {
        console.warn('error handling IPFS request:', error);
        event.ports[0].postMessage({ error: error.message });
      }
    }
});

async function handleIPFSRequest(request: Request) {
    const url = new URL(request.url);
    const command = url.pathname.replace('/api/v0/', '');
  
    // create object from query params
    const queryParams = Object.fromEntries(url.searchParams);
    if (command.startsWith('files')) {
      console.debug('files command detected');
      return await handleFilesCommand(command.replace('files/', ''), queryParams);
    }
    if (command.startsWith('version')) {
        // https://github.com/ipfs/helia/wiki/Migrating-from-js-IPFS#version
        // TODO: return package.json version + other infor about helia (browser IPFS node)
      return "unsupported version command"
    }
  
    return "unsupported command";
}

async function handleFilesCommand(command: string, params: any) { 
    if (command.startsWith('write')) {
        console.debug('write command detected');
        await heliaFS.writeBytes(Buffer.from(params.data), params.arg);
        return {};
    }
    if (command.startsWith('ls')) {
        console.debug('ls command detected');
        const entries = []
        for await (const entry of heliaFS.ls(params.arg)) {
            entries.push({ Name: entry.name, Type: entry.type, Size: +entry.size.toString(), Hash: Buffer.from(entry.cid.multihash.digest).toString('hex') });
        }
        return { Entries: entries };
    }
    return "unsupported files command";
}

// TODO: take a quick look at http api server (uses fastify)
// - https://github.com/ipfs/helia-routing-v1-http-api/tree/main/packages/server
// TODO: can we somehow use browser-based server to forward requests to heliaFS?
// - look at service worker request hijacking?
// - look at browser based wasm servers?
// - look at in-browser CGI?

// fetch IPFS `/`
const result = await fetch("http://127.0.0.1:5001/api/v0/files/ls?arg=/", {
    method: "POST",
    mode: "cors",
});
const body = await result.json();
console.log(body);


// // load host wasm module (isomorphic bls runtime/extensions)
// import init, { Blockless } from "../bls-runtime-wasm/pkg";
// await init();

// const bls = new Blockless({
//     env: {
//         BLS_RUNTIME: "browser",
//         BLS_REQUEST_METHOD: "GET",
//         BLS_REQUEST_PATH: "/",
//         BLS_REQUEST_QUERY: "",
//     },
//     args: ["--my-arg"],
//     // "fs_root_path": "/", 
//     // "drivers_root_path": "/drivers", 
//     // "runtime_logger": "runtime.log", 
//     // "limited_fuel": 200000000,
//     // "limited_memory": 30,
//     // "debug_info": false,
//     // "entry": "lib.wasm",
//     permissions: [
//         "https://jsonplaceholder.typicode.com/todos/1",
//         "http://httpbin.org/anything",
//         "file://a.go"
//     ],
//     // fs,
// });

// // const wasmPath = "../simple.wasm";
// // const wasmPath = "../basics.wasm";
// // const wasmPath = "../release.wasm";
// // const wasmPath = "../crates/rust-sdk/target/wasm32-unknown-unknown/release/rust_sdk.wasm";
// const wasmPath = "../crates/rust-sdk/target/wasm32-wasi/release/rust_sdk.wasm";
// // const wasmPath = "../release-asc.wasm";
// const wasmModule = await WebAssembly.compileStreaming(fetch(wasmPath));
// const _wasmInstance = bls.instantiate(wasmModule, {});
// const exitCode = bls.start();
// const stdout = bls.getStdoutString();

// console.info(`${stdout}\n(exit code: ${exitCode})`);
// writeOutputToDOM(stdout);

// function writeOutputToDOM(out: string) {
//     const parser = new DOMParser();
//     const htmlDoc = parser.parseFromString(out, "text/html");
//     const parsedHead = htmlDoc.head; // get the <head> element from the parsed document
//     const mainDocumentHead = document.head; // get the main document's <head> element

//     // move all children from parsedHead to mainDocumentHead
//     while (parsedHead.firstChild) {
//         mainDocumentHead.appendChild(parsedHead.firstChild);
//     }

//     const parsedBody = htmlDoc.body; // get the <body> element from the parsed document
//     const mainDocumentBody = document.body; // get the main document's <body> element

//     // move all children from parsedBody to mainDocumentBody
//     while (parsedBody.firstChild) {
//         mainDocumentBody.appendChild(parsedBody.firstChild);
//     }
// }

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
