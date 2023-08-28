
// import { run } from './pkg';
// const res = await run('wit-bindgen');
// console.log(res)

// import init, { run } from './pkg';
// await init();

// const res = await run('wit-bindgen');
// console.log(res)

// const guestImports = {
//     blockless: {
//       run,
//     },
// };
// const { instance, module } = await WebAssembly.instantiateStreaming(
//     fetch("./exam1.wasm"),
//     guestImports,
// );
// console.log(instance.exports.run("wit-bindgen"));

const importObject = {
    blockless: { run: (arg) => console.log(arg) },
};
WebAssembly.instantiateStreaming(fetch("./simple.wasm"), importObject).then(
    (obj) => obj.instance.exports.exported_func(),
);