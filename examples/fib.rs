extern crate llvm;
use llvm::*;
use llvm::Attribute::*;
fn main() {
    let ctx = Context::new();
    let module = Module::new("simple", &ctx);
    let func = module.add_function("fib", Type::get::<fn(u64) -> u64>(&ctx));
    func.add_attributes(&[NoUnwind, ReadNone]);
    let value = &func[0];
    let entry = func.append("entry");
    let on_zero = func.append("on_zero");
    let on_one = func.append("on_one");
    let default = func.append("default");
    let builder = Builder::new(&ctx);
    let zero = 0u64.compile(&ctx);
    let one = 1u64.compile(&ctx);
    builder.position_at_end(entry);
    builder.build_switch(value, default, &[
        (zero, on_zero),
        (one, on_one)
    ]);
    builder.position_at_end(on_zero);
    builder.build_ret(zero);
    builder.position_at_end(on_one);
    builder.build_ret(one);
    builder.position_at_end(default);
    let two = 2u64.compile(&ctx);
    let a = builder.build_sub(value, one);
    let b = builder.build_sub(value, two);
    let fa = builder.build_tail_call(func, &[a]);
    let fb = builder.build_tail_call(func, &[b]);
    builder.build_ret(builder.build_add(fa, fb));
    module.verify().unwrap();
    let ee = JitEngine::new(&module, JitOptions {opt_level: 0}).unwrap();
    ee.with_function(func, |fib: extern fn(u64) -> u64| {
        for i in 0..10 {
            println!("fib {} = {}", i, fib(i))
        }
    });
}
