use ffi::prelude::LLVMTypeRef;
use ffi::{core, target, LLVMTypeKind};
use libc::{c_int, c_uint};
use compile::Compile;
use context::{Context, GetContext};
use target::TargetData;
use util::{self, Sub};
use std::{fmt, mem};
use std::marker::PhantomData;
use std::iter::Iterator;
use std::ops::Deref;

macro_rules! sub {
    ($ty:ty, $kind:ident) => (
unsafe impl Sub<::Type> for $ty {
    fn is(ty: &Type) -> bool {
        unsafe {
            let kind = core::LLVMGetTypeKind(ty.into());
            kind as c_uint == LLVMTypeKind::$kind as c_uint
        }
    }
}
deref!{$ty, Type}
    )
}

/// Defines how a value should be laid out in memory.
pub struct Type(PhantomData<[u8]>);
native_ref!{&Type = LLVMTypeRef}
get_context!{Type, LLVMGetTypeContext}
to_str!{Type, LLVMPrintTypeToString}
impl Type {
    #[inline(always)]
    /// Get the type given as an LLVM type descriptor in the context given.
    pub fn get<'a, T>(context:&'a Context) -> &'a Type where T:Compile<'a> {
        T::get_type(context)
    }
    /// Returns true if the size of the type is known at compile-time.
    ///
    /// This is equivalent to the type implementing `Sized` in Rust
    pub fn is_sized(&self) -> bool {
        unsafe { core::LLVMTypeIsSized(self.into()) != 0 }
    }
    /// Returns true if this type is a function.
    ///
    /// This is equivalent to `FunctionType::is`.
    pub fn is_function(&self) -> bool {
        let kind = unsafe { core::LLVMGetTypeKind(self.into()) };
        kind as c_uint == LLVMTypeKind::LLVMFunctionTypeKind as c_uint
    }
    /// Returns true if this type is a struct.
    ///
    /// This is equivalent to `StructType::is`.
    pub fn is_struct(&self) -> bool {
        let kind = unsafe { core::LLVMGetTypeKind(self.into()) };
        kind as c_uint == LLVMTypeKind::LLVMStructTypeKind as c_uint
    }
    /// Returns true if this type is void.
    pub fn is_void(&self) -> bool {
        let kind = unsafe { core::LLVMGetTypeKind(self.into()) };
        kind as c_uint == LLVMTypeKind::LLVMVoidTypeKind as c_uint
    }
    /// Returns true if this type is a pointer.
    ///
    /// This is equivalent to `PointerType::is`.
    pub fn is_pointer(&self) -> bool {
        let kind = unsafe { core::LLVMGetTypeKind(self.into()) };
        kind as c_uint == LLVMTypeKind::LLVMPointerTypeKind as c_uint
    }
    /// Returns true if this type is an integer.
    pub fn is_integer(&self) -> bool {
        let kind = unsafe { core::LLVMGetTypeKind(self.into()) };
        kind as c_uint == LLVMTypeKind::LLVMIntegerTypeKind as c_uint
    }
    /// Returns true if this type is any floating-point number.
    pub fn is_float(&self) -> bool {
        let kind = unsafe { core::LLVMGetTypeKind(self.into()) } as c_uint;
        kind == LLVMTypeKind::LLVMHalfTypeKind as c_uint ||
        kind == LLVMTypeKind::LLVMFloatTypeKind as c_uint ||
        kind == LLVMTypeKind::LLVMDoubleTypeKind as c_uint
    }
    /// Returns the size of the type in bytes.
    pub fn get_size(&self, target: &TargetData) -> usize {
        unsafe { target::LLVMABISizeOfType(target.into(), self.into()) as usize }
    }
}

/// A structure type, such as a tuple or struct.
pub struct StructType(PhantomData<[u8]>);
native_ref!{&StructType = LLVMTypeRef}
get_context!{StructType, LLVMGetTypeContext}
to_str!{StructType, LLVMPrintTypeToString}
sub!{StructType, LLVMStructTypeKind}
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
    /// Returns the elements that make up this struct.
    pub fn get_elements(&self) -> Vec<&Type> {
        unsafe {
            let size = core::LLVMCountStructElementTypes(self.into());
            let mut els:Vec<_> = (0..size).map(|_| mem::uninitialized()).collect();
            core::LLVMGetStructElementTypes(self.into(), els.as_mut_ptr() as *mut LLVMTypeRef);
            els
        }
    }
}

/// A function signature type.
pub struct FunctionType(PhantomData<[u8]>);
native_ref!{&FunctionType = LLVMTypeRef}
get_context!{FunctionType, LLVMGetTypeContext}
to_str!{FunctionType, LLVMPrintTypeToString}
deref!{FunctionType, Type}
unsafe impl Sub<Type> for FunctionType {
    fn is(mut ty: &Type) -> bool {
        unsafe {
            while let Some(ptr) = PointerType::from_super(ty) {
                ty = ptr.get_element();
            }
            let kind = core::LLVMGetTypeKind(ty.into());
            kind as c_uint == LLVMTypeKind::LLVMFunctionTypeKind as c_uint
        }
    }
}
impl FunctionType {
    /// Make a new function signature with the return type and arguments given.
    pub fn new<'a>(ret: &'a Type, args: &[&'a Type]) -> &'a FunctionType {
        unsafe { core::LLVMFunctionType(ret.into(), args.as_ptr() as *mut LLVMTypeRef, args.len() as c_uint, 0) }.into()
    }
    /// Returns the number of parameters this signature takes.
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

/// A pointer type.
pub struct PointerType(PhantomData<[u8]>);
native_ref!{&PointerType = LLVMTypeRef}
get_context!{PointerType, LLVMGetTypeContext}
to_str!{PointerType, LLVMPrintTypeToString}
sub!{PointerType, LLVMPointerTypeKind}
impl PointerType {
    /// Make a new pointer type with the given element type.
    pub fn new(elem: &Type) -> &Type {
        unsafe { core::LLVMPointerType(elem.into(), 0 as c_uint) }.into()
    }
    /// Returns the element of this pointer type.
    pub fn get_element(&self) -> &Type {
        unsafe { mem::transmute(core::LLVMGetElementType(self.into())) }
    }
}

/// An integer type.
pub struct IntegerType(PhantomData<[u8]>);
native_ref!{&IntegerType = LLVMTypeRef}
get_context!{IntegerType, LLVMGetTypeContext}
to_str!{IntegerType, LLVMPrintTypeToString}
sub!{IntegerType, LLVMPointerTypeKind}
impl IntegerType {
    /// Make a new integer type that will be the size of the given number of bits.
    pub fn new(context: &Context, numbits: usize) -> &IntegerType {
        unsafe { core::LLVMIntTypeInContext(context.into(), numbits as c_uint) }.into()
    }
    /// Returns how long an integer of this type is, in bits.
    pub fn get_width(&self) -> usize {
        unsafe { core::LLVMGetIntTypeWidth (self.into()) as usize }
    }
}

/// A vector type.
pub struct VectorType(PhantomData<[u8]>);
native_ref!{&VectorType = LLVMTypeRef}
get_context!{VectorType, LLVMGetTypeContext}
to_str!{VectorType, LLVMPrintTypeToString}
sub!{VectorType, LLVMVectorTypeKind}
impl VectorType {
    /// Make a new vector type with the length given.
    pub fn new(element: &Type, length: usize) -> &VectorType {
        unsafe { core::LLVMVectorType(element.into(), length as c_uint) }.into()
    }
    /// Returns the element type of this vector type.
    pub fn get_element(&self) -> &Type {
        unsafe { mem::transmute(core::LLVMGetElementType(self.into())) }
    }
    /// Returns the number of elements in this vector type.
    pub fn get_size(&self) -> usize {
        unsafe { core::LLVMGetVectorSize(self.into()) as usize }
    }
}
/// An array type.
pub struct ArrayType(PhantomData<[u8]>);
native_ref!{&ArrayType = LLVMTypeRef}
get_context!{ArrayType, LLVMGetTypeContext}
to_str!{ArrayType, LLVMPrintTypeToString}
sub!{ArrayType, LLVMArrayTypeKind}
impl ArrayType {
    /// Make a new array type with the length given.
    pub fn new(element: &Type, length: usize) -> &ArrayType {
        unsafe { core::LLVMArrayType(element.into(), length as c_uint) }.into()
    }
    /// Returns the element type of this array type.
    pub fn get_element(&self) -> &Type {
        unsafe { mem::transmute(core::LLVMGetElementType(self.into())) }
    }
    /// Returns the number of elements in this vector type.
    pub fn get_length(&self) -> usize {
        unsafe { core::LLVMGetArrayLength(self.into()) as usize }
    }
}