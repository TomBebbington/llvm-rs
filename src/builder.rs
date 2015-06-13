use libc::{c_char, c_uint};
use ffi::prelude::{LLVMBuilderRef, LLVMValueRef};
use ffi::{core, LLVMBuilder};
use std::{mem, ptr};
use block::BasicBlock;
use context::Context;
use ty::Type;
use util::{CBox, DisposeRef};
use value::Value;

static NULL_NAME:[c_char; 1] = [0];

/// This provides a uniform API for creating instructions and inserting them into a basic block
pub struct Builder;
native_ref!(&Builder = LLVMBuilderRef);
macro_rules! bin_op(
    ($name:ident, $func:ident) => (
        pub fn $name(&self, left: &Value, right: &Value) -> &Value {
            unsafe { core::$func(self.into(), left.into(), right.into(), NULL_NAME.as_ptr()) }.into()
        }
    );
);
macro_rules! un_op(
    ($name:ident, $func:ident) => (
        pub fn $name(&self, value: &Value) -> &Value {
            unsafe { core::$func(self.into(), value.into(), NULL_NAME.as_ptr() as *const c_char) }.into()
        }
    );
);
impl Builder {
    /// Create a new builder in the context given
    pub fn new(context: &Context) -> CBox<Builder> {
        CBox::new(unsafe { core::LLVMCreateBuilderInContext(context.into()) }.into())
    }
    /// Position the builder at the end of `block`
    pub fn position_at_end(&self, block: &BasicBlock) {
        unsafe { core::LLVMPositionBuilderAtEnd(self.into(), block.into()) }
    }
    /// Build an instruction that returns from the function with void
    pub fn build_ret_void(&self) -> &Value {
        unsafe { core::LLVMBuildRetVoid(self.into()) }.into()
    }
    /// Build an instruction that returns from the function with `value`
    pub fn build_ret(&self, value: &Value) -> &Value {
        unsafe { core::LLVMBuildRet(self.into(), value.into()) }.into()
    }
    /// Build an instruction that allocates an array with the element type `elem` and the size `size`
    ///
    /// The size of this array will be the `size` of elem times `size`
    pub fn build_array_alloca(&self, elem: &Type, size: &Value) -> &Value {
        unsafe { core::LLVMBuildArrayAlloca(self.into(), elem.into(), size.into(), ptr::null_mut()) }.into()
    }
    pub fn build_alloca(&self, ty: &Type) -> &Value {
        unsafe { core::LLVMBuildAlloca(self.into(), ty.into(), ptr::null_mut()) }.into()
    }
    /// Frees the `val`
    pub fn build_free(&self, val: &Value) -> &Value {
        unsafe { core::LLVMBuildFree(self.into(), val.into()) }.into()
    }
    /// Store the value `val` in `ptr`
    pub fn build_store(&self, val: &Value, ptr: &Value) -> &Value {
        unsafe { core::LLVMBuildStore(self.into(), val.into(), ptr.into()) }.into()
    }
    /// Build an instruction that branches to `dest`
    pub fn build_br(&self, dest: &BasicBlock) -> &Value {
        unsafe { core::LLVMBuildBr(self.into(), dest.into()).into() }
    }
    /// Build an instruction that branches to `if_block` if `cond` evaluates to true, and `else_block` otherwise
    pub fn build_cond_br(&self, cond: &Value, if_block: &BasicBlock, else_block: Option<&BasicBlock>) -> &Value {
        unsafe { core::LLVMBuildCondBr(self.into(), cond.into(), if_block.into(), mem::transmute(else_block)).into() }
    }
    /// Build an instruction that calls the function `func` with the arguments `args`
    pub fn build_call(&self, func: &Value, args: &[&Value]) -> &Value {
        unsafe { core::LLVMBuildCall(self.into(), func.into(), args.as_ptr() as *mut LLVMValueRef, args.len() as c_uint, NULL_NAME.as_ptr()).into() }
    }
    /// Build an instruction that yields to `true_val` if `cond` is equal to `1`, and `false_val` otherwise
    pub fn build_select(&self, cond: &Value, true_val: &Value, false_val: &Value) -> &Value {
        unsafe { core::LLVMBuildSelect(self.into(), cond.into(), true_val.into(), false_val.into(), NULL_NAME.as_ptr()).into() }
    }
    un_op!{build_load, LLVMBuildLoad}
    un_op!{build_neg, LLVMBuildNeg}
    un_op!{build_not, LLVMBuildNot}
    bin_op!{build_add, LLVMBuildAdd}
    bin_op!{build_sub, LLVMBuildSub}
    bin_op!{build_mul, LLVMBuildMul}
    bin_op!{build_fdiv, LLVMBuildFDiv}
    bin_op!{build_sdiv, LLVMBuildSDiv}
    bin_op!{build_and, LLVMBuildAnd}
    bin_op!{build_or, LLVMBuildOr}
}

impl DisposeRef for Builder {
    type RefTo = LLVMBuilder;
    #[inline(always)]
    unsafe fn dispose(ptr: LLVMBuilderRef) {
        core::LLVMDisposeBuilder(ptr)
    }
}
