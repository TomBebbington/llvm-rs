//! This library provides wrappers for LLVM that are (mostly) memory-safe and follow
//! Rust idioms

extern crate llvm_sys as ffi;
extern crate libc;

#[macro_use]
mod macros;
mod buffer;
mod block;
mod builder;
mod compile;
mod context;
mod engine;
mod module;
mod pass;
mod target;
mod ty;
mod value;
mod util;

pub use builder::Builder;
pub use block::BasicBlock;
pub use compile::Compile;
pub use context::{Context, GetContext};
pub use engine::{JitEngine, JitOptions, Interpreter, ExecutionEngine, GenericValue, GenericValueCast};
pub use module::{Module, Functions};
pub use pass::{PassManager, PassManagerBuilder};
pub use target::{TargetData, Target};
pub use ty::{FunctionType, Type};
pub use value::{Value, Function};
pub use util::CBox;
