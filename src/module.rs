use libc::{c_char, c_uint};
use ffi::prelude::{LLVMValueRef, LLVMModuleRef};
use ffi::analysis::LLVMVerifierFailureAction;
use ffi::{analysis, core, linker, LLVMModule};
use ffi::transforms::pass_manager_builder as builder;
use ffi::bit_writer as writer;
use ffi::bit_reader as reader;
use cbox::{CBox, CSemiBox};
use std::ffi::CString;
use std::iter::{Iterator, IntoIterator};
use std::io::{Error, ErrorKind};
use std::io::Result as IoResult;
use std::{env, fmt, mem};
use std::marker::PhantomData;
use std::path::Path;
use std::process::Command;
use buffer::MemoryBuffer;
use context::{Context, GetContext};
use value::{Function, Value, GlobalValue};
use ty::Type;
use util;

/// Represents a single translation unit of code.
///
/// This is attached to the lifetime of the context that constructs it, but is owned by the `CSemiBox`.
pub struct Module;
native_ref!(&Module = LLVMModuleRef);
impl Module {
    /// Create a new module in the context given with the name given.
    ///
    /// The lifetime of the module will match the lifetime of the context
    /// you instance it in because the context contains it.
    ///
    /// ```rust
    /// use llvm::*;
    /// let context = Context::new();
    /// let context = context.as_semi();
    /// let module = Module::new("name", &context);
    /// println!("{:?}", module)
    /// ```
    pub fn new<'a>(name: &str, context: &'a Context) -> CSemiBox<'a, Module> {
        let c_name = CString::new(name).unwrap();
        unsafe { CSemiBox::new(core::LLVMModuleCreateWithNameInContext(c_name.as_ptr(), context.into())) }
    }
    /// Add an external global to the module with the given type and name.
    pub fn add_global<'a>(&'a self, name: &str, ty: &'a Type) -> &'a GlobalValue {
        util::with_cstr(name, |ptr| unsafe {
            core::LLVMAddGlobal(self.into(), ty.into(), ptr).into()
        })
    }
    /// Add a global in the given address space to the module with the given type and name.
    pub fn add_global_in_addr_space<'a>(&'a self, name: &str, ty: &'a Type, sp: AddressSpace) -> &'a GlobalValue {
        util::with_cstr(name, |ptr| unsafe {
            core::LLVMAddGlobalInAddressSpace(self.into(), ty.into(), ptr, sp as c_uint).into()
        })
    }
    /// Add a constant global to the module with the given type, name and value.
    pub fn add_global_constant<'a>(&'a self, name: &str, val: &'a Value) -> &'a GlobalValue {
        util::with_cstr(name, |ptr| unsafe {
            let global = core::LLVMAddGlobal(self.into(), val.get_type().into(), ptr);
            core::LLVMSetInitializer (global.into(), val.into());
            global.into()
        })
    }
    /// Get the global with the name given, or `None` if no global with that name exists.
    pub fn get_global<'a>(&'a self, name: &str) -> Option<&'a GlobalValue> {
        util::with_cstr(name, |ptr| unsafe {
            let ptr = core::LLVMGetNamedGlobal(self.into(), ptr);
            util::ptr_to_null(ptr)
        })
    }
    /// Parse this bitcode file into a module, or return an error string.
    pub fn parse_bitcode<'a>(context: &'a Context, path: &str) -> Result<CSemiBox<'a, Module>, CBox<str>> {
        unsafe {
            let mut out = mem::uninitialized();
            let mut err = mem::uninitialized();
            let buf = try!(MemoryBuffer::new_from_file(path));
            if reader::LLVMParseBitcodeInContext(context.into(), buf.as_ptr(), &mut out, &mut err) == 1 {
                Err(CBox::new(err))
            } else {
                Ok(CSemiBox::new(out))
            }
        }
    }
    /// Write this module's bitcode to the path given.
    pub fn write_bitcode(&self, path: &str) -> IoResult<()> {
        util::with_cstr(path, |cpath| unsafe {
            if writer::LLVMWriteBitcodeToFile(self.into(), cpath) != 0 {
                Err(Error::new(ErrorKind::Other, &format!("could not write to {}", path) as &str))
            } else {
                Ok(())
            }
        })
    }
    /// Add a function to the module with the name given.
    pub fn add_function<'a>(&'a self, name: &str, sig: &'a Type) -> &'a mut Function {
        let c_name = CString::new(name).unwrap();
        unsafe { core::LLVMAddFunction(self.into(), c_name.as_ptr(), sig.into()) }.into()
    }
    /// Returns the function with the name given if it exists.
    pub fn get_function<'a>(&'a self, name: &str) -> Option<&'a Function> {
        let c_name = CString::new(name).unwrap();
        unsafe {
            let ty = core::LLVMGetNamedFunction(self.into(), c_name.as_ptr());
            util::ptr_to_null(ty)
        }
    }
    /// Returns the type with the name given if it exists.
    pub fn get_type<'a>(&'a self, name: &str) -> Option<&'a Type> {
        let c_name = CString::new(name).unwrap();
        unsafe {
            let ty = core::LLVMGetTypeByName(self.into(), c_name.as_ptr());
            util::ptr_to_null(ty)
        }
    }
    /// Clone this module.
    pub fn clone<'a>(&'a self) -> CSemiBox<'a, Module> {
        CSemiBox::new(unsafe { core::LLVMCloneModule(self.into()) })
    }

    /// Optimize this module with the given optimization level and size level.
    pub fn optimize(&self, opt_level: usize, size_level: usize) {
        unsafe {
            let builder = builder::LLVMPassManagerBuilderCreate();
            builder::LLVMPassManagerBuilderSetOptLevel(builder, opt_level as c_uint);
            builder::LLVMPassManagerBuilderSetSizeLevel(builder, size_level as c_uint);
            let pass_manager = core::LLVMCreatePassManager();
            builder::LLVMPassManagerBuilderPopulateModulePassManager(builder, pass_manager);
            builder::LLVMPassManagerBuilderDispose(builder);
            core::LLVMRunPassManager(pass_manager, self.into());
        }
    }

    /// Returns the target data of this module represented as a string
    pub fn get_target(&self) -> &str {
        unsafe {
            let target = core::LLVMGetTarget(self.into());
            util::to_str(target as *mut c_char)
        }
    }

    /// Set the target data of this module
    pub fn set_target(&self, target: &str) {
        let c_target = CString::new(target).unwrap();
        unsafe { core::LLVMSetTarget(self.into(), c_target.as_ptr()) }
    }

    /// Verify that the module is safe to run.
    pub fn verify(&self) -> Result<(), CBox<str>> {
        unsafe {
            let mut error = mem::uninitialized();
            let action = LLVMVerifierFailureAction::LLVMReturnStatusAction;
            if analysis::LLVMVerifyModule(self.into(), action, &mut error) == 1 {
                Err(CBox::new(error))
            } else {
                Ok(())
            }
        }
    }

    /// Compile the module into an object file at the given location.
    ///
    /// Note that this uses the LLVM tool `llc` to do this, which may or may not be
    /// installed on the user's machine.
    pub fn compile(&self, path: &Path, opt_level: usize) -> IoResult<()> {
        let dir = env::temp_dir();
        let path = path.to_str().unwrap();
        let mod_path = dir.join("module.bc");
        let mod_path = mod_path.to_str().unwrap();
        try!(self.write_bitcode(mod_path));
        Command::new("llc")
            .arg(&format!("-O={}", opt_level))
            .arg("-filetype=obj")
            .arg("-o").arg(path)
            .arg(mod_path)
            .spawn()
            .map(|_| ())
    }

    /// Link a module into this module
    pub fn link(&self, src: &Module) -> Result<(), CBox<str>> {
        unsafe {
            let dest = self.into();
            let src = src.into();
            let mut message = mem::uninitialized();
            if linker::LLVMLinkModules(dest, src, linker::LLVMLinkerMode::LLVMLinkerPreserveSource, &mut message) == 1 {
                Err(CBox::new(message))
            } else {
                Ok(())
            }
        }
    }

    /// Link a module into this module
    pub fn link_destroy(&self, src: CSemiBox<Module>) -> Result<(), CBox<str>> {
        unsafe {
            let dest = self.into();
            let src = src.as_ptr();
            let mut message = mem::uninitialized();
            if linker::LLVMLinkModules(dest, src, linker::LLVMLinkerMode::LLVMLinkerDestroySource, &mut message) == 1 {
                Err(CBox::new(message))
            } else {
                Ok(())
            }
        }
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
/// An iterator through the functions contained in a module.
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

#[repr(C)]
#[derive(Copy, Clone)]
pub enum AddressSpace {
    Generic = 0,
    Global = 1,
    Shared = 3,
    Const = 4,
    Local = 5,
}
