(module
    ;; (type (;0;) (func))
    ;; (type (;1;) (func (result i32)))
    ;; (type (;2;) (func (result i64)))
    ;; (type (;3;) (func (param i32)))
    ;; (type (;4;) (func (param i32) (result i32)))
    ;; (type (;5;) (func (param i32) (result i64)))
    ;; (type (;6;) (func (param i32 i32)))
    ;; (type (;7;) (func (param i32 i32) (result i32)))
    ;; (import "wbg" "__wbindgen_json_parse" (func $wasm_bindgen::__wbindgen_json_parse::hae9a7c632dd5676a (type 7)))

    (func $i (import "blockless" "run") (param i32))
    ;; (func $i (import "imports" "imported_func") (param i32))
    (func (export "exported_func")
        i32.const 42
        call $i
    )
)