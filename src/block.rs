use ffi::core;
use ffi::prelude::LLVMBasicBlockRef;
use std::marker::PhantomData;
use value::Function;
use util;

/// A container of instructions that execute sequentially.
pub struct BasicBlock(PhantomData<[u8]>);
native_ref!(&BasicBlock = LLVMBasicBlockRef);
impl BasicBlock {
    /// Return the enclosing method, or `None` if it is not attached to a method.
    pub fn get_parent(&self) -> Option<&Function> {
        unsafe { util::ptr_to_null(core::LLVMGetBasicBlockParent(self.into())) }
    }
    /// Move this basic block after the `other` basic block in its function.
    pub fn move_after(&self, other: &BasicBlock) {
        unsafe { core::LLVMMoveBasicBlockAfter(self.into(), other.into()) }
    }
    /// Move this basic block before the `other` basic block in its function.
    pub fn move_before(&self, other: &BasicBlock) {
        unsafe { core::LLVMMoveBasicBlockBefore(self.into(), other.into()) }
    }
    /// Unlink from the containing function, but do not delete it.
    pub fn remove(&self) {
        unsafe { core::LLVMRemoveBasicBlockFromParent(self.into()) }
    }
    /// Delete this basic block.
    ///
    /// This is unsafe because there should be no other reference to this, but
    /// this can't be guranteed using Rust semantics.
    pub unsafe fn delete(&self) {
        core::LLVMDeleteBasicBlock(self.into())
    }
}
