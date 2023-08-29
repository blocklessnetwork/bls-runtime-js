;; (module
;;     (func $i (import "blockless" "run") (param i32))
;;     (func (export "exported_func")
;;         i32.const 42
;;         call $i
;;     )
;; )

(module
    ;; (import "js" "mem" (memory 1))
    (func $run (import "blockless" "run") (param i32 i32) (result i32)) ;; run function accepts pointer and length
    (func (export "exported_func") (param i32 i32) (result i32) ;; accepts pointer and length
        local.get 0
        local.get 1
        call $run
    )
    (memory (export "memory") 10)
)