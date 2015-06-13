use ffi::prelude::LLVMTypeRef;
use ffi::{core, target, LLVMTypeKind};
use libc::{c_int, c_uint};
use compile::Compile;
use context::{Context, GetContext};
use target::TargetData;
use util;
use std::{fmt, mem};
use std::iter::Iterator;
use std::ops::Deref;

/// Defines how a value should be laid out in memory
pub struct Type;
native_ref!(&Type = LLVMTypeRef);
impl Type {
    #[inline(always)]
    /// Get the type given as an LLVM type descriptor
    pub fn get<'a, T>(context:&'a Context) -> &'a Type where T:Compile<'a> {
        T::get_type(context)
    }
    /// Make a new function signature with the return type and arguments given
    pub fn new_function<'a>(ret: &'a Type, args: &[&'a Type]) -> &'a FunctionType {
        unsafe { core::LLVMFunctionType(ret.into(), args.as_ptr() as *mut LLVMTypeRef, args.len() as c_uint, 0) }.into()
    }
    /// Make a new array with the length given
    pub fn new_array<'a>(element: &'a Type, size: usize) -> &'a Type {
        unsafe { core::LLVMArrayType(element.into(), size as c_uint) }.into()
    }
    /// Make a new vector with the length given
    pub fn new_vector<'a>(element: &'a Type, size: usize) -> &'a Type {
        unsafe { core::LLVMVectorType(element.into(), size as c_uint) }.into()
    }
    /// Make a new struct with the given fields
    pub fn new_struct<'a>(context: &'a Context, fields: &[&'a Type], packed: bool) -> &'a Type {
        unsafe { core::LLVMStructTypeInContext(context.into(), fields.as_ptr() as *mut LLVMTypeRef, fields.len() as c_uint, packed as c_int) }.into()
    }
    /// Make a new pointer with the given type
    pub fn new_pointer<'a>(elem: &'a Type) -> &'a Type {
        unsafe { core::LLVMPointerType(elem.into(), 0 as c_uint) }.into()
    }
    /// Returns true if the size of the type is known
    pub fn is_sized(&self) -> bool {
        unsafe { core::LLVMTypeIsSized(self.into()) != 0 }
    }
    /// Gets the size of the type in bytes
    pub fn get_size(&self, target: &TargetData) -> usize {
        unsafe { target::LLVMABISizeOfType(target.into(), self.into()) as usize }
    }
    /// Get the return type of this function
    pub fn get_return(&self) -> Option<&Type> {
        unsafe { mem::transmute(core::LLVMGetReturnType(self.into())) }
    }
    /// Get the element of this pointer type
    pub fn get_element(&self) -> Option<&Type> {
        unsafe { mem::transmute(core::LLVMGetElementType(self.into())) }
    }
}
get_context!(Type, LLVMGetTypeContext);
to_str!(Type, LLVMPrintTypeToString);


/// Defines how a value should be laid out in memory
pub struct FunctionType;
native_ref!(&FunctionType = LLVMTypeRef);
impl Deref for FunctionType {
    type Target = Type;
    fn deref(&self) -> &Type {
        unsafe { mem::transmute(self) }
    }
}
impl<'a> From<&'a Type> for &'a FunctionType {
    fn from(mut ty: &'a Type) -> &'a FunctionType {
        unsafe {
            use libc::c_uint;
            while let Some(elem) = ty.get_element() {
                ty = elem;
            }
            let kind = core::LLVMGetTypeKind(ty.into());
            if kind as c_uint == LLVMTypeKind::LLVMFunctionTypeKind as c_uint {
                mem::transmute(ty)
            } else {
                panic!("type {:?} cannot be cast into a function", ty)
            }
        }
    }
}
impl FunctionType {
    /// Returns how many parameters this signature takes
    pub fn num_params(&self) -> usize {
        unsafe { core::LLVMCountParamTypes(self.into()) as usize }
    }
    /// Returns a vector of this signature's parameters' types
    pub fn get_params(&self) -> Vec<&Type> {
        unsafe {
            let count = core::LLVMCountParamTypes(self.into());
            let mut types:Vec<_> = (0..count).map(|_| mem::uninitialized()).collect();
            core::LLVMGetParamTypes(self.into(), types.as_mut_ptr() as *mut LLVMTypeRef);
            types
        }
    }
}
get_context!(FunctionType, LLVMGetTypeContext);
to_str!(FunctionType, LLVMPrintTypeToString);
