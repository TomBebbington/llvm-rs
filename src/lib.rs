//! This library provides wrappers for LLVM that are (mostly) memory-safe and follow
//! Rust idioms.

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
mod target;
mod ty;
mod value;
mod util;
mod phi;

pub use cbox::{CBox, CSemiBox};
pub use builder::Builder;
pub use block::BasicBlock;
pub use compile::Compile;
pub use context::{Context, GetContext};
pub use engine::{JitEngine, JitOptions, Interpreter, ExecutionEngine, GenericValue, GenericValueCast};
pub use module::{Module, Functions};
pub use object::{ObjectFile, Symbol, Symbols};
pub use target::{TargetData, Target};
pub use ty::{FunctionType, StructType, Type};
pub use value::{Arg, Attribute, Value, Function, Predicate};
pub use util::CastFrom;
pub use phi::PhiNode;
