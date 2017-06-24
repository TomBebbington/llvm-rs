extern crate llvm_sys;
extern crate llvm;
use llvm::*;
use llvm::Attribute::*;
fn main() {
    let ctx = Context::new();
    let module = Module::new("simple", &ctx);
    type N = f64;
    type T = extern "C" fn(N) -> N;
    let func = module.add_function(
        "thr", Type::get::<T>(&ctx));
    func.add_attributes(&[NoUnwind, ReadNone]);
    let entry = func.append("entry");
    let builder = Builder::new(&ctx);
    fn n(x: N) -> N { x }
    let three_r = n(3 as N).compile(&ctx);
    builder.position_at_end(entry);
    builder.build_ret(three_r);
    module.verify().unwrap();

    let ee = llvm::JitEngine::new(
        &module, llvm::JitOptions {opt_level: 0}).unwrap();
    println!("{:?}", module);
    ee.with_function(func, |thr: T| {
        for i in 0..3 {
            println!("thr {} = {}", i, thr(0 as N))
        }
    });
    ee.remove_module(&module);
}
