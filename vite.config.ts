// import { defineConfig } from 'vite';
// import nodePolyfills from 'vite-plugin-node-stdlib-browser'

// export default defineConfig({
//   server: {
//     port: 8080,
//   },
// 	build: {
// 		target: 'esnext', // support top-level await
// 	},
//   plugins: [nodePolyfills()]
// });


import { defineConfig } from 'vite';
// import wasmPack from 'vite-plugin-wasm-pack';
// import wasm from "vite-plugin-wasm";
// import topLevelAwait from "vite-plugin-top-level-await";

export default defineConfig({
  server: {
    port: 8080,
  },
  build: {
    target: 'esnext',
  },
  plugins: [
    // wasm(),
    // wasmPack('.'),
    // topLevelAwait()
  ],
});