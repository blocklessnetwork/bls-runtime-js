
### Install dependencies\

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
cargo build -p rust-sdk --target wasm32-unknown-unknown --release
```

### Run the development server (vite).

```sh
npm run dev
```
