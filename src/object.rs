use libc::c_void;
use ffi::object::{self,  LLVMObjectFileRef, LLVMSectionIteratorRef, LLVMSymbolIteratorRef};
use cbox::CBox;
use std::ffi::CStr;
use std::fmt;
use std::iter::{IntoIterator, Iterator};
use std::marker::PhantomData;
use std::mem;
use buffer::MemoryBuffer;
use util;

pub struct ObjectFile {
    obj: LLVMObjectFileRef
}
native_ref!(ObjectFile, obj: LLVMObjectFileRef);
impl ObjectFile {
    pub fn read(path: &str) -> Result<ObjectFile, CBox<str>> {
        let buf = try!(MemoryBuffer::new_from_file(path));
        unsafe {
            let ptr = object::LLVMCreateObjectFile(buf.as_ptr());
            if ptr.is_null() {
                Err(CBox::from("unknown error"))
            } else {
                Ok(ptr.into())
            }
        }
    }
    pub fn symbols(&self) -> Symbols {
        Symbols {
            iter: unsafe { object::LLVMGetSymbols(self.obj) },
            marker: PhantomData
        }
    }
}
pub struct Symbols<'a> {
    iter: LLVMSymbolIteratorRef,
    marker: PhantomData<&'a ()>
}
impl<'a> Iterator for Symbols<'a> {
    type Item = Symbol<'a>;
    fn next(&mut self) -> Option<Symbol<'a>> {
        unsafe {
            let name = util::to_str(object::LLVMGetSymbolName(self.iter) as *mut i8);
            let size = object::LLVMGetSymbolSize(self.iter) as usize;
            let address = object::LLVMGetSymbolAddress(self.iter) as usize;
            Some(Symbol {
                name: name,
                address: mem::transmute(address),
                size: size
            })
        }
    }
}
impl<'a> Drop for Symbols<'a> {
    fn drop(&mut self) {
        unsafe {
            object::LLVMDisposeSymbolIterator(self.iter)
        }
    }
}

pub struct Symbol<'a> {
    pub name: &'a str,
    pub address: *const c_void,
    pub size: usize
}
impl<'a> Copy for Symbol<'a> {}
impl<'a> Clone for Symbol<'a> {
    fn clone(&self) -> Symbol<'a> {
        *self
    }
}
impl<'a> fmt::Debug for Symbol<'a> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{} - {}", self.name, self.size)
    }
}
impl<'a> Symbol<'a> {
    /// Get the pointer for this symbol
    pub unsafe fn get<T>(self) -> &'a T {
        mem::transmute(self.address)
    }
}
