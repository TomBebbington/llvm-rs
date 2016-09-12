extern crate llvm;
use llvm::*;

#[test]
fn test_ir_reader() {
    let ctx = Context::new();
    let ir_text = "define i32 @main() { ret i32 42 }";

    let module = Module::parse_ir_from_str(&ctx, ir_text);
    assert!(module.is_ok(), "Failed to prase LLVM IR: {}", module.err().unwrap());
}
