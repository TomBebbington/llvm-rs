use ffi::core;
use ffi::prelude::LLVMBasicBlockRef;
use std::marker::PhantomData;
use std::mem;
use value::{Function, Value};
use util::{self, Sub};

/// A container of instructions that execute sequentially.
pub struct BasicBlock(PhantomData<[u8]>);
native_ref!(&BasicBlock = LLVMBasicBlockRef);

unsafe impl Sub<Value> for BasicBlock {
    fn is(value: &Value) -> bool {
        unsafe { core::LLVMValueIsBasicBlock(value.into()) != 0 }
    }
    fn from_super(value: &Value) -> Option<&BasicBlock> {
        unsafe {
            mem::transmute(core::LLVMValueAsBasicBlock(value.into()))
        }
    }
    fn to_super(&self) -> &Value {
        unsafe { core::LLVMBasicBlockAsValue(self.into()).into() }
    }
}

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
