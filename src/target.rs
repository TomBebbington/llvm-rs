use libc::{c_char,  c_uint};
use ffi::target_machine::{self, LLVMTargetRef};
use ffi::target::{self, LLVMTargetDataRef, LLVMOpaqueTargetData};
use std::ffi::CString;
use std::fmt;
use util::{self, CBox, DisposeRef};

pub struct TargetData;
native_ref!(&TargetData = LLVMTargetDataRef);

impl TargetData {
    /// Creates target data from a target layout string
    pub fn new(rep: &str) -> CBox<TargetData> {
        let c_rep = CString::new(rep).unwrap();
        CBox::new(unsafe {
            target::LLVMCreateTargetData(c_rep.as_ptr())
        }.into())
    }
    /// Returns true if the target is big endian
    pub fn is_big_endian(&self) -> bool {
        let order = unsafe { target::LLVMByteOrder(self.into()) } as c_uint;
        order == 0
    }
    /// Returns the size of a pointer on the target
    pub fn get_pointer_size(&self) -> usize {
        unsafe { target::LLVMPointerSize(self.into()) as usize }
    }
    /// Convers this to a target layout string
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

pub struct Target;
native_ref!(&Target = LLVMTargetRef);
impl Target {
    /// Get the name of this target
    pub fn get_name(&self) -> &str {
        unsafe { util::to_str(target_machine::LLVMGetTargetName(self.into()) as *mut c_char) }
    }
    /// Get the description of this target
    pub fn get_description(&self) -> &str {
        unsafe { util::to_str(target_machine::LLVMGetTargetDescription(self.into()) as *mut c_char) }
    }
    pub fn has_asm_backend(&self) -> bool {
        unsafe { target_machine::LLVMTargetHasAsmBackend(self.into()) != 0 }
    }
    pub fn has_jit(&self) -> bool {
        unsafe { target_machine::LLVMTargetHasJIT(self.into()) != 0 }
    }
    pub fn has_target_machine(&self) -> bool {
        unsafe { target_machine::LLVMTargetHasTargetMachine(self.into()) != 0 }
    }
}
