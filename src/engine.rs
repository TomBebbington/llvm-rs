use libc::{c_int, c_uint, c_ulonglong, size_t};
use ffi::{core, target};
use ffi::execution_engine as engine;
use ffi::execution_engine::*;
use ffi::target_machine::LLVMCodeModel;
use cbox::CBox;
use std::marker::PhantomData;
use std::{mem, ptr};
use std::ops::*;
use compile::Compile;
use context::Context;
use module::Module;
use util;
use value::{Function, Value};

/// Runs the module
pub trait ExecutionEngine<'a, 'b:'a> where LLVMExecutionEngineRef:From<&'b Self> {
    /// The options given to this upon creation
    type Options : Copy;
    /// Create a new execution engine with the given `Module` and options, or return a
    /// description of the error
    fn new(module: &'a Module, options: Self::Options) -> Result<Self, CBox<'a, str>>;

    /// Add a module to the list of modules
    fn add_module(&'b self, module: &'a Module) {
        unsafe { engine::LLVMAddModule(self.into(), (&*module).into()) }
    }
    /// Remove a module from the list of modules
    fn remove_module(&'b self, module: &'a Module) -> &'a Module {
        unsafe {
            let mut out = mem::uninitialized();
            engine::LLVMRemoveModule(self.into(), module.into(), &mut out, ptr::null_mut());
            out.into()
        }
    }
    /// Execute all of the static constructors for this program
    fn run_static_constructors(&'b self) {
        unsafe { engine::LLVMRunStaticConstructors(self.into()) }
    }
    /// Execute all of the static destructors for this program
    fn run_static_destructors(&'b self) {
        unsafe { engine::LLVMRunStaticDestructors(self.into()) }
    }

    fn find_function(&'b self, name: &str) -> Option<&'a Function> {
        util::with_cstr(name, |c_name| unsafe {
            let mut out = mem::zeroed();
            engine::LLVMFindFunction(self.into(), c_name, &mut out);
            mem::transmute(out)
        })
    }

    /// Get a pointer to the global value given
    unsafe fn get_pointer<T>(&'b self, global: &'a Value) -> &'b T {
        mem::transmute(engine::LLVMGetPointerToGlobal(self.into(), global.into()))
    }
}

/// The options to pass to the MCJIT backend
#[derive(Copy, Clone)]
pub struct JitOptions {
    /// The degree to which optimisations should be done, between 0 and 3
    pub opt_level: usize
}
/// The MCJIT backend, which compiles functions into machine code
pub struct JitEngine<'a> {
    engine: LLVMExecutionEngineRef,
    marker: PhantomData<&'a ()>
}
native_ref!{contra JitEngine, engine: LLVMExecutionEngineRef}
impl<'a, 'b> JitEngine<'a> {
    /// Run `cb` with the machine code for the `function`
    pub fn with_function<C, A, R>(&self, function: &'b Function, cb: C) where C:FnOnce(extern fn(A) -> R) {
        unsafe {
            let ptr:&u8 = self.get_pointer(function);
            cb(mem::transmute(ptr));
        }
    }
}
impl<'a, 'b:'a> ExecutionEngine<'a, 'b> for JitEngine<'a> {
    type Options = JitOptions;
    fn new(module: &'a Module, options: JitOptions) -> Result<JitEngine<'a>, CBox<'a, str>> {
        unsafe {
            let mut ee = mem::uninitialized();
            let mut out = mem::zeroed();
            engine::LLVMLinkInMCJIT();
            if target::LLVM_InitializeNativeTarget() == 1 {
                return Err("failed to initialize native target".into())
            }
            if target::LLVM_InitializeNativeAsmPrinter() == 1 {
                return Err("failed to initialize native asm printer".into())
            }
            let mut options = LLVMMCJITCompilerOptions {
                OptLevel: options.opt_level as c_uint,
                CodeModel: LLVMCodeModel::LLVMCodeModelDefault,
                NoFramePointerElim: 0,
                EnableFastISel: 1,
                MCJMM: ptr::null_mut()
            };
            let size = mem::size_of::<LLVMMCJITCompilerOptions>();
            let result = engine::LLVMCreateMCJITCompilerForModule(&mut ee, (&*module).into(), &mut options, size as size_t, &mut out);
            if result == 0 {
                Ok(ee.into())
            } else {
                Err(CBox::new(out))
            }
        }
    }
}
/// The interpreter backend
pub struct Interpreter<'a> {
    engine: LLVMExecutionEngineRef,
    marker: PhantomData<&'a ()>
}
native_ref!{contra Interpreter, engine: LLVMExecutionEngineRef}
impl<'a> Interpreter<'a> {
    /// Run `function` with the `args` given
    pub fn run_function(&self, function: &'a Function, args: &[GenericValue<'a>]) -> GenericValue<'a> {
        let ptr = args.as_ptr() as *mut LLVMGenericValueRef;
        unsafe { engine::LLVMRunFunction(self.into(), function.into(), args.len() as c_uint, ptr).into() }
    }
}
impl<'a, 'b:'a> ExecutionEngine<'a, 'b> for Interpreter<'a> {
    type Options = ();
    fn new(module: &'a Module, _: ()) -> Result<Interpreter<'a>, CBox<'a, str>> {
        unsafe {
            let mut ee = mem::uninitialized();
            let mut out = mem::zeroed();
            engine::LLVMLinkInInterpreter();
            let result = engine::LLVMCreateInterpreterForModule(&mut ee, (&*module).into(), &mut out);
            if result == 0 {
                Ok(ee.into())
            } else {
                Err(CBox::new(out))
            }
        }
    }
}
/// A wrapped value that can be passed to an interpreter function or returned from one
pub struct GenericValue<'a> {
    value: LLVMGenericValueRef,
    marker: PhantomData<&'a ()>
}
native_ref!(contra GenericValue, value: LLVMGenericValueRef);
impl<'a> Drop for GenericValue<'a> {
    fn drop(&mut self) {
        unsafe {
            engine::LLVMDisposeGenericValue(self.value)
        }
    }
}

/// A value that can be cast into a `GenericValue` and that a generic value can be cast into
pub trait GenericValueCast<'a> {
    /// Create a `GenericValue` from this value
    fn to_generic(self, context: &'a Context) -> GenericValue<'a>;
    /// Convert the `GenericValue` into a value of this type again
    fn from_generic(value: GenericValue<'a>, context: &'a Context) -> Self;
}

impl<'a> GenericValueCast<'a> for f64 {
    fn to_generic(self, ctx: &'a Context) -> GenericValue<'a> {
        unsafe {
            let ty = core::LLVMDoubleTypeInContext(ctx.into());
            engine::LLVMCreateGenericValueOfFloat(ty, self).into()
        }
    }
    fn from_generic(value: GenericValue<'a>, ctx: &'a Context) -> f64 {
        unsafe {
            let ty = core::LLVMDoubleTypeInContext(ctx.into());
            engine::LLVMGenericValueToFloat(ty, value.into())
        }
    }
}
impl<'a> GenericValueCast<'a> for f32 {
    fn to_generic(self, ctx: &'a Context) -> GenericValue<'a> {
        unsafe {
            let ty = core::LLVMFloatTypeInContext(ctx.into());
            engine::LLVMCreateGenericValueOfFloat(ty, self as f64).into()
        }
    }
    fn from_generic(value: GenericValue<'a>, ctx: &'a Context) -> f32 {
        unsafe {
            let ty = core::LLVMFloatTypeInContext(ctx.into());
            engine::LLVMGenericValueToFloat(ty, value.into()) as f32
        }
    }
}
macro_rules! generic_int(
    ($ty:ty, $signed:expr) => (
        impl<'a> GenericValueCast<'a> for $ty {
            fn to_generic(self, ctx: &'a Context) -> GenericValue<'a> {
                unsafe {
                    let ty = <Self as Compile<'a>>::get_type(ctx);
                    engine::LLVMCreateGenericValueOfInt(ty.into(), self as c_ulonglong, $signed as c_int).into()
                }
            }
            fn from_generic(value: GenericValue<'a>, _: &'a Context) -> $ty {
                unsafe {
                    engine::LLVMGenericValueToInt(value.into(), $signed as c_int) as $ty
                }
            }
        }
    );
    (some $signed:ty, $unsigned:ty) => (
        generic_int!{$signed, true}
        generic_int!{$unsigned, false}
    );
);

impl<'a> GenericValueCast<'a> for bool {
    fn to_generic(self, ctx: &'a Context) -> GenericValue<'a> {
        unsafe {
            let ty = <Self as Compile<'a>>::get_type(ctx);
            engine::LLVMCreateGenericValueOfInt(ty.into(), self as c_ulonglong, 0).into()
        }
    }
    fn from_generic(value: GenericValue<'a>, _: &'a Context) -> bool {
        unsafe {
            engine::LLVMGenericValueToInt(value.into(), 0) != 0
        }
    }
}
generic_int!{some i8, u8}
generic_int!{some i16, u16}
generic_int!{some i32, u32}
generic_int!{some i64, u64}
