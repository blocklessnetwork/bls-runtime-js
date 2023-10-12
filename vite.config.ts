import { defineConfig } from 'vite';
import nodePolyfills from 'vite-plugin-node-stdlib-browser';

// run local IPFS node (docker) with CORS enabled
// docker run --rm -it --name ipfs-node -p 4001:4001 -p 5001:5001 --entrypoint sh ipfs/go-ipfs:latest
// /usr/local/bin/start_ipfs config --json API.HTTPHeaders.Access-Control-Allow-Origin "[\"*\"]"
// /usr/local/bin/start_ipfs daemon --migrate=true --agent-version-suffix=docker 

export default defineConfig({
  server: {
    port: 8080,
  },
	build: {
		target: 'esnext', // support top-level await
	},
  plugins: [nodePolyfills()]
});


// import { defineConfig } from 'vite';
// import wasmPack from 'vite-plugin-wasm-pack';
// import wasm from "vite-plugin-wasm";
// import topLevelAwait from "vite-plugin-top-level-await";

// export default defineConfig({
//   server: {
//     port: 8080,
//   },
//   build: {
//     target: 'esnext',
//   },
//   plugins: [
//     // wasm(),
//     // wasmPack('.'),
//     // topLevelAwait()
//   ],
//   define: {
//     'process.env': process.env,
//     // 'global': globalThis,
//   }
// });