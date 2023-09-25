# Blockless Isomorphic Runtime

A wasi-compatible isomorphic runtime for blockless.

The runtime itself a wasm module (host) that is loaded in an environment.
The runtime loads a guest wasm module and executes it.

The runtime provides the guest with a set of APIs to interact with the host; the host injects WASI APIs in addition to the blockless APIs as as extensions to the guest.

### Install dependencies

```sh
npm install
```

### Compile isomorphic host blockless runtime to wasm

```sh
wasm-pack build bls-runtime-wasm --target web --release
```

Note: Requires [wasm-pack]()

### Compile guest app to wasm

```sh
cargo build -p rust-sdk --target wasm32-wasi --release
```

### Run the development server (vite).

```sh
npm run dev
```
