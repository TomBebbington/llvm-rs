use libc::{c_char, size_t};
use ffi::{core, LLVMMemoryBuffer};
use ffi::prelude::LLVMMemoryBufferRef;
use cbox::{CBox, DisposeRef};
use std::ops::Deref;
use std::marker::PhantomData;
use std::mem;
use std::ffi::CString;
use std::default::Default;
use util;

pub struct MemoryBuffer(PhantomData<[u8]>);
native_ref!(&MemoryBuffer = LLVMMemoryBufferRef);
impl MemoryBuffer {
    pub fn new_from_file(path: &str) -> Result<CBox<MemoryBuffer>, CBox<str>> {
        util::with_cstr(path, |path| unsafe {
            let mut output = mem::uninitialized();
            let mut error = mem::uninitialized();
            if core::LLVMCreateMemoryBufferWithContentsOfFile(path, &mut output, &mut error) == 1 {
                Err(CBox::new(error))
            } else {
                Ok(CBox::new(output))
            }
        })
    }

    pub fn new_from_str(buf: &str, name: Option<&str>) -> Result<CBox<MemoryBuffer>, CBox<str>> {
        unsafe {
            let in_name = name
                .map(|n| n.as_bytes().to_vec())
                .map(|mut v| { v.push(0); CString::from_vec_unchecked(v) })
                .unwrap_or(CString::default());

            let out = core::LLVMCreateMemoryBufferWithMemoryRangeCopy(
                buf.as_ptr() as *const c_char, buf.len() as size_t, in_name.as_ptr());
            Ok(CBox::new(out))
        }
    }
}
impl Deref for MemoryBuffer {
    type Target = str;
    fn deref(&self) -> &str {
        unsafe {
            #[allow(dead_code)]
            struct StrSlice {
                data: *const c_char,
                len: usize
            }
            mem::transmute(StrSlice {
                data: core::LLVMGetBufferStart(self.into()),
                len: core::LLVMGetBufferSize(self.into()) as usize
            })
        }
    }
}
impl DisposeRef for MemoryBuffer {
    type RefTo = LLVMMemoryBuffer;
    unsafe fn dispose(ptr: LLVMMemoryBufferRef) {
        core::LLVMDisposeMemoryBuffer(ptr)
    }
}
