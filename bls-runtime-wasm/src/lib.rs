#![allow(unused_variables)]
#![allow(unused_imports)]

use std::{sync::{Arc, Mutex}, cell::Ref};
use std::cell::RefCell;
use std::io::{Read, Write};

pub mod fs;
pub mod utils;

use bls_common::{types::{ModuleCall, ModuleCallResponse}, http::{HttpResponse, HttpRequest}};

use serde::{Deserialize, Serialize};
use js_sys::{Map, Object, Reflect, WebAssembly};
use wasm_bindgen::{JsValue, JsCast, JsError};
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen_futures::{JsFuture, spawn_local};
use web_sys::{Request, RequestInit, RequestMode, Response, console};
use wasmer::{imports, Imports, Exports, ExternType, TypedFunction, Instance, Module, Store, Function, FunctionEnv, FunctionEnvMut, Memory, AsStoreRef, AsStoreMut, Value, MemoryView};
use wasmer_wasi::Pipe;
use wasmer::NativeWasmTypeInto;
use wasmer_wasi::{WasiError, WasiFunctionEnv, WasiState};

// https://github.com/rustwasm/console_error_panic_hook
#[wasm_bindgen]
pub fn init_panic_hook() {
    console_error_panic_hook::set_once();
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    pub(crate) fn log(a: &str);
    #[wasm_bindgen(js_namespace = console)]
    pub(crate) fn error(a: &str);
}
macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}
macro_rules! console_error {
    ($($t:tt)*) => (error(&format_args!($($t)*).to_string()))
}

// const WASM: &[u8] = include_bytes!("../../target/wasm32-unknown-unknown/release/rust_sdk.wasm");
// const WASM: &[u8] = include_bytes!("../../simple.wasm");
// #[wasm_bindgen(start)]
// fn start() {
//     wasm_bindgen_futures::spawn_local(async {
//         match run_wasm(WASM).await {
//             Ok(_) => console_log!("successfully finished running wasm module"),
//             Err(e) => console_error!("{:?}", e),
//         }
//     });
// }


#[wasm_bindgen(typescript_custom_section)]
const BLOCKLESS_CONFIG_TYPE_DEFINITION: &str = r#"
/** Options used when configuring a new WASI instance.  */
export type BlocklessConfig = {
    /** The command-line arguments passed to the WASI executable. */
    readonly args?: string[];
    /** Additional environment variables made available to the WASI executable. */
    readonly env?: Record<string, string>;
    /** Preopened directories. */
    readonly preopens?: Record<string, string>;
    /** Additional permissions. */
    readonly permissions?: string[];
    /** The in-memory filesystem that should be used. */
    readonly fs?: MemFS;
};
"#;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "BlocklessConfig")]
    pub type BlocklessConfig;
}

// use wasmer::{Imports, Instance, Module, Store, AsJs};
// use wasmer_wasix::{
//     virtual_fs::{Pipe, PipeTx, PipeRx, DuplexPipe},
//     WasiError, WasiFunctionEnv, WasiEnvBuilder, runtime::{task_manager::WebThreadPool, web::WebRuntime}
// };

#[wasm_bindgen]
pub struct Blockless {
    store: Store,
    stdout: Pipe,
    stdin: Pipe,
    stderr: Pipe,
    wasi_env: WasiFunctionEnv,
    permissions: Vec<String>,
    module: Option<Module>,
    instance: Option<Instance>,
    exports: Arc<Mutex<RefCell<Option<Exports>>>>,
}

#[wasm_bindgen]
impl Blockless {
    #[wasm_bindgen(constructor)]
    pub fn new(config: BlocklessConfig) -> Result<Blockless, JsValue> {
        init_panic_hook();

        let args: Vec<String> = {
            let args = js_sys::Reflect::get(&config, &"args".into())?;
            if args.is_undefined() {
                vec![]
            } else {
                let args_array: js_sys::Array = args.dyn_into()?;
                args_array
                    .iter()
                    .map(|arg| {
                        arg.as_string()
                            .ok_or(js_sys::Error::new("All arguments must be strings").into())
                    })
                    .collect::<Result<Vec<String>, JsValue>>()?
            }
        };
        let env: Vec<(String, String)> = {
            let env = js_sys::Reflect::get(&config, &"env".into())?;
            if env.is_undefined() {
                vec![]
            } else {
                let env_obj: js_sys::Object = env.dyn_into()?;
                js_sys::Object::entries(&env_obj)
                    .iter()
                    .map(|entry| {
                        let entry: js_sys::Array = entry.unchecked_into();
                        let key: Result<String, JsValue> = entry.get(0).as_string().ok_or(
                            js_sys::Error::new("All environment keys must be strings").into(),
                        );
                        let value: Result<String, JsValue> = entry.get(1).as_string().ok_or(
                            js_sys::Error::new("All environment values must be strings").into(),
                        );
                        key.and_then(|key| Ok((key, value?)))
                    })
                    .collect::<Result<Vec<(String, String)>, JsValue>>()?
            }
        };
        let preopens: Vec<(String, String)> = {
            let preopens = js_sys::Reflect::get(&config, &"preopens".into())?;
            if preopens.is_undefined() {
                vec![(".".to_string(), "/".to_string())]
            } else {
                let preopens_obj: js_sys::Object = preopens.dyn_into()?;
                js_sys::Object::entries(&preopens_obj)
                    .iter()
                    .map(|entry| {
                        let entry: js_sys::Array = entry.unchecked_into();
                        let key: Result<String, JsValue> = entry.get(0).as_string().ok_or(
                            js_sys::Error::new("All preopen keys must be strings").into(),
                        );
                        let value: Result<String, JsValue> = entry.get(1).as_string().ok_or(
                            js_sys::Error::new("All preopen values must be strings").into(),
                        );
                        key.and_then(|key| Ok((key, value?)))
                    })
                    .collect::<Result<Vec<(String, String)>, JsValue>>()?
            }
        };
        let permissions = {
            let permissions = js_sys::Reflect::get(&config, &"permissions".into())?;
            if permissions.is_undefined() {
                vec![]
            } else {
                let permissions_array: js_sys::Array = permissions.dyn_into()?;
                permissions_array
                    .iter()
                    .map(|permission| {
                        permission
                            .as_string()
                            .ok_or(js_sys::Error::new("All permissions must be strings").into())
                    })
                    .collect::<Result<Vec<String>, JsValue>>()?
            }
        };

        let fs = {
            let fs = js_sys::Reflect::get(&config, &"fs".into())?;
            if fs.is_undefined() {
                fs::MemFS::new()?
            } else {
                fs::MemFS::from_js(fs)?
            }
        };

        let mut store = Store::default();
        let stdout = Pipe::default();
        let stdin = Pipe::default();
        let stderr = Pipe::default();
        let wasi_env = WasiState::new(args.get(0).unwrap_or(&"".to_string()))
            .args(if !args.is_empty() { &args[1..] } else { &[] })
            .envs(env)
            .set_fs(Box::new(fs))
            .stdout(Box::new(stdout.clone()))
            .stdin(Box::new(stdin.clone()))
            .stderr(Box::new(stderr.clone()))
            .map_dirs(preopens)
            .map_err(|e| js_sys::Error::new(&format!("Couldn't preopen the dir: {}`", e)))?
            // .map_dirs(vec![(".".to_string(), "/".to_string())])
            // .preopen_dir("/").map_err(|e| js_sys::Error::new(&format!("Couldn't preopen the dir: {}`", e)))?
            .finalize(&mut store)
            .map_err(|e| js_sys::Error::new(&format!("Failed to create the WasiEnv: {}`", e)))?;

        Ok(Blockless{
            store,
            stdout,
            stdin,
            stderr,
            wasi_env,
            permissions,
            module: None,
            instance: None,
            exports: Arc::new(Mutex::new(RefCell::new(None))),
        })
    }

    #[wasm_bindgen(getter)]
    pub fn fs(&mut self) -> Result<fs::MemFS, JsValue> {
        let state = self.wasi_env.data_mut(&mut self.store).state();
        let mem_fs = state
            .fs
            .fs_backing
            .downcast_ref::<fs::MemFS>()
            .ok_or_else(|| js_sys::Error::new("Failed to downcast to MemFS"))?;
        Ok(mem_fs.clone())
    }

    #[wasm_bindgen(js_name = getImports)]
    pub fn get_imports(
        &mut self,
        module: js_sys::WebAssembly::Module,
    ) -> Result<js_sys::Object, JsValue> {
        let module: js_sys::WebAssembly::Module = module.dyn_into().map_err(|_e| {
            js_sys::Error::new(
                "You must provide a module to the WASI new. `let module = new WASI({}, module);`",
            )
        })?;
        let module: Module = module.into();
        let mut import_object = self.get_wasi_imports(&module)?;
        import_object.extend(&self.get_host_imports()?);
        // let mut import_object = &self.get_host_imports()?;

        self.module = Some(module);

        // let js_value = import_object.as_jsvalue(&self.store);
        // Ok(js_value.into())
        Ok(import_object.as_jsobject(&self.store))
    }

    fn get_wasi_imports(&mut self, module: &Module) -> Result<Imports, JsValue> {
        let import_object = self
            .wasi_env
            .import_object(&mut self.store, module)
            .map_err(|e| {
                js_sys::Error::new(&format!("Failed to create the Import Object: {}`", e))
            })?;
        Ok(import_object)
    }

    fn get_host_imports(&mut self) -> Result<Imports, JsValue> {
        #[derive(Clone)]
        struct Env {
            exports: Arc<Mutex<RefCell<Option<Exports>>>>,
            permissions: Vec<String>,
        }
        let env = FunctionEnv::new(&mut self.store, Env {
            exports: self.exports.clone(),
            permissions: self.permissions.clone(),
        });

        fn host_log(ctx: FunctionEnvMut<Env>, ptr: u32, len: u32) {
            let exports = {
                let binding = ctx.data().exports.lock().unwrap();
                let exports = binding.borrow().to_owned().expect("exports should have been set");
                exports
            };
            let memory = exports.get_memory("memory").expect("memory export wasn't found");
            let mut buf = vec![0u8; len as usize];
            memory.view(&ctx.as_store_ref()).read(ptr.into(), &mut buf).expect("failed to read memory");
            let buf_str = std::str::from_utf8(&buf).unwrap();
            console_log!("[log]: {}", buf_str);
        }

        fn host_call(ctx: FunctionEnvMut<Env>, ptr: u32, len: u32) -> u32 {
            let exports = {
                let binding = ctx.data().exports.lock().unwrap();
                let exports = binding.borrow().to_owned().expect("exports should have been set");
                exports
            };
            let memory = exports.get_memory("memory").expect("memory export wasn't found");

            let mut buf = vec![0u8; len as usize];
            memory.view(&ctx.as_store_ref()).read(ptr as u64, &mut buf).expect("failed to read memory");

            // required to write data back to guest
            // TODO: find another way to do this without manually allocating memory?
            let alloc_func = exports.get_function("alloc").expect("alloc function not found");

            // let buf_str = std::str::from_utf8(&buf).unwrap();
            // console_log!("host_call successfully read data: {:?}", buf_str);

            let call = serde_json::from_slice::<ModuleCall>(&buf).unwrap();
            call.validate_permissions(); // TODO: pass in config?
            match call {
                ModuleCall::Http(http_req) => {
                    console_log!("host_call: http_request called: {}", http_req); // TODO trace
                    let blockless_callback = exports.get_function("blockless_callback").expect("blockless_callback function not found");

                    if !http_req.valid_permissions(&ctx.data().permissions) {
                        console_error!("invalid permissions");
                        let data = serde_json::to_vec(&ModuleCallResponse::Http(Err("invalid permissions".into())))
                            .expect("failed to serialize module call response");
                        // allocate memory for size of result and return back pointer to the allocated memory
                        // first 4 bytes are the length of the result
                        memory.view(&ctx.as_store_ref()).write(ptr as u64, &(data.len() as u32).to_le_bytes()).expect("failed to write data length to memory");
                        // next bytes are the actual result
                        memory.view(&ctx.as_store_ref()).write((ptr + 4) as u64, &data).expect("failed to write data to memory");
                        return ptr as u32;
                    }

                    let boxed_ctx_ref: Box<FunctionEnvMut<Env>> = Box::new(ctx);
                    let static_ctx_ref: &'static mut FunctionEnvMut<Env> = unsafe { std::mem::transmute(Box::leak(boxed_ctx_ref)) };

                    wasm_bindgen_futures::spawn_local(async move {
                        let memory = exports.get_memory("memory").expect("memory export wasn't found");

                        // NOTE: convert callbacks to wasm_bindgen types - since return values do not seem to work!
                        let memory_obj: WebAssembly::Memory = exports.
                            get_extern("memory")
                            .expect("memory export wasn't found")
                            .to_vm_extern()
                            .as_jsvalue(&static_ctx_ref.as_store_ref())
                            .clone()
                            .into();
                        let blockless_callback: js_sys::Function = exports
                            .get_function("blockless_callback")
                            .expect("blockless_callback function not found")
                            .to_vm_extern()
                            .as_jsvalue(&static_ctx_ref.as_store_ref())
                            .clone()
                            .into();
                        let alloc_func: js_sys::Function = exports
                            .get_function("alloc")
                            .expect("alloc function not found")
                            .to_vm_extern()
                            .as_jsvalue(&static_ctx_ref.as_store_ref())
                            .clone()
                            .into();

                        // NOTE: convert callbacks to wasm_bindgen types - since return values do not seem to work!
                        // let memory = exports.get_memory("memory").expect("memory export wasn't found");
                        // let blockless_callback = exports.get_function("blockless_callback").expect("blockless_callback function not found");
                        // let blockless_callback_typed: TypedFunction<i32, i32> = blockless_callback.typed(&static_ctx_ref.as_store_ref()).expect("failed to get typed blockless_callback");

                        let module_call_response = match http_req.request().await {
                            Ok(response) => {
                                match HttpResponse::from_reqwest(response).await {
                                    Ok(res) => ModuleCallResponse::Http(Ok(res)),
                                    Err(_) => ModuleCallResponse::Http(Err("failed to parse response".into())),
                                }
                            },
                            Err(err) => {
                                console_error!("Error while running start function: {}", err);
                                ModuleCallResponse::Http(Err(err.to_string()))
                            }
                        };
                        let data = serde_json::to_vec(&module_call_response).expect("failed to serialize module call response");
                        let result_ptr = utils::encode_data_to_memory(&memory_obj, &alloc_func, &data);

                        // memory.view(&static_ctx_ref.as_store_ref()).write(ptr as u64, &(data.len() as u32).to_le_bytes()).expect("failed to write data length to memory");
                        // memory.view(&static_ctx_ref.as_store_ref()).write((ptr + 4) as u64, &data).expect("failed to write data to memory");

                        // match blockless_callback.call(&mut static_ctx_ref, &[Value::I32(ptr as i32)]) {
                        // match blockless_callback_typed.call(&mut static_ctx_ref, ptr as i32) {
                        match blockless_callback.call1(&JsValue::undefined(), &JsValue::from(result_ptr)) {
                            Ok(_val) => console_log!("blockless_callback called successfully"),
                            Err(err) => console_error!("Error while running blockless_callback {}", err.as_string().unwrap_or_default()),
                        };

                        // manually deallocate memory
                        unsafe {
                            let _reclaimed = Box::from_raw(static_ctx_ref);
                        }
                    });
                },
                ModuleCall::Ipfs(ipfs_get) => {
                    console_log!("host_call: ipfs_get called: {}", ipfs_get);
                    // TODO: validate guest has exported function to callback into
                    // TODO: perform the request in spawn_local
                    // TODO: callback into guest with the result (in spawn_local)
                },
            };
            0
        }

        fn http_call(ctx: FunctionEnvMut<Env>, ptr: u32, len: u32, callback_id: u64) -> u32 {
            let exports = {
                let binding = ctx.data().exports.lock().unwrap();
                let exports = binding.borrow().to_owned().expect("exports should have been set");
                exports
            };
            let memory = exports.get_memory("memory").expect("memory export wasn't found");

            let mut buf = vec![0u8; len as usize];
            memory.view(&ctx.as_store_ref()).read(ptr as u64, &mut buf).expect("failed to read memory");

            // required to write data back to guest
            // TODO: find another way to do this without manually allocating memory?
            let alloc_func = exports.get_function("alloc").expect("alloc function not found");
            let http_callback = exports.get_function("http_callback").expect("http_callback function not found");

            let buf_str = std::str::from_utf8(&buf).unwrap();
            console_log!("http_request successfully read data: {:?}", buf_str);

            let http_req = serde_json::from_slice::<HttpRequest>(&buf).expect("failed to deserialize http request");
            console_log!("host_call: http_request called: {}", http_req); // TODO trace

            if !http_req.valid_permissions(&ctx.data().permissions) {
                console_error!("invalid permissions");
                let data = serde_json::to_vec(&Err::<HttpResponse, String>("invalid permissions".into()))
                    .expect("failed to serialize module call response");
                // allocate memory for size of result and return back pointer to the allocated memory
                // first 4 bytes are the length of the result
                memory.view(&ctx.as_store_ref()).write(ptr as u64, &(data.len() as u32).to_le_bytes()).expect("failed to write data length to memory");
                // next bytes are the actual result
                memory.view(&ctx.as_store_ref()).write((ptr + 4) as u64, &data).expect("failed to write data to memory");
                return ptr as u32;
            }

            let boxed_ctx_ref: Box<FunctionEnvMut<Env>> = Box::new(ctx);
            let static_ctx_ref: &'static mut FunctionEnvMut<Env> = unsafe { std::mem::transmute(Box::leak(boxed_ctx_ref)) };
            wasm_bindgen_futures::spawn_local(async move {
                let memory = exports.get_memory("memory").expect("memory export wasn't found");

                // NOTE: convert callbacks to wasm_bindgen types - since return values do not seem to work!
                let memory_obj: WebAssembly::Memory = exports
                    .get_extern("memory")
                    .expect("memory export wasn't found")
                    .to_vm_extern()
                    .as_jsvalue(&static_ctx_ref.as_store_ref())
                    .clone()
                    .into();
                let http_callback: js_sys::Function = exports
                    .get_function("http_callback")
                    .expect("http_callback function not found")
                    .to_vm_extern()
                    .as_jsvalue(&static_ctx_ref.as_store_ref())
                    .clone()
                    .into();
                let alloc_func: js_sys::Function = exports
                    .get_function("alloc")
                    .expect("alloc function not found")
                    .to_vm_extern()
                    .as_jsvalue(&static_ctx_ref.as_store_ref())
                    .clone()
                    .into();

                let http_call_response = match http_req.request().await {
                    Ok(response) => HttpResponse::from_reqwest(response).await,
                    Err(err) => {
                        console_error!("Error while running start function: {}", err);
                        Err(err.to_string())
                    }
                };
                console_log!("host_call: http_call_response: {:?}", http_call_response);
                let data = serde_json::to_vec(&http_call_response).expect("failed to serialize module call response");
                let result_ptr = utils::encode_data_to_memory(&memory_obj, &alloc_func, &data);

                match http_callback.call2(&JsValue::undefined(), &JsValue::from(result_ptr), &JsValue::from(callback_id)) {
                    Ok(_val) => console_log!("http_callback called successfully"),
                    Err(err) => console_error!("Error while running http_callback {}", err.as_string().unwrap_or_default()),
                };

                // manually deallocate memory
                unsafe {
                    let _reclaimed = Box::from_raw(static_ctx_ref);
                }
            });
            0
        }

        let imports = imports! {
            "blockless" => {
                "host_log" => Function::new_typed_with_env(&mut self.store, &env, host_log),
                "host_call" => Function::new_typed_with_env(&mut self.store, &env, host_call),
                "http_call" => Function::new_typed_with_env(&mut self.store, &env, http_call),
            },
        };

        // using exports approach - may be another function to use
        // let mut exports = Exports::new();
        // exports.insert("add", Function::new_typed(&mut self.store, func));
        // imports.register_namespace("blockless", exports);
        Ok(imports)
    }

    pub fn instantiate(
        &mut self,
        module_or_instance: JsValue,
        imports: Option<js_sys::Object>,
    ) -> Result<js_sys::WebAssembly::Instance, JsValue> {
        let instance = if module_or_instance.has_type::<js_sys::WebAssembly::Module>() {
            let js_module: js_sys::WebAssembly::Module = module_or_instance.unchecked_into();
            let module: Module = js_module.into();

            // inject wasi + host + guest imports
            let mut runtime_imports = self.get_wasi_imports(&module)?;
            runtime_imports.extend(&self.get_host_imports()?);
            if let Some(base_imports) = imports {
                let imports = Imports::new_from_js_object(&mut self.store, &module, base_imports).map_err(
                // let imports = Imports::from_jsvalue(&mut self.store, &module, &base_imports.into()).map_err(
                    // |e| js_sys::Error::new(&format!("Failed to get runtime imports: {}", e.into())),
                    |_e| js_sys::Error::new("Failed to get runtime imports"),
                )?;
                runtime_imports.extend(&imports);
            };

            let instance = Instance::new(&mut self.store, &module, &runtime_imports)
                .map_err(|e| js_sys::Error::new(&format!("Failed to instantiate WASI: {}`", e)))?;
            self.module = Some(module);
            instance
        } else if module_or_instance.has_type::<js_sys::WebAssembly::Instance>() {
            if let Some(instance) = &self.instance {
                // We completely skip the set instance step
                return Ok(instance.raw(&self.store).clone());
                // return Ok(instance.as_jsvalue(&self.store).into());
            }
            let module = self.module.as_ref().ok_or(js_sys::Error::new("When providing an instance, the `wasi.getImports` must be called with the module first"))?;
            let js_instance: js_sys::WebAssembly::Instance = module_or_instance.unchecked_into();
            Instance::from_module_and_instance(&mut self.store, module, js_instance).map_err(
            // Instance::from_jsvalue(&mut self.store, &module, &js_instance.into()).map_err(
                // |e| js_sys::Error::new(&format!("Can't get the Wasmer Instance: {:?}", e.into())),
                |_e| js_sys::Error::new("Can't get the Wasmer Instance"),
            )?
        } else {
            return Err(
                js_sys::Error::new("You need to provide a `WebAssembly.Module` or `WebAssembly.Instance` as first argument to `wasi.instantiate`").into(),
            );
        };

        // self.wasi_env
        //     .initialize(&mut self.store, instance.clone())
        //     .map_err(|e| js_sys::Error::new(&format!("Failed to initialize WASI: {}`", e)))?;
        self.wasi_env
            .data_mut(&mut self.store)
            .set_memory(instance.exports.get_memory("memory").unwrap().clone());

        // let raw_instance: WebAssembly::Instance = instance.as_jsvalue(&self.store).into();
        let raw_instance = instance.raw(&self.store).clone();

        // self.config.lock().unwrap().borrow_mut().instance = Some(raw_instance.clone()); // TODO: address this

        self.instance = Some(instance);

        // TODO: is there a better approach?
        self.exports.lock().unwrap().borrow_mut().replace(self.instance.as_ref().unwrap().exports.clone());

        Ok(raw_instance)
    }

    /// Start the WASI Instance, it returns the status code when calling the start
    /// function
    pub fn start(
        &mut self,
        instance: Option<js_sys::WebAssembly::Instance>,
    ) -> Result<u32, JsValue> {
        if let Some(instance) = instance {
            self.instantiate(instance.into(), None)?;
        } else if self.instance.is_none() {
            return Err(
                js_sys::Error::new("You need to provide an instance as argument to `start`, or call `wasi.instantiate` with the `WebAssembly.Instance` manually").into(),
            );
        }
        let start = self
            .instance
            .as_ref()
            .unwrap()
            .exports
            .get_function("_start")
            .map_err(|_e| js_sys::Error::new("The _start function is not present"))?;
        let result = start.call(&mut self.store, &[]);

        match result {
            Ok(_) => Ok(0),
            Err(err) => {
                match err.downcast::<WasiError>() {
                    Ok(WasiError::Exit(exit_code)) => {
                        // We should exit with the provided exit code
                        Ok(exit_code)
                    }
                    Ok(err) => {
                        return Err(js_sys::Error::new(&format!(
                            "Unexpected WASI error while running start function: {}",
                            err
                        ))
                        .into())
                    }
                    Err(err) => {
                        return Err(js_sys::Error::new(&format!(
                            "Error while running start function: {}",
                            err
                        ))
                        .into())
                    }
                }
            }
        }
    }

    // Stdio methods below

    /// Get the stdout buffer
    /// Note: this method flushes the stdout
    #[wasm_bindgen(js_name = getStdoutBuffer)]
    pub fn get_stdout_buffer(&mut self) -> Result<Vec<u8>, JsValue> {
        let mut buf = Vec::new();
        self.stdout
            .read_to_end(&mut buf)
            .map_err(|e| js_sys::Error::new(&format!("Could not get the stdout bytes: {}`", e)))?;
        Ok(buf)
    }

    /// Get the stdout data as a string
    /// Note: this method flushes the stdout
    #[wasm_bindgen(js_name = getStdoutString)]
    pub fn get_stdout_string(&mut self) -> Result<String, JsValue> {
        let mut stdout_str = String::new();
        self.stdout.read_to_string(&mut stdout_str).map_err(|e| {
            js_sys::Error::new(&format!(
                "Could not convert the stdout bytes to a String: {}`",
                e
            ))
        })?;
        Ok(stdout_str)
    }

    /// Get the stderr buffer
    /// Note: this method flushes the stderr
    #[wasm_bindgen(js_name = getStderrBuffer)]
    pub fn get_stderr_buffer(&mut self) -> Result<Vec<u8>, JsValue> {
        let mut buf = Vec::new();
        self.stderr
            .read_to_end(&mut buf)
            .map_err(|e| js_sys::Error::new(&format!("Could not get the stderr bytes: {}`", e)))?;
        Ok(buf)
    }

    /// Get the stderr data as a string
    /// Note: this method flushes the stderr
    #[wasm_bindgen(js_name = getStderrString)]
    pub fn get_stderr_string(&mut self) -> Result<String, JsValue> {
        let mut stderr_str = String::new();
        self.stderr.read_to_string(&mut stderr_str).map_err(|e| {
            js_sys::Error::new(&format!(
                "Could not convert the stderr bytes to a String: {}`",
                e
            ))
        })?;
        Ok(stderr_str)
    }

    /// Set the stdin buffer
    #[wasm_bindgen(js_name = setStdinBuffer)]
    pub fn set_stdin_buffer(&mut self, buf: &[u8]) -> Result<(), JsValue> {
        self.stdin
            .write_all(buf)
            .map_err(|e| js_sys::Error::new(&format!("Error writing stdin: {}`", e)))?;
        Ok(())
    }

    /// Set the stdin data as a string
    #[wasm_bindgen(js_name = setStdinString)]
    pub fn set_stdin_string(&mut self, input: String) -> Result<(), JsValue> {
        self.set_stdin_buffer(input.as_bytes())
    }
}
