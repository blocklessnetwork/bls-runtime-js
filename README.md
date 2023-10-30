# Blockless Isomorphic Runtime

A wasi-compatible isomorphic runtime.

![Blockless Runtime Logo](https://github.com/blocklessnetwork/bls-runtime/blob/main/blockless.png?raw=true)

## Features

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

##  Testing Blockless extensions

### S3

First run localstack locally:
```sh
docker run \
  --rm -it \
  -e EXTRA_CORS_ALLOWED_ORIGINS="http://localhost:8080" \
  -p 4566:4566 \
  -p 4510-4559:4510-4559 \
  localstack/localstack
```

### IPFS

Run local IPFS node (with CORS):
```sh
docker run --rm -it --name ipfs-node -p 4001:4001 -p 5001:5001 --entrypoint sh ipfs/go-ipfs:latest
/usr/local/bin/start_ipfs config --json API.HTTPHeaders.Access-Control-Allow-Origin "[\"*\"]"
/usr/local/bin/start_ipfs daemon --migrate=true --agent-version-suffix=docker
```
