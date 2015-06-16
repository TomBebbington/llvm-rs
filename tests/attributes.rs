extern crate llvm;
use llvm::*;
use llvm::Attribute::*;
#[test]
fn test_function_attributes() {
    let ctx = Context::new();
    let module = Module::new("simple", &ctx);
    let func = module.add_function("main", Type::get::<fn() -> ()>(&ctx));
    assert!(!func.has_attributes(&[NoUnwind, ReadNone]));
    func.add_attributes(&[NoUnwind, ReadNone]);
    assert!(func.has_attributes(&[NoUnwind, ReadNone]));
    func.remove_attribute(NoUnwind);
    assert!(!func.has_attribute(NoUnwind));
    assert!(func.has_attribute(ReadNone));
}

#[test]
fn test_arg_attributes() {
    let ctx = Context::new();
    let module = Module::new("simple", &ctx);
    let func = module.add_function("main", Type::get::<fn() -> ()>(&ctx));
    assert!(!func.has_attribute(NoUnwind));
    assert!(!func.has_attribute(ReadNone));
    func.add_attributes(&[NoUnwind, ReadNone]);
    assert!(func.has_attribute(NoUnwind));
    assert!(func.has_attribute(ReadNone));
    func.remove_attribute(NoUnwind);
    assert!(!func.has_attribute(NoUnwind));
    assert!(func.has_attribute(ReadNone));
}
