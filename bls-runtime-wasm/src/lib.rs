use js_sys::{Function, Map, Object, Reflect, WebAssembly};
use wasm_bindgen::prelude::{wasm_bindgen, JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{Request, RequestInit, RequestMode, Response};

mod utils;

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

#[wasm_bindgen]
pub struct Imports;

// TODO: fix this (dont use unsafe global hacks)
static mut GUEST_INSTANCE: Option<WebAssembly::Instance> = None;

#[wasm_bindgen]
impl Imports {
    pub fn host_log(&self, ptr: u32, len: u32) {
        let instance = unsafe {
            crate::GUEST_INSTANCE
                .as_ref()
                .expect("Guest instance should have been initialized")
        };

        let data = utils::decode_data_from_memory(instance, ptr, len);
        let msg = std::str::from_utf8(&data).unwrap();

        console_log!("host_log: {}", msg);
    }
    pub fn host_call(&self, ptr: u32, len: u32) -> u32 {// Allocate space in the guest's memory to store the return string
        let instance = unsafe {
            crate::GUEST_INSTANCE
                .as_ref()
                .expect("Guest instance should have been initialized")
        };
    
        let data = utils::decode_data_from_memory(instance, ptr, len);
        let msg = std::str::from_utf8(&data).unwrap();
        console_log!("host_call: {}", msg);

        // TODO: do something to get a result

        let return_str = "<hello world from host>";
        let return_bytes = return_str.as_bytes();
        let result_ptr = utils::encode_data_to_memory(instance, return_bytes);
        result_ptr
    }
}

fn bind(this: &JsValue, func_name: &str) -> Result<(), JsValue> {
    let property_key = JsValue::from(func_name);
    let orig_func = Reflect::get(this, &property_key)?.dyn_into::<Function>()?;
    let func = orig_func.bind(this);
    if !Reflect::set(this, &property_key, &func)? {
        return Err(JsValue::from("failed to set property"));
    }
    Ok(())
}

#[wasm_bindgen]
pub async fn run_wasm(wasm_module: &[u8]) -> Result<(), JsValue> {
    console_log!("instantiating a new wasm module directly");

    let imports_obj = {
        let map = Map::new();
        let imports: JsValue = Imports.into();

        bind(&imports, "host_log")?;
        bind(&imports, "host_call")?;

        map.set(&JsValue::from("blockless"), &imports);
        Object::from_entries(&map.into())?
    };

    let buffer = JsFuture::from(WebAssembly::instantiate_buffer(wasm_module, &imports_obj)).await?;
    let instance: WebAssembly::Instance = Reflect::get(&buffer, &"instance".into())?.dyn_into()?;
    let exports = instance.exports();

    // TODO: FIX THIS - storing the instance globally
    unsafe {
        GUEST_INSTANCE = Some(instance);
    }

    let start_func = Reflect::get(&exports, &"start".into())?
        .dyn_into::<Function>()
        .map_err(|_| "not a function")?;

    start_func.call0(&JsValue::undefined())?;

    // console_log!("created module has {} pages of memory", mem.grow(0));
    // console_log!("giving the module 4 more pages of memory");
    // mem.grow(4);
    // console_log!("now the module has {} pages of memory", mem.grow(0));

    Ok(())
}

const WASM: &[u8] = include_bytes!("../../target/wasm32-unknown-unknown/release/rust_sdk.wasm");
// const WASM: &[u8] = include_bytes!("../../simple.wasm");

#[wasm_bindgen(start)]
fn start() {
    wasm_bindgen_futures::spawn_local(async {
        match run_wasm(WASM).await {
            Ok(_) => console_log!("successfully finished running wasm module"),
            Err(e) => console_error!("{:?}", e),
        }
    });
}

// {"page":1,"per_page":6,"total":12,"total_pages":2,"data":[{"id":1,"name":"cerulean","year":2000,"color":"#98B2D1","pantone_value":"15-4020"},{"id":2,"name":"fuchsia rose","year":2001,"color":"#C74375","pantone_value":"17-2031"},{"id":3,"name":"true red","year":2002,"color":"#BF1932","pantone_value":"19-1664"},{"id":4,"name":"aqua sky","year":2003,"color":"#7BC4C4","pantone_value":"14-4811"},{"id":5,"name":"tigerlily","year":2004,"color":"#E2583E","pantone_value":"17-1456"},{"id":6,"name":"blue turquoise","year":2005,"color":"#53B0AE","pantone_value":"15-5217"}],"support":{"url":"https://reqres.in/#support-heading","text":"To keep ReqRes free, contributions towards server costs are appreciated!"}}
use bls_common::Products;

#[wasm_bindgen]
pub async fn run(url: i32) -> Result<JsValue, JsValue> {
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
