use ffi::core;
use ffi::prelude::LLVMBasicBlockRef;
use std::iter::{Iterator, DoubleEndedIterator, IntoIterator};
use std::marker::PhantomData;
use std::mem;
use std::ops::Deref;
use value::{Function, Value};
use util::{self, Sub};

/// A container of instructions that execute sequentially.
pub struct BasicBlock(PhantomData<[u8]>);
deref!{BasicBlock, Value}
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
    /// Return the terminator instruction for this basic block.
    pub fn get_terminator(&self) -> Option<&Value> {
        unsafe { util::ptr_to_null(core::LLVMGetBasicBlockTerminator(self.into())) }
    }
    /// Return the first instruction for this basic block.
    pub fn get_first(&self) -> Option<&Value> {
        unsafe { util::ptr_to_null(core::LLVMGetFirstInstruction(self.into())) }
    }
    /// Return the last instruction for this basic block.
    pub fn get_last(&self) -> Option<&Value> {
        unsafe { util::ptr_to_null(core::LLVMGetLastInstruction(self.into())) }
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

/// Iterates through all the blocks contained in a function.
pub struct BlockIter<'a> {
    pub min: &'a BasicBlock,
    pub max: &'a BasicBlock
}
impl<'a> BlockIter<'a> {
    pub fn new(function: &'a Function) -> BlockIter<'a> {
        BlockIter {
            min: unsafe { core::LLVMGetFirstBasicBlock(function.into()).into() },
            max: unsafe { core::LLVMGetLastBasicBlock(function.into()).into() }
        }
    }
}

impl<'a> IntoIterator for &'a Function {
    type IntoIter = BlockIter<'a>;
    type Item = &'a BasicBlock;
    fn into_iter(self) -> BlockIter<'a> {
        BlockIter::new(self)
    }
}
impl<'a> Iterator for BlockIter<'a> {
    type Item = &'a BasicBlock;
    fn next(&mut self) -> Option<&'a BasicBlock> {
        if self.min == self.max {
            None
        } else {
            unsafe {
                let _block = self.min;
                self.min = core::LLVMGetNextBasicBlock(self.min.into()).into();
                Some(_block)
            }
        }
    }
}
impl<'a> DoubleEndedIterator for BlockIter<'a> {
    fn next_back(&mut self) -> Option<&'a BasicBlock> {
        if self.min == self.max {
            None
        } else {
            unsafe {
                let _block = self.max;
                self.max = core::LLVMGetPreviousBasicBlock(self.max.into()).into();
                Some(_block)
            }
        }
    }
}