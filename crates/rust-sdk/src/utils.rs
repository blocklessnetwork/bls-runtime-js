
#![allow(dead_code)]
fn log(msg: &str) {
  let msg_bytes = msg.as_bytes();
  let ptr = msg_bytes.as_ptr() as u32;
  let len = msg_bytes.len() as u32;
  unsafe {
      crate::host_log(ptr, len);
  }
}

#[macro_export]
macro_rules! log {
  ($($t:tt)*) => {
    let msg = &format_args!($($t)*).to_string();
    let msg_bytes = msg.as_bytes();
    let ptr = msg_bytes.as_ptr() as u32;
    let len = msg_bytes.len() as u32;
    unsafe {
      host_log(ptr, len);
    }
  }
}