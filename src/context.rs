use ffi::prelude::LLVMContextRef;
use ffi::{core, LLVMContext};
use util::CBox;

/// Contains all the LLVM entities
///
/// Every single entity attached to it has its lifetime to enforce the
/// rule that things from different contexts cannot interact
pub struct Context;
native_ref!(&Context = LLVMContextRef);
impl Context {
    /// Get a reference to the global context which has the same lifetime as
    /// the LLVM holder
    ///
    /// This is marked as unsafe because this results in undefined behaviour
    /// in a multithreaded context
    pub unsafe fn get_global() -> &'static Context {
        core::LLVMGetGlobalContext().into()
    }
    /// Create a new context, owned by the block that calls it
    pub fn new<'a>() -> CBox<'a, Self> {
        CBox::new(unsafe { core::LLVMContextCreate() })
    }
}
dispose!(Context, LLVMContext, core::LLVMContextDispose);

/// Implemented by everything that is owned by a context
pub trait GetContext {
    /// Returns a reference to the context that owns this value
    fn get_context(&self) -> &Context;
}
