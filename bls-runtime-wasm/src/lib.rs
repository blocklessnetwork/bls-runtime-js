use js_sys::{Function, Map, Object, Reflect, WebAssembly};
use wasm_bindgen::{prelude::{wasm_bindgen, JsCast, JsValue}, JsError};
use wasm_bindgen_futures::{JsFuture, spawn_local};
use web_sys::{Request, RequestInit, RequestMode, Response, console};

pub mod utils;

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

// #[wasm_bindgen]
// pub fn host_call(data: &[u8]) {
//     let string = std::str::from_utf8(data).unwrap();
//     console_log!("Host Log: {}", string);
// }

// #[wasm_bindgen]
// pub async fn run_wasm(wasm_module: &[u8]) -> Result<(), JsValue> {
//     init_panic_hook();
//     console_log!("instantiating a new wasm module directly");

//     let imports_obj = {
//         let map = Map::new();
//         let imports: JsValue = Imports.into();

//         bind(&imports, "host_log")?;
//         bind(&imports, "host_call")?;

//         map.set(&JsValue::from("blockless"), &imports);
//         Object::from_entries(&map.into())?
//     };

//     let buffer = JsFuture::from(WebAssembly::instantiate_buffer(wasm_module, &imports_obj)).await?;
//     let instance: WebAssembly::Instance = Reflect::get(&buffer, &"instance".into())?.dyn_into()?;
//     let exports = instance.exports();

//     // TODO: FIX THIS - storing the instance globally
//     unsafe {
//         GUEST_INSTANCE = Some(instance);
//     }

//     let start_func = Reflect::get(&exports, &"start".into())?
//         .dyn_into::<Function>()
//         .map_err(|_| "not a function")?;

//     start_func.call0(&JsValue::undefined())?;

//     // console_log!("created module has {} pages of memory", mem.grow(0));
//     // console_log!("giving the module 4 more pages of memory");
//     // mem.grow(4);
//     // console_log!("now the module has {} pages of memory", mem.grow(0));

//     Ok(())
// }

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

// {"page":1,"per_page":6,"total":12,"total_pages":2,"data":[{"id":1,"name":"cerulean","year":2000,"color":"#98B2D1","pantone_value":"15-4020"},{"id":2,"name":"fuchsia rose","year":2001,"color":"#C74375","pantone_value":"17-2031"},{"id":3,"name":"true red","year":2002,"color":"#BF1932","pantone_value":"19-1664"},{"id":4,"name":"aqua sky","year":2003,"color":"#7BC4C4","pantone_value":"14-4811"},{"id":5,"name":"tigerlily","year":2004,"color":"#E2583E","pantone_value":"17-1456"},{"id":6,"name":"blue turquoise","year":2005,"color":"#53B0AE","pantone_value":"15-5217"}],"support":{"url":"https://reqres.in/#support-heading","text":"To keep ReqRes free, contributions towards server costs are appreciated!"}}

#[wasm_bindgen]
pub async fn run_fetch(url: i32) -> Result<JsValue, JsValue> {
    let mut opts = RequestInit::new();
    opts.method("GET");
    opts.mode(RequestMode::Cors);

    let url = format!("https://reqres.in/api/products");

    let request = Request::new_with_str_and_init(&url, &opts)?;

    request
        .headers()
        .set("Accept", "application/vnd.github.v3+json")?;

    let window = web_sys::window().unwrap();
    let resp_value = JsFuture::from(window.fetch_with_request(&request)).await?;

    // `resp_value` is a `Response` object.
    assert!(resp_value.is_instance_of::<Response>());
    let resp: Response = resp_value.dyn_into().unwrap();

    // Convert this other `Promise` into a rust `Future`.
    let json = JsFuture::from(resp.json()?).await?;

    // // Use serde to parse the JSON into a struct.
    // let products: Products = json.into_serde().unwrap();

    // // Send the `Branch` struct back to JS as an `Object`.
    // Ok(JsValue::from_serde(&products).unwrap())

    Ok(json)
}

use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;


// #[wasm_bindgen]
// pub async fn run_reqwest(url: &str) -> Result<JsValue, JsValue> {
pub async fn run_reqwest(url: &str) -> Result<Vec<u8>, &'static str> {
    let res = reqwest::Client::new()
        // .get("https://reqres.in/api/products")
        .get(url)
        .header("Accept", "application/vnd.github.v3+json")
        .send()
        .await
        .map_err(|_| "failed to send request")?;

    let bytes = res.bytes().await.map_err(|_| "failed to read response bytes")?;
    Ok(bytes.to_vec())

    // let products: bls_common::Products = serde_json::from_slice(&bytes).unwrap();
    // Ok(serde_wasm_bindgen::to_value(&products).unwrap())

    // let res_str = serde_json::to_string(&products).unwrap();
    // console_log!("run_reqwest: {}", res_str);

    // let result_bytes = serde_json::to_vec(&products).unwrap();
    // Ok(result_bytes)
}


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

use std::sync::{Arc, Mutex};
use std::cell::RefCell;

#[derive(Debug, Clone)]
pub struct Config {
    // module: Option<Module>,
    args: Vec<String>,
    env: Vec<(String, String)>,
    preopens: Vec<(String, String)>,
    permissions: Vec<String>,
    // fs: BrowserFS,
    instance: Option<WebAssembly::Instance>,
}

#[derive(Debug, Clone)]
#[wasm_bindgen]
pub struct Blockless(Arc<RefCell<Config>>);

#[wasm_bindgen]
impl Blockless {
    #[wasm_bindgen(constructor)]
    pub fn new(config: BlocklessConfig) -> Result<Blockless, JsValue> {
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

        let config = Config {
            args,
            env,
            preopens,
            permissions,
            // fs,
            instance: None,
        };
        Ok(Blockless(Arc::new(RefCell::new(config))))
    }

    pub fn instantiate(
        &mut self,
        module_or_instance: JsValue,
        imports: Option<js_sys::Object>,
    ) -> Result<(), JsValue> {
        let raw_instance = if module_or_instance.has_type::<js_sys::WebAssembly::Module>() {
            let js_module: js_sys::WebAssembly::Module = module_or_instance.unchecked_into();
            let host_exports = self.host_exports()?;
            let guest_imports = imports.unwrap_or_else(|| js_sys::Object::new());
            let combined_imports = js_sys::Object::assign(&host_exports, &guest_imports);
            let instance: WebAssembly::Instance = WebAssembly::Instance::new(&js_module, &combined_imports)?;
            instance
        } else if module_or_instance.has_type::<js_sys::WebAssembly::Instance>() {
            let js_instance: js_sys::WebAssembly::Instance = module_or_instance.unchecked_into();
            js_instance
        } else {
            return Err(
                js_sys::Error::new("You need to provide a `WebAssembly.Module` or `WebAssembly.Instance` as first argument to `wasi.instantiate`").into(),
            );
        };

        self.0.borrow_mut().instance = Some(raw_instance);

        Ok(())
    }

    /// Start the WASI Instance, it returns the status code when calling the start function
    pub fn start(
        &mut self,
    ) -> Result<u32, JsValue> {
        let Some(instance) = &self.0.borrow().instance else {
            return Err(
                js_sys::Error::new("You need to provide an instance as argument to `start`, or call `wasi.instantiate` with the `WebAssembly.Instance` manually").into(),
            );
        };

        let start_func = Reflect::get(&instance.exports(), &"_start".into())?
            .dyn_into::<Function>()
            .map_err(|_| "The _start function is not present")?;

        let result = start_func.call0(&JsValue::undefined());
        match result {
            Ok(_) => return Ok(0),
            Err(err) => {
                let err_text = err.as_string().unwrap_or_else(|| "Unknown error".to_string());
                return Err(js_sys::Error::new(&format!("Error while running start function: {}", err_text)).into());
            }
        };
    }

    #[wasm_bindgen(js_name = hostExports)]
    pub fn host_exports(&self) -> Result<js_sys::Object, JsValue> {
        let blockless_exports: JsValue = Object::new().into();

        let instance_arc = self.0.clone();
        let log_fn = Closure::wrap(Box::new(move |ptr: u32, len: u32| {
            let Some(ref instance) = (*instance_arc.borrow()).instance else {
                console_error!("Guest instance should have been set");
                return;
            };
            host_log(instance, ptr, len)
        }) as Box<dyn FnMut(u32, u32)>);

        let instance_arc = self.0.clone();
        let host_call_fn = Closure::wrap(Box::new(move |ptr: u32, len: u32| {
            let Some(ref _instance) = (*instance_arc.borrow()).instance else {
                console_error!("Guest instance should have been set");
                return 0;
            };

            host_call(instance_arc.clone(), ptr, len)
        }) as Box<dyn FnMut(u32, u32) -> u32>);

        js_sys::Reflect::set(&blockless_exports, &JsValue::from("host_log"), log_fn.as_ref().unchecked_ref())?;
        log_fn.forget();

        js_sys::Reflect::set(&blockless_exports, &JsValue::from("host_call"), host_call_fn.as_ref().unchecked_ref())?;
        host_call_fn.forget();
        
        let map = Map::new();
        map.set(&JsValue::from("blockless"), &blockless_exports);
        Ok(Object::from_entries(&map.into())?)
    }
}

use bls_common::{types::{ModuleCall, ModuleCallResponse}, http::HttpResponse};

pub fn host_log(instance: &WebAssembly::Instance, ptr: u32, len: u32) {
    let data = utils::decode_data_from_memory(&instance, ptr, len);
    let msg = std::str::from_utf8(&data).unwrap();

    console_log!("host_log: {}", msg);
}

pub fn host_call(config: Arc<RefCell<Config>>, ptr: u32, len: u32) -> u32 { // Allocate space in the guest's memory to store the return string
    // get the instance again - using interior mutability
    let cfg = config.borrow();
    let instance = cfg.instance.as_ref().expect("Guest instance should have been set");
    let permissions = &cfg.permissions;

    let call_data = utils::decode_data_from_memory(&instance, ptr, len);
    // console_log!("host_call: {}", std::str::from_utf8(&call_data).unwrap());

    // deserialize this to a canonical format
    let call = serde_json::from_slice::<ModuleCall>(&call_data).unwrap();
    call.validate_permissions(); // TODO: pass in config
    match call {
        ModuleCall::Http(http_req) => {
            console_log!("host_call: http_request called: {}", http_req);

            let blockless_callback = Reflect::get(&instance.exports(), &"blockless_callback".into())
                .expect("callback export wasn't found")
                .dyn_into::<Function>()
                .expect("The blockless_callback function is not present");

            if !http_req.valid_permissions(permissions) {
                let result_bytes = serde_json::to_vec(&ModuleCallResponse::Http(Err("invalid permissions".into())))
                    .expect("failed to serialize module call response");
                let result_ptr = utils::encode_data_to_memory(instance, &result_bytes);
                return result_ptr;
            }

            let cfg_ptr = config.clone();
            wasm_bindgen_futures::spawn_local(async move {
                // get the instance again - using interior mutability
                let cfg = cfg_ptr.borrow();
                let instance = cfg.instance.as_ref().expect("Guest instance should have been set");

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
                let result_bytes = serde_json::to_vec(&module_call_response).expect("failed to serialize module call response");
                let result_ptr = utils::encode_data_to_memory(instance, &result_bytes);

                // TODO: fix error being thrown on callback
                match blockless_callback.call1(&JsValue::undefined(), &JsValue::from(result_ptr)) {
                    Ok(_) => console_log!("blockless_callback called successfully"),
                    Err(err) => console_error!("Error while running blockless_callback {}", err.as_string().unwrap_or_default()),
                };
            });
        },
        ModuleCall::Ipfs(ipfs_get) => {
            console_log!("host_call: ipfs_get called: {}", ipfs_get);
            // TODO: validate guest has exported function to callback into
            // TODO: perform the request in spawn_local
            // TODO: callback into guest with the result (in spawn_local)
        },
    };

    // TODO: serialize a canonical response format to end back to client
    let return_str = "<hello world from host> <hello world from host> <hello world from host> <hello world from host> <hello world from host> <hello world from host> <hello world from host> <hello world from host> <hello world from host> <hello world from host> <hello world from host> <hello world from host> <hello world from host> <hello world from host> <hello world from host>";
    let return_bytes = return_str.as_bytes();

    let result_ptr = utils::encode_data_to_memory(&instance, return_bytes);
    result_ptr
}
