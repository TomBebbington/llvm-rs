extern crate llvm;
use llvm::*;
fn main() {
    let ctx = Context::new();
    let module = Module::new("simple", &ctx);
    let cos = module.add_function("cos", Type::get::<fn(f64) -> f64>(&ctx));
    let sin = module.add_function("sin", Type::get::<fn(f64) -> f64>(&ctx));
    let func = module.add_function("tan", Type::get::<fn(f64) -> f64>(&ctx));
    let entry = func.append("entry");
    let builder = Builder::new(&ctx);
    builder.position_at_end(entry);
    let value = &func[0];
    let sin_v = builder.build_call(sin, &[value]);
    let cos_v = builder.build_call(cos, &[value]);
    let value = builder.build_fdiv(sin_v, cos_v);
    builder.build_ret(value);
    println!("{:?}", &module as &Module);
    let ee = JitEngine::new(&module, JitOptions {opt_level: 0}).unwrap();
    ee.with_function(func, |tan: extern fn(f64) -> f64| {
        for i in 0..10 {
            let i = i as f64;
            println!("tan {} = {}", i, tan(i))
        }
    });
}
