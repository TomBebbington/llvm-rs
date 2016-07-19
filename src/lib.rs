//! This library provides wrappers for LLVM that are memory-safe and follow
//! Rust idioms.
//!
//! The original LLVM reference is available [here](http://llvm.org/doxygen/)
//! but take note that this isn't as thorough as this documentation.

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
pub mod types;
pub mod value;
mod util;

pub use cbox::{CBox, CSemiBox};
pub use builder::Builder;
pub use block::BasicBlock;
pub use compile::Compile;
pub use context::{Context, GetContext};
pub use engine::{JitEngine, JitOptions, Interpreter, ExecutionEngine, GenericValue, GenericValueCast};
pub use module::{AddressSpace, Module, Functions};
pub use object::{ObjectFile, Symbol, Symbols};
pub use target::{TargetData, Target};
pub use types::*;
pub use value::{Alias, Arg, Attribute, Value, Function, GlobalValue, GlobalVariable, Linkage, Predicate};
pub use util::Sub;
