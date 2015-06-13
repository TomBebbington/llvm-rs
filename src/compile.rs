use libc::{c_uint, c_ulonglong};
use ffi::core;
use ffi::prelude::LLVMValueRef;
use builder::Builder;
use context::Context;
use libc::c_char;
use value::Value;
use ty::Type;
use std::mem;
use std::ffi::CStr;

/// Implemented for any type that can be represeented in IR
///
/// Please note that destructors are NOT compiled and must be handled manually in the code
/// you compile using LLVM
pub trait Compile<'a> {
    /// Compile this value with the builder given in the context given
    fn compile(self, builder: &'a Builder, context: &'a Context) -> &'a Value;
    /// Get the type descriptor for this type
    fn get_type(context: &'a Context) -> &'a Type;
}
impl<'a, T> Compile<'a> for &'a T where T:Sized + Copy + Compile<'a> {
    fn compile(self, builder: &'a Builder, context: &'a Context) -> &'a Value {
        let val = (*self).compile(builder, context);
        let ptr = builder.build_alloca(T::get_type(context));
        builder.build_store(val, ptr);
        ptr
    }
    fn get_type(context: &'a Context) -> &'a Type {
        Type::new_pointer(T::get_type(context))
    }
}
macro_rules! compile_int(
    ($uty:ty, $sty:ty, $ctx:ident => $ty_ex:expr) => (
        impl<'a> Compile<'a> for $uty {
            fn compile(self, _: &'a Builder, context: &'a Context) -> &'a Value {
                unsafe { core::LLVMConstInt(Self::get_type(context).into(), self as c_ulonglong, 0) }.into()
            }
            fn get_type($ctx: &'a Context) -> &'a Type {
                let $ctx = $ctx.into();
                unsafe { $ty_ex }.into()
            }
        }
        impl<'a> Compile<'a> for $sty {
            fn compile(self, _: &'a Builder, context: &'a Context) -> &'a Value {
                unsafe { core::LLVMConstInt(Self::get_type(context).into(), self as c_ulonglong, 0) }.into()
            }
            fn get_type($ctx: &'a Context) -> &'a Type {
                let $ctx = $ctx.into();
                unsafe { $ty_ex }.into()
            }
        }
    );
    ($uty:ty, $sty:ty, $func:ident) => (
        compile_int!{$uty, $sty, ctx => core::$func(ctx)}
    );
);
impl<'a> Compile<'a> for bool {
    fn compile(self, _: &'a Builder, context: &'a Context) -> &'a Value {
        unsafe { core::LLVMConstInt(Self::get_type(context).into(), self as c_ulonglong, 0) }.into()
    }
    fn get_type(ctx: &'a Context) -> &'a Type {
        unsafe { core::LLVMInt1TypeInContext(ctx.into()) }.into()
    }
}
impl<'a> Compile<'a> for f32 {
    fn compile(self, _: &'a Builder, context: &'a Context) -> &'a Value {
        unsafe { core::LLVMConstReal(Self::get_type(context).into(), self as f64) }.into()
    }
    fn get_type(ctx: &'a Context) -> &'a Type {
        unsafe { core::LLVMFloatTypeInContext(ctx.into()) }.into()
    }
}
impl<'a> Compile<'a> for f64 {
    fn compile(self, _: &'a Builder, context: &'a Context) -> &'a Value {
        unsafe { core::LLVMConstReal(Self::get_type(context).into(), self) }.into()
    }
    fn get_type(ctx: &'a Context) -> &'a Type {
        unsafe { core::LLVMDoubleTypeInContext(ctx.into()) }.into()
    }
}
impl<'a> Compile<'a> for char {
    fn compile(self, _: &'a Builder, context: &'a Context) -> &'a Value {
        unsafe { core::LLVMConstInt(Self::get_type(context).into(), self as u32 as c_ulonglong, 0) }.into()
    }
    fn get_type(ctx: &'a Context) -> &'a Type {
        unsafe { core::LLVMInt32TypeInContext(ctx.into()) }.into()
    }
}
impl<'a> Compile<'a> for *const c_char {
    fn compile(self, _: &'a Builder, context: &'a Context) -> &'a Value {
        unsafe {
            let len = CStr::from_ptr(self).to_bytes().len();
            core::LLVMConstStringInContext(context.into(), self, len as c_uint, 0).into()
        }
    }
    fn get_type(ctx: &'a Context) -> &'a Type {
        Type::new_pointer(Type::get::<c_char>(ctx))
    }
}
impl<'a> Compile<'a> for *const str {
    fn compile(self, builder: &'a Builder, context: &'a Context) -> &'a Value {
        unsafe {
            let text:&str = mem::transmute(self);
            let ptr = text.as_ptr() as *const c_char;
            let len = text.len() as c_uint;
            let ptr = core::LLVMConstStringInContext(context.into(), ptr, len, 1).into();
            let size = text.len().compile(builder, context);
            Value::new_struct(context, &[ptr, size], true)
        }
    }
    fn get_type(ctx: &'a Context) -> &'a Type {
        let usize_t = usize::get_type(ctx);
        Type::new_struct(ctx, &[usize_t, usize_t], true)
    }
}
impl<'a, 'b> Compile<'a> for &'b str {
    fn compile(self, builder: &'a Builder, context: &'a Context) -> &'a Value {
        (self as *const str).compile(builder, context)
    }
    fn get_type(ctx: &'a Context) -> &'a Type {
        <*const str as Compile<'a>>::get_type(ctx)
    }
}
compile_int!{u8, i8, LLVMInt8TypeInContext}
compile_int!{u16, i16, LLVMInt16TypeInContext}
compile_int!{u32, i32, LLVMInt32TypeInContext}
compile_int!{u64, i64, LLVMInt64TypeInContext}
compile_int!{usize, isize, ctx => core::LLVMIntTypeInContext(ctx, mem::size_of::<isize>() as c_uint * 8)}
impl<'a> Compile<'a> for () {
    fn compile(self, _: &'a Builder, context: &'a Context) -> &'a Value {
        unsafe { core::LLVMConstNull(Self::get_type(context).into()) }.into()
    }
    fn get_type(context: &'a Context) -> &'a Type {
        unsafe { core::LLVMVoidTypeInContext(context.into()) }.into()
    }
}

macro_rules! compile_tuple(
    ($($name:ident = $oname:ident),+) => (
        impl<'a, $($name),+> Compile<'a> for ($($name),+) where $($name:Compile<'a>),+ {
            fn compile(self, builder: &'a Builder, context: &'a Context) -> &'a Value {
                let ($($oname, )+) = self;
                Value::new_struct(context, &[$($oname.compile(builder, context)),+], false)
            }
            fn get_type(context: &'a Context) -> &'a Type {
                Type::new_struct(context, &[$($name::get_type(context)),+], false)
            }
        }
    )
);
compile_tuple!{A = a, B = b}
compile_tuple!{A = a, B = b, C = c}
compile_tuple!{A = a, B = b, C = c, D = d}
compile_tuple!{A = a, B = b, C = c, D = d, E = e}
compile_tuple!{A = a, B = b, C = c, D = d, E = e, F = f}
compile_tuple!{A = a, B = b, C = c, D = d, E = e, F = f, G = g}

macro_rules! compile_array(
    ($ty:ty, $num:expr) => (
        impl<'a, T> Compile<'a> for $ty where T: Copy + Compile<'a> + 'a {
            fn compile(self, builder: &'a Builder, context: &'a Context) -> &'a Value {
                let values:Vec<_> = self.iter().map(|&value| value.compile(builder, context)).collect();
                unsafe { core::LLVMConstVector(values.as_ptr() as *mut LLVMValueRef, $num) }.into()
            }
            fn get_type(context: &'a Context) -> &'a Type {
                Type::new_vector(Type::get::<T>(context), $num)
            }
        }
    )
);
compile_array!{[T; 0], 0}
compile_array!{[T; 1], 1}
compile_array!{[T; 2], 2}
compile_array!{[T; 3], 3}
compile_array!{[T; 4], 4}
compile_array!{[T; 5], 5}
compile_array!{[T; 6], 6}

macro_rules! compile_func(
    ($($name:ident),*) => (
        impl<'a, R, $($name),*> Compile<'a> for fn($($name),*) -> R where R:Compile<'a>, $($name:Compile<'a>),* {
            fn compile(self, builder: &'a Builder, context: &'a Context) -> &'a Value {
                unsafe {
                    let as_usize: usize = mem::transmute(self);
                    let value = as_usize.compile(builder, context);
                    core::LLVMConstIntToPtr(value.into(), Self::get_type(context).into())
                }.into()
            }
            fn get_type(context: &'a Context) -> &'a Type {
                Type::new_function(R::get_type(context), &[$($name::get_type(context)),*])
            }
        }
        impl<'a, R, $($name),*> Compile<'a> for extern fn($($name),*) -> R where R:Compile<'a>, $($name:Compile<'a>),* {
            fn compile(self, builder: &'a Builder, context: &'a Context) -> &'a Value {
                unsafe {
                    let as_usize: usize = mem::transmute(self);
                    let value = as_usize.compile(builder, context);
                    core::LLVMConstIntToPtr(value.into(), Self::get_type(context).into())
                }.into()
            }
            fn get_type(context: &'a Context) -> &'a Type {
                Type::new_function(R::get_type(context), &[$($name::get_type(context)),*])
            }
        }
    )
);
compile_func!{}
compile_func!{A}
compile_func!{A, B}
compile_func!{A, B, C}
compile_func!{A, B, C, D}
compile_func!{A, B, C, D, E}
compile_func!{A, B, C, D, E, F}
compile_func!{A, B, C, D, E, F, G}

/*
impl<'a, 'b> Compile<'a> for &'b str {
    fn compile(self, builder: &'a Builder, context: &'a Context) -> &'a Value {
        unsafe {
            let text = core::LLVMConstStringInContext(context.into(), self.as_ptr() as *const i8, self.len() as c_uint, 1);
            let ptr = builder.alloca()
        }
    }
    fn get_type(context: &'a Context) -> &'a Type {
        <(*mut i8, usize) as Compile<'a>>::get_type(context)
    }
}*/
