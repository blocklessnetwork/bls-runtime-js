import init, { run } from "../pkg";
await init();

const guestImports = {
    blockless: {
        run,
    },
};
WebAssembly.instantiateStreaming(fetch("./simple.wasm"), guestImports).then(
    (obj) => (obj.instance.exports as any).exported_func(),
);
