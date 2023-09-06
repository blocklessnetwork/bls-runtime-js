//! Small example of how to instantiate a wasm module that imports one function,
//! showing how you can fill in host functionality for a wasm module.

// You can execute this example with `cargo run --example hello`

pub use wasm_bridge::*;

pub async fn run_wasm(wasm: &[u8]) -> Result<i32, Error>{
  let mut store = Store::<()>::default();
  let module = Module::new(&store.engine(), wasm).unwrap();

  // let module = new_module_async(store.engine(), bytes).await?;
  let mut linker = Linker::new(store.engine());
  linker.func_wrap("blockless", "host_log", |caller: Caller<()>, ptr: u32, len: u32| {
    println!("{} {}", ptr, len);
  })?;
  linker.func_wrap("blockless", "host_call", | mut caller: Caller<()>, ptr: u32, len: u32| {
    println!("{} {}", ptr, len);
    // let memory = caller
    //   .get_export("memory")
    //   .and_then(|e| e.into_memory())
    //   .expect("failed to get memory export");
    1u32
  })?;
  // linker.func_new_async(
  //   "blockless",
  //   "host_call",
  //   wasm_bridge::FuncType::new(
  //     vec![wasm_bridge::ValType::I32, wasm_bridge::ValType::I32],
  //     vec![wasm_bridge::ValType::I32],
  //   ),
  //   |mut caller: Caller<()>, params: &[wasm_bridge::Val], results: &mut [wasm_bridge::Val]| {
  //       Box::new(async move {
  //           results[0] = wasm_bridge::Val::from(1); // store non-zero exit code

  //           // println!("blockless module called.");
  //           // println!("params: {:?}", params);
  //           // let (call_ptr, call_ptr_len) = (params[0].unwrap_i32() as usize, params[1].unwrap_i32() as usize);
  //           // let memory = caller
  //           //   .get_export("memory")
  //           //   .and_then(|e| e.into_memory())
  //           //   .expect("failed to get memory export");
  //           // let param_bytes = memory.data(&caller)
  //           //   .get(call_ptr..)
  //           //   .and_then(|arr| arr.get(..call_ptr_len))
  //           //   .expect("pointer/length out of bounds");
  //           // let param_str = std::str::from_utf8(param_bytes).expect("invalid utf-8");
  //           // println!("param_str: {:?}", param_str);
  //           Ok(())
  //       })
  //     }).unwrap();


  let instance = linker.instantiate(&mut store, &module).unwrap();

  let start_func = instance.get_typed_func::<(), ()>(&mut store, "_start").unwrap();
  start_func.call(&mut store, ()).unwrap();
  Ok(1)
}

pub fn double_no_imports() -> Result<i32, Error>{
  let wat = r#"
  (module
    (func (export "double") (param i32) (result i32)
      local.get 0
      i32.const 2
      i32.mul
    )
  )
  "#;
  let mut store = Store::<()>::default();
  let module = Module::new(&store.engine(), wat.as_bytes()).unwrap();
  let instance = Instance::new(&mut store, &module, &[]).unwrap();
  let double_func = instance.get_typed_func::<i32, i32>(&mut store, "double").unwrap();
  let result = double_func.call(&mut store, 4).unwrap();
  Ok(result)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_double_no_imports() {
        let result = double_no_imports();
        assert_eq!(result.unwrap(), 8);
    }
}