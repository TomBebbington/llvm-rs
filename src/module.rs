use libc::c_char;
use ffi::prelude::{LLVMValueRef, LLVMModuleRef};
use ffi::{core, LLVMModule};
use ffi::bit_writer as writer;
use ffi::bit_reader as reader;
use cbox::CBox;
use std::ffi::CString;
use std::iter::{Iterator, IntoIterator};
use std::{fmt, mem};
use std::marker::PhantomData;
use buffer::MemoryBuffer;
use context::{Context, GetContext};
use value::Function;
use ty::Type;
use util;

/// Represents a translation unit
///
/// This is attached to the lifetime of the context that constructs it
pub struct Module;
native_ref!(&Module = LLVMModuleRef);
impl Module {
    /// Create a new module in the context given with the name given
    ///
    /// The lifetime of the module will match the lifetime of the context
    /// you instance it in because the context contains it
    ///
    /// ```rust
    /// use llvm::*;
    /// let context = Context::new();
    /// let module = Module::new("name", &context);
    /// println!("{:?}", module)
    /// ```
    pub fn new<'a>(name: &str, context: &'a Context) -> CBox<'a, Module> {
        let c_name = CString::new(name).unwrap();
        unsafe { CBox::new(core::LLVMModuleCreateWithNameInContext(c_name.as_ptr(), context.into())) }
    }
    /// Parse this bitcode file into a module
    pub fn parse_bitcode<'a, 'b>(context: &'a Context, path: &'b str) -> Result<CBox<'b, Module>, CBox<'b, str>> {
        unsafe {
            let mut out = mem::uninitialized();
            let mut err = mem::uninitialized();
            let buf = try!(MemoryBuffer::new_from_file(path));
            if reader::LLVMParseBitcodeInContext(context.into(), (&*buf).into(), &mut out, &mut err) == 1 {
                Err(CBox::new(err))
            } else {
                Ok(CBox::new(out))
            }
        }
    }
    /// Write this module's bitcode to the path given
    pub fn write_bitcode(&self, path: &str) {
        util::with_cstr(path, |cpath| unsafe {
            if writer::LLVMWriteBitcodeToFile(self.into(), cpath) != 0 {
                panic!("failed to write bitcode to file {}", path)
            }
        })
    }
    /// Add a function to the module with the name given
    pub fn add_function<'a>(&'a self, name: &str, sig: &'a Type) -> &'a mut Function {
        let c_name = CString::new(name).unwrap();
        unsafe { core::LLVMAddFunction(self.into(), c_name.as_ptr(), sig.into()) }.into()
    }
    /// Returns the function with the name given if it exists
    pub fn get_function<'a>(&'a self, name: &str) -> Option<&'a Function> {
        let c_name = CString::new(name).unwrap();
        unsafe {
            let ty = core::LLVMGetNamedFunction(self.into(), c_name.as_ptr());
            util::ptr_to_null(ty)
        }
    }
    /// Returns the type with the name given if it exists
    pub fn get_type<'a>(&'a self, name: &str) -> Option<&'a Type> {
        let c_name = CString::new(name).unwrap();
        unsafe {
            let ty = core::LLVMGetTypeByName(self.into(), c_name.as_ptr());
            util::ptr_to_null(ty)
        }
    }
    /// Clone this module
    pub fn clone<'a>(&'a self) -> CBox<'a, Module> {
        CBox::new(unsafe { core::LLVMCloneModule(self.into()) })
    }

    /// Get the target data of this module
    pub fn get_target(&self) -> &str {
        unsafe {
            let target = core::LLVMGetTarget(self.into());
            util::to_str(target as *mut c_char)
        }
    }

    /// Set the target data of this module
    pub fn set_target(&mut self, target: &str) {
        let c_target = CString::new(target).unwrap();
        unsafe { core::LLVMSetTarget(self.into(), c_target.as_ptr()) }
    }
}
impl<'a> IntoIterator for &'a Module {
    type Item = &'a Function;
    type IntoIter = Functions<'a>;
    /// Iterate through the functions in the module
    fn into_iter(self) -> Functions<'a> {
        Functions {
            value: unsafe { core::LLVMGetFirstFunction(self.into()) },
            marker: PhantomData
        }
    }
}
get_context!(Module, LLVMGetModuleContext);
to_str!(Module, LLVMPrintModuleToString);
dispose!(Module, LLVMModule, core::LLVMDisposeModule);
#[derive(Copy, Clone)]
pub struct Functions<'a> {
    value: LLVMValueRef,
    marker: PhantomData<&'a ()>
}
impl<'a> Iterator for Functions<'a> {
    type Item = &'a Function;
    fn next(&mut self) -> Option<&'a Function> {
        if self.value.is_null() {
            None
        } else {
            let c_next = unsafe { core::LLVMGetNextFunction(self.value) };
            self.value = c_next;
            Some(self.value.into())
        }
    }
}
