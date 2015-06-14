//! This example shows how to compile a program that prints 'Hello, world!' using LLVM and GCC
extern crate llvm;
extern crate libc;
use llvm::*;
use libc::*;
use std::process::Command;
use std::{env, fs};
fn main() {
    let ctx = Context::new();
    let module = Module::new("run", &ctx);
    // Add the extern for the function `printf`
    let printf = module.add_function("printf", Type::get::<fn(*const c_char)>(&ctx));
    // The main signature
    let sig = Type::get::<fn(c_int, usize) -> c_int>(&ctx);
    let func = module.add_function("main", sig);
    let entry = func.append("entry");
    let builder = Builder::new(&ctx);
    builder.position_at_end(&entry);
    // Make a new string constant
    let text = Value::new_string(&ctx, "Hello, world!\n", false);
    // Allocate a pointer for the constant
    let text_ptr = builder.build_alloca(text.get_type());
    let text_ptr_ty = text_ptr.get_type();
    builder.build_store(text, text_ptr);
    let text_ptr = builder.build_bit_cast(text_ptr, Type::get::<*const c_char>(&ctx));
    // Call the print function with the pointer
    builder.build_call(printf, &[text_ptr]);
    // Free the string
    builder.build_free(builder.build_bit_cast(text_ptr, text_ptr_ty));
    // Return 0 to show success
    builder.build_ret(0i32.compile(&builder, &ctx));
    let temp = env::temp_dir();
    let folder = temp.join("llvm-rs-module");
    fs::create_dir(&folder);
    let obj_path = folder.join("run.o");
    // Compile the module into an object file
    module.compile(&obj_path, 3).ok().expect("Could not make object file");
    let obj_path = obj_path.to_str().unwrap();
    let bin_path = folder.join("run");
    let bin_path = bin_path.to_str().unwrap();

    // Link the object file to produce an executable
    Command::new("gcc")
        .arg(obj_path)
        .arg("-o")
        .arg(bin_path)
        .status().ok().expect("could not link");
    // Run the new executable
    Command::new(bin_path).status().ok().expect("error in binary");
    fs::remove_dir_all(&folder).unwrap();
}
