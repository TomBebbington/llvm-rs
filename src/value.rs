use libc::{c_uint, c_int};
use ffi::prelude::LLVMValueRef;
use ffi::core;
use std::ffi::CString;
use std::{fmt, mem};
use std::ops::{Deref, Index};
use block::BasicBlock;
use context::{Context, GetContext};
use ty::{FunctionType, Type};
use util;

/// A typed value that can be used as an operand in instructions
pub struct Value;
native_ref!(&Value = LLVMValueRef);
impl Index<usize> for Value {
    type Output = Value;
    fn index(&self, index: usize) -> &Value {
        unsafe {
            if index < core::LLVMCountParams(self.into()) as usize {
                core::LLVMGetParam(self.into(), index as c_uint).into()
            } else {
                panic!("no such index {} on {:?}", index, self.get_type())
            }
        }
    }
}
impl Value {
    pub fn new_struct<'a>(context: &'a Context, vals: &[&'a Value], packed: bool) -> &'a Value {
        unsafe { core::LLVMConstStructInContext(context.into(), vals.as_ptr() as *mut LLVMValueRef, vals.len() as c_uint, packed as c_int) }.into()
    }
    pub fn get_name(&self) -> Option<&str> {
        unsafe {
            let c_name = core::LLVMGetValueName(self.into());
            util::to_null_str(c_name as *mut i8)
        }
    }
    pub fn set_name(&self, name: &str) {
        let c_name = CString::new(name).unwrap();
        unsafe {
            core::LLVMSetValueName(self.into(), c_name.as_ptr())
        }
    }
    pub fn get_type(&self) -> &Type {
        unsafe { core::LLVMTypeOf(self.into()) }.into()
    }
}
/// A `Value` that represents a `Function`
pub struct Function;
native_ref!(&Function = LLVMValueRef);
impl Deref for Function {
    type Target = Value;
    fn deref(&self) -> &Value {
        unsafe { mem::transmute(self) }
    }
}
impl Function {
    pub fn append<'a>(&'a self, name: &str) -> &'a BasicBlock {
        util::with_cstr(name, |ptr| unsafe {
            core::LLVMAppendBasicBlockInContext(self.get_context().into(), self.into(), ptr).into()
        })
    }
    pub fn get_entry(&self) -> Option<&BasicBlock> {
        unsafe { mem::transmute(core::LLVMGetEntryBasicBlock(self.into())) }
    }

    pub fn get_name(&self) -> &str {
        unsafe {
            let c_name = core::LLVMGetValueName(self.into());
            util::to_str(c_name as *mut i8)
        }
    }
    pub fn get_signature(&self) -> &FunctionType {
        unsafe { core::LLVMTypeOf(self.into()) }.into()
    }
}
impl GetContext for Value {
    fn get_context(&self) -> &Context {
        self.get_type().get_context()
    }
}
to_str!(Value, LLVMPrintValueToString);
