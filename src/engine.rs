use libc::{c_int, c_uint, c_ulonglong};
use ffi::{core, target};
use ffi::execution_engine as engine;
use ffi::execution_engine::*;
use ffi::target_machine::LLVMCodeModel;
use cbox::{CBox, CSemiBox, DisposeRef};
use std::marker::PhantomData;
use std::{mem, ptr};
use compile::Compile;
use context::{Context, GetContext};
use module::Module;
use types::{StructType, Type};
use util::{self, Sub};
use value::{Function, Value};

/// An abstract interface for implementation execution of LLVM modules.
///
/// This is designed to support both interpreter and just-in-time (JIT) compiler implementations.
pub trait ExecutionEngine<'a>:'a + Sized + DisposeRef where LLVMExecutionEngineRef: From<&'a Self> {
    /// The options given to the engine upon creation.
    type Options : Copy;
    /// Create a new execution engine with the given `Module` and optiions, or return a
    /// description of the error.
    fn new(module: &'a Module, options: Self::Options) -> Result<CSemiBox<'a, Self>, CBox<str>>;

    /// Add a module to the list of modules to interpret or compile.
    fn add_module(&'a self, module: &'a Module) {
        unsafe { engine::LLVMAddModule(self.into(), (&*module).into()) }
    }
    /// Remove a module from the list of modules to interpret or compile.
    fn remove_module(&'a self, module: &'a Module) -> &'a Module {
        unsafe {
            let mut out = mem::uninitialized();
            engine::LLVMRemoveModule(self.into(), module.into(), &mut out, ptr::null_mut());
            out.into()
        }
    }
    /// Execute all of the static constructors for this program.
    fn run_static_constructors(&'a self) {
        unsafe { engine::LLVMRunStaticConstructors(self.into()) }
    }
    /// Execute all of the static destructors for this program.
    fn run_static_destructors(&'a self) {
        unsafe { engine::LLVMRunStaticDestructors(self.into()) }
    }
    /// Attempt to find a function with the name given, or `None` if there wasn't
    /// a function with that name.
    fn find_function(&'a self, name: &str) -> Option<&'a Function> {
        util::with_cstr(name, |c_name| unsafe {
            let mut out = mem::zeroed();
            engine::LLVMFindFunction(self.into(), c_name, &mut out);
            mem::transmute(out)
        })
    }
    /// Run `function` with the arguments given as ``GenericValue`s, then return the result as one.
    ///
    /// Note that if this engine is a `JitEngine`, it only supports a small fraction of combinations
    /// for the arguments and return value, so be warned.
    ///
    /// To convert the arguments to `GenericValue`s, you should use the `GenericValueCast::to_generic` method.
    /// To convert the return value from a `GenericValue`, you should use the `GenericValueCast::from_generic` method.
    fn run_function(&'a self, function: &'a Function, args: &[&'a GenericValue]) -> &'a GenericValue {
        let ptr = args.as_ptr() as *mut LLVMGenericValueRef;
        unsafe { engine::LLVMRunFunction(self.into(), function.into(), args.len() as c_uint, ptr).into() }
    }
    /// Returns a pointer to the global value given.
    ///
    /// This is marked as unsafe because the type cannot be guranteed to be the same as the
    /// type of the global value at this point.
    unsafe fn get_global<T>(&'a self, global: &'a Value) -> &'a T {
        mem::transmute(engine::LLVMGetPointerToGlobal(self.into(), global.into()))
    }
    /// Returns a pointer to the global value with the name given.
    ///
    /// This is marked as unsafe because the type cannot be guranteed to be the same as the
    /// type of the global value at this point.
    unsafe fn find_global<T>(&'a self, name: &str) -> Option<&'a T> {
        util::with_cstr(name, |ptr|
            mem::transmute(engine::LLVMGetGlobalValueAddress(self.into(), ptr))
        )
    }
}

/// The options to pass to the MCJIT backend.
#[derive(Copy, Clone)]
pub struct JitOptions {
    /// The degree to which optimizations should be done, between 0 and 3.
    ///
    /// 0 represents no optimizations, 3 represents maximum optimization
    pub opt_level: usize
}
/// The MCJIT backend, which compiles functions and values into machine code.
pub struct JitEngine(PhantomData<[u8]>);
native_ref!{&JitEngine = LLVMExecutionEngineRef}
dispose!{JitEngine, LLVMOpaqueExecutionEngine, LLVMDisposeExecutionEngine}
impl<'a> JitEngine {
    /// Run the closure `cb` with the machine code for the function `function`.
    ///
    /// If the function takes multiple arguments, these should be wrapped in a tuple due to 
    /// the limitations of Rust's type system.
    ///
    /// This will check that the types match at runtime when in debug mode, but not release mode.
    /// You should make sure to use debug mode if you want it to error when the types don't match.
    pub fn with_function<C, A, R>(&self, function: &'a Function, cb: C) where A:Compile<'a>, R:Compile<'a>, C:FnOnce(extern "C" fn (A) -> R) {
        if cfg!(debug_assertions) {
            let ctx = function.get_context();
            let sig = function.get_signature();
            assert_eq!(Type::get::<R>(ctx), sig.get_return());
            let arg = Type::get::<A>(ctx);
            assert_eq!(sig.get_params(), if let Some(args) = StructType::from_super(arg) {
                args.get_elements()
            } else {
                vec![arg]
            });
        }
        unsafe {
            cb(self.get_function::<A, R>(function));
        }
    }
    /// Run the closure `cb` with the machine code for the function `function`.
    pub unsafe fn with_function_unchecked<C, A, R>(&self, function: &'a Function, cb: C) where A:Compile<'a>, R:Compile<'a>, C:FnOnce(extern fn(A) -> R) {
        cb(self.get_function::<A, R>(function));
    }
    /// Returns a pointer to the machine code for the function `function`.
    ///
    /// This is marked as unsafe because the types given as arguments and return could be different
    /// from their internal representation.
    pub unsafe fn get_function<A, R>(&self, function: &'a Function) -> extern fn(A) -> R {
        let ptr:&u8 = self.get_global(function);
        mem::transmute(ptr)
    }
}
impl<'a> ExecutionEngine<'a> for JitEngine {
    type Options = JitOptions;
    fn new(module: &'a Module, options: JitOptions) -> Result<CSemiBox<'a, JitEngine>, CBox<str>> {
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
            let result = engine::LLVMCreateMCJITCompilerForModule(&mut ee, (&*module).into(), &mut options, size, &mut out);
            if result == 0 {
                Ok(ee.into())
            } else {
                Err(CBox::new(out))
            }
        }
    }
}
/// The interpreter backend
pub struct Interpreter(PhantomData<[u8]>);
native_ref!{&Interpreter = LLVMExecutionEngineRef}
dispose!{Interpreter, LLVMOpaqueExecutionEngine, LLVMDisposeExecutionEngine}
impl<'a> ExecutionEngine<'a> for Interpreter {
    type Options = ();
    fn new(module: &'a Module, _: ()) -> Result<CSemiBox<'a, Interpreter>, CBox<str>> {
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
/// A wrapped value that can be passed to an interpreted function or returned from one
pub struct GenericValue(PhantomData<[u8]>);
native_ref!{&GenericValue = LLVMGenericValueRef}
dispose!{GenericValue, LLVMOpaqueGenericValue, LLVMDisposeGenericValue}

/// A value that can be cast into a `GenericValue` and that a `GenericValue` can be cast into.
///
/// Both these methods require contexts because some `Type` constructors are needed for the
/// conversion and these constructors need a context.
pub trait GenericValueCast {
    /// Create a `GenericValue` from this value.
    fn to_generic(self, context: &Context) -> CSemiBox<GenericValue>;
    /// Convert the `GenericValue` into a value of this type again.
    fn from_generic(value: &GenericValue, context: &Context) -> Self;
}

impl GenericValueCast for f64 {
    fn to_generic(self, ctx: &Context) -> CSemiBox<GenericValue> {
        unsafe {
            let ty = core::LLVMDoubleTypeInContext(ctx.into());
            CSemiBox::new(engine::LLVMCreateGenericValueOfFloat(ty, self))
        }
    }
    fn from_generic(value: &GenericValue, ctx: &Context) -> f64 {
        unsafe {
            let ty = core::LLVMDoubleTypeInContext(ctx.into());
            engine::LLVMGenericValueToFloat(ty, value.into())
        }
    }
}
impl GenericValueCast for f32 {
    fn to_generic(self, ctx: &Context) -> CSemiBox<GenericValue> {
        unsafe {
            let ty = core::LLVMFloatTypeInContext(ctx.into());
            CSemiBox::new(engine::LLVMCreateGenericValueOfFloat(ty, self as f64))
        }
    }
    fn from_generic(value: &GenericValue, ctx: &Context) -> f32 {
        unsafe {
            let ty = core::LLVMFloatTypeInContext(ctx.into());
            engine::LLVMGenericValueToFloat(ty, value.into()) as f32
        }
    }
}
macro_rules! generic_int(
    ($ty:ty, $signed:expr) => (
        impl GenericValueCast for $ty {
            fn to_generic(self, ctx: &Context) -> CSemiBox<GenericValue> {
                unsafe {
                    let ty = <Self as Compile>::get_type(ctx);
                    CSemiBox::new(engine::LLVMCreateGenericValueOfInt(ty.into(), self as c_ulonglong, $signed as c_int))
                }
            }
            fn from_generic(value: &GenericValue, _: &Context) -> $ty {
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

impl GenericValueCast for bool {
    fn to_generic(self, ctx: &Context) -> CSemiBox<GenericValue> {
        unsafe {
            let ty = <Self as Compile>::get_type(ctx);
            CSemiBox::new(engine::LLVMCreateGenericValueOfInt(ty.into(), self as c_ulonglong, 0))
        }
    }
    fn from_generic(value: &GenericValue, _: &Context) -> bool {
        unsafe {
            engine::LLVMGenericValueToInt(value.into(), 0) != 0
        }
    }
}
generic_int!{some i8, u8}
generic_int!{some i16, u16}
generic_int!{some i32, u32}
generic_int!{some i64, u64}
generic_int!{some isize, usize}
