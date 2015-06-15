//! This library provides wrappers for LLVM that are (mostly) memory-safe and follow
//! Rust idioms
//!
//! Ownership of some of the LLVM types is managed through the `CBox` struct, which
//! just wraps a pointer and calls a destructor when it falls out of scope, but allows
//! being dereferenced into the type it represents e.g. a `Context`

extern crate llvm_sys as ffi;
extern crate libc;
extern crate cbox;

#[macro_use]
mod macros;
mod buffer;
mod block;
mod builder;
mod compile;
mod context;
mod engine;
mod module;
mod object;
mod pass;
mod target;
mod ty;
mod value;
mod util;

pub use cbox::CBox;
pub use builder::Builder;
pub use block::BasicBlock;
pub use compile::Compile;
pub use context::{Context, GetContext};
pub use engine::{JitEngine, JitOptions, Interpreter, ExecutionEngine, GenericValue, GenericValueCast};
pub use module::{Module, Functions};
pub use object::{ObjectFile, Symbol, Symbols};
pub use pass::{PassManager, PassManagerBuilder};
pub use target::{TargetData, Target};
pub use ty::{FunctionType, StructType, Type};
pub use value::{Arg, Attribute, Value, Function};
pub use util::CastFrom;
