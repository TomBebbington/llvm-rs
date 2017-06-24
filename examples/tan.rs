extern crate llvm;
use llvm::*;
use llvm::Attribute::*;
fn main() {
    let ctx = Context::new();
    let module = Module::new("simple", &ctx);
    let cos = module.add_function("llvm.cos.f64", Type::get::<fn(f64) -> f64>(&ctx));
    let sin = module.add_function("llvm.sin.f64", Type::get::<fn(f64) -> f64>(&ctx));
    let func = module.add_function("tan", Type::get::<fn(f64) -> f64>(&ctx));
    func.add_attributes(&[NoUnwind, ReadNone]);
    let entry = func.append("entry");
    let builder = Builder::new(&ctx);
    builder.position_at_end(entry);
    let value = &func[0];
    let sin_v = builder.build_call(sin, &[value]);
    let cos_v = builder.build_call(cos, &[value]);
    let value = builder.build_div(sin_v, cos_v);
    builder.build_ret(value);
    module.verify().unwrap();
    let ee = JitEngine::new(&module, JitOptions {opt_level: 0}).unwrap();
    ee.with_function(func, |tan:extern fn(f64) -> f64| {
        for i in 0..10 {
            let i = i as f64;
            println!("tan {} = {}", i, tan(i))
        }
    });
    ee.remove_module(&module);
}
