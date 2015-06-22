use ffi::prelude::LLVMTypeRef;
use ffi::{core, target, LLVMTypeKind};
use libc::{c_int, c_uint};
use compile::Compile;
use context::{Context, GetContext};
use target::TargetData;
use util::{self, CastFrom};
use std::{fmt, mem};
use std::iter::Iterator;
use std::ops::Deref;

/// Defines how a value should be laid out in memory.
pub struct Type;
native_ref!(&Type = LLVMTypeRef);
impl Type {
    #[inline(always)]
    /// Get the type given as an LLVM type descriptor in the context given.
    pub fn get<'a, T>(context:&'a Context) -> &'a Type where T:Compile<'a> {
        T::get_type(context)
    }
    /// Make a new function signature with the return type and arguments given.
    pub fn new_function<'a>(ret: &'a Type, args: &[&'a Type]) -> &'a FunctionType {
        unsafe { core::LLVMFunctionType(ret.into(), args.as_ptr() as *mut LLVMTypeRef, args.len() as c_uint, 0) }.into()
    }
    /// Make a new array with the length given.
    pub fn new_array<'a>(element: &'a Type, length: usize) -> &'a Type {
        unsafe { core::LLVMArrayType(element.into(), length as c_uint) }.into()
    }
    /// Make a new vector with the length given.
    pub fn new_vector<'a>(element: &'a Type, length: usize) -> &'a Type {
        unsafe { core::LLVMVectorType(element.into(), length as c_uint) }.into()
    }
    /// Make a new pointer with the given element type.
    pub fn new_pointer<'a>(elem: &'a Type) -> &'a Type {
        unsafe { core::LLVMPointerType(elem.into(), 0 as c_uint) }.into()
    }
    /// Make a new structure type with the given types.
    pub fn new_struct<'a>(elems: &[&'a Type], packed: bool) -> &'a Type {
        unsafe { core::LLVMStructType(elems.as_ptr() as *mut LLVMTypeRef, elems.len() as c_uint, packed as c_int) }.into()
    }
    /// Returns true if the size of the type is known at compile-time.
    ///
    /// This is equivalent to the type implementing `Sized` in Rust
    pub fn is_sized(&self) -> bool {
        unsafe { core::LLVMTypeIsSized(self.into()) != 0 }
    }
    /// Returns true if this type is a struct.
    pub fn is_struct(&self) -> bool {
        let kind = unsafe { core::LLVMGetTypeKind(self.into()) };
        kind as c_uint == LLVMTypeKind::LLVMFunctionTypeKind as c_uint
    }
    /// Returns the size of the type in bytes.
    pub fn get_size(&self, target: &TargetData) -> usize {
        unsafe { target::LLVMABISizeOfType(target.into(), self.into()) as usize }
    }
    /// Returns the element of this pointer type.
    pub fn get_element(&self) -> Option<&Type> {
        unsafe { mem::transmute(core::LLVMGetElementType(self.into())) }
    }
}
get_context!(Type, LLVMGetTypeContext);
to_str!(Type, LLVMPrintTypeToString);

/// A structure type, such as a tuple or struct
pub struct StructType;
native_ref!(&StructType = LLVMTypeRef);
impl StructType {
    /// Make a new struct with the given fields and packed representation.
    pub fn new<'a>(context: &'a Context, fields: &[&'a Type], packed: bool) -> &'a StructType {
        unsafe { core::LLVMStructTypeInContext(context.into(), fields.as_ptr() as *mut LLVMTypeRef, fields.len() as c_uint, packed as c_int) }.into()
    }
    /// Make a new named struct with the given fields and packed representation.
    pub fn new_named<'a>(context: &'a Context, name: &str, fields: &[&'a Type], packed: bool) -> &'a StructType {
        util::with_cstr(name, |name| unsafe {
            let ty = core::LLVMStructCreateNamed(context.into(), name);
            core::LLVMStructSetBody(ty, fields.as_ptr() as *mut LLVMTypeRef, fields.len() as c_uint, packed as c_int);
            ty.into()
        })
    }
    /// Returns the elements (AKA fields) that make up this struct.
    pub fn get_elements(&self) -> Vec<Type> {
        unsafe {
            let size = core::LLVMCountStructElementTypes(self.into());
            let mut els:Vec<_> = (0..size).map(|_| mem::uninitialized()).collect();
            core::LLVMGetStructElementTypes(self.into(), els.as_mut_ptr() as *mut LLVMTypeRef);
            els
        }
    }
}
impl CastFrom for StructType {
    type From = Type;
    fn cast(ty: &Type) -> Option<&StructType> {
        unsafe {
            let kind = core::LLVMGetTypeKind(ty.into());
            if kind as c_uint == LLVMTypeKind::LLVMStructTypeKind as c_uint {
                mem::transmute(ty)
            } else {
                None
            }
        }
    }
}
deref!(StructType, Type);
get_context!(StructType, LLVMGetTypeContext);
to_str!(StructType, LLVMPrintTypeToString);

/// A function signature type
pub struct FunctionType;
native_ref!(&FunctionType = LLVMTypeRef);
deref!(FunctionType, Type);
impl CastFrom for FunctionType {
    type From = Type;
    fn cast(mut ty: &Type) -> Option<&FunctionType> {
        unsafe {
            use libc::c_uint;
            while let Some(elem) = ty.get_element() {
                ty = elem;
            }
            let kind = core::LLVMGetTypeKind(ty.into());
            if kind as c_uint == LLVMTypeKind::LLVMFunctionTypeKind as c_uint {
                mem::transmute(ty)
            } else {
                None
            }
        }
    }
}
impl FunctionType {
    /// Returns how many parameters this signature takes.
    pub fn num_params(&self) -> usize {
        unsafe { core::LLVMCountParamTypes(self.into()) as usize }
    }
    /// Returns a vector of this signature's parameters' types.
    pub fn get_params(&self) -> Vec<&Type> {
        unsafe {
            let count = core::LLVMCountParamTypes(self.into());
            let mut types:Vec<_> = (0..count).map(|_| mem::uninitialized()).collect();
            core::LLVMGetParamTypes(self.into(), types.as_mut_ptr() as *mut LLVMTypeRef);
            types
        }
    }
    /// Returns the type that this function returns.
    pub fn get_return(&self) -> &Type {
        unsafe { core::LLVMGetReturnType(self.into()).into() }
    }
}
get_context!(FunctionType, LLVMGetTypeContext);
to_str!(FunctionType, LLVMPrintTypeToString);
