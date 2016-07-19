use libc::{c_char,  c_uint};
use ffi::target_machine::{self, LLVMTargetRef};
use ffi::target::{self, LLVMTargetDataRef, LLVMOpaqueTargetData};
use cbox::{CBox, DisposeRef};
use std::ffi::CString;
use std::fmt;
use std::marker::PhantomData;
use types::Type;
use util;

/// Represents an LLVM Target
pub struct TargetData(PhantomData<[u8]>);
native_ref!(&TargetData = LLVMTargetDataRef);

impl TargetData {
    /// Create a target data from a target layout string.
    pub fn new(rep: &str) -> CBox<TargetData> {
        let c_rep = CString::new(rep).unwrap();
        CBox::new(unsafe {
            target::LLVMCreateTargetData(c_rep.as_ptr())
        })
    }
    /// Returns true if the target is big endian.
    pub fn is_big_endian(&self) -> bool {
        let order = unsafe { target::LLVMByteOrder(self.into()) } as c_uint;
        order == 0
    }
    /// Returns the size of a pointer on the target.
    pub fn get_pointer_size(&self) -> usize {
        unsafe { target::LLVMPointerSize(self.into()) as usize }
    }
    /// Returns the size of the type given in bits.
    pub fn size_of_in_bits(&self, ty: &Type) -> u64 {
        unsafe { target::LLVMSizeOfTypeInBits(self.into(), ty.into()) }
    }
    /// Returns the size of the type given in bytes.
    pub fn size_of(&self, ty: &Type) -> u64 {
        unsafe { target::LLVMStoreSizeOfType(self.into(), ty.into()) }
    }
    /// Returns the alignment of the type given in bytes.
    pub fn alignment_of(&self, ty: &Type) -> usize {
        unsafe { target::LLVMABIAlignmentOfType(self.into(), ty.into()) as usize }
    }
    /// Computes the structure element that contains the byte offset for a target.
    pub fn element_at(&self, struct_ty: &Type, offset: u64) -> usize {
        unsafe { target::LLVMElementAtOffset(self.into(), struct_ty.into(), offset) as usize }
    }
    /// Compute the byte offset of an element in the struct type given.
    pub fn offset_of(&self, struct_ty: &Type, element: usize) -> u64 {
        unsafe { target::LLVMOffsetOfElement(self.into(), struct_ty.into(), element as c_uint) }
    }
    /// Returns the string representation of this target data.
    pub fn as_str(&self) -> CBox<str> {
        unsafe {
            CBox::new(target::LLVMCopyStringRepOfTargetData(self.into()))
        }
    }
}
impl fmt::Display for TargetData {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str(&self.as_str())
    }
}

impl DisposeRef for TargetData {
    type RefTo = LLVMOpaqueTargetData;
    unsafe fn dispose(ptr: LLVMTargetDataRef) {
        target::LLVMDisposeTargetData(ptr)
    }
}

pub struct Target(PhantomData<[u8]>);
native_ref!(&Target = LLVMTargetRef);
impl Target {
    /// Returns the name of this target.
    pub fn get_name(&self) -> &str {
        unsafe { util::to_str(target_machine::LLVMGetTargetName(self.into()) as *mut c_char) }
    }
    /// Returns the description of this target.
    pub fn get_description(&self) -> &str {
        unsafe { util::to_str(target_machine::LLVMGetTargetDescription(self.into()) as *mut c_char) }
    }
    /// Returns true if this target has an assembly generation backend implemented.
    pub fn has_asm_backend(&self) -> bool {
        unsafe { target_machine::LLVMTargetHasAsmBackend(self.into()) != 0 }
    }
    /// Returns true if this target supports JIT compilation.
    pub fn has_jit(&self) -> bool {
        unsafe { target_machine::LLVMTargetHasJIT(self.into()) != 0 }
    }
    /// Returns true if this target has a target machine.
    pub fn has_target_machine(&self) -> bool {
        unsafe { target_machine::LLVMTargetHasTargetMachine(self.into()) != 0 }
    }
}
