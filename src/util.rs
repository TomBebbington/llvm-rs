use libc::c_char;
use ffi::core;
use std::ffi::{CStr, CString};
use std::{fmt, str};
use std::ops::{Deref, DerefMut, Drop};
use std::marker::PhantomData;
/// Implemented by any type represented by a pointer that can be disposed
pub trait DisposeRef {
    /// What type this reference is to
    type RefTo;
    /// Destroy the contents at the pointer's location
    unsafe fn dispose(ptr: *mut Self::RefTo);
}

/// A wrapper for C-owned unique pointers
///
/// This is necessary to allow owned and borrowed representations of C types
/// to be represented by the same type as they are in C with little overhead
pub struct CBox<'a, D:?Sized> where D:DisposeRef+'a {
    ptr: *mut D::RefTo,
    marker: PhantomData<&'a ()>
}
impl<'a, D:?Sized> CBox<'a, D> where D:DisposeRef+'a {
    #[inline(always)]
    pub fn new(ptr: *mut D::RefTo) -> Self {
        CBox {
            ptr: ptr,
            marker: PhantomData
        }
    }
}
impl<'a, D:?Sized> Drop for CBox<'a, D> where D:DisposeRef+'a {
    #[inline(always)]
    fn drop(&mut self) {
        unsafe { <D as DisposeRef>::dispose(self.ptr) }
    }
}
impl<'a, D> Deref for CBox<'a, D> where D:DisposeRef+'a, *mut D::RefTo:Into<&'a D> {
    type Target = D;
    fn deref(&self) -> &D {
        self.ptr.into()
    }
}
impl<'a, D> DerefMut for CBox<'a, D> where D:DisposeRef+'a, *mut D::RefTo:Into<&'a D>, *mut D::RefTo:Into<&'a mut D> {
    fn deref_mut(&mut self) -> &mut D {
        self.ptr.into()
    }
}
impl<'a> Deref for CBox<'a, str> {
    type Target = str;
    fn deref(&self) -> &str {
        unsafe { to_str(self.ptr) }
    }
}
impl<'a, 'b> From<&'a str> for CBox<'b, str> {
    fn from(text: &'a str) -> CBox<'b, str> {
        CBox::new(with_cstr(text, |c_text| unsafe { core::LLVMCreateMessage(c_text) }))
    }
}
impl<'a> fmt::Display for CBox<'a, str> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str(self.deref())
    }
}
impl<'a> fmt::Debug for CBox<'a, str> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str(self.deref())
    }
}
impl<'a, T> fmt::Display for CBox<'a, T> where T:fmt::Display+DisposeRef+'a, *mut T::RefTo:Into<&'a T> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(self as &T, fmt)
    }
}
impl<'a, T> fmt::Debug for CBox<'a, T> where T:fmt::Debug+DisposeRef+'a, *mut T::RefTo:Into<&'a T> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self as &T, fmt)
    }
}
impl DisposeRef for str {
    type RefTo = c_char;
    unsafe fn dispose(ptr: *mut c_char) {
        core::LLVMDisposeMessage(ptr)
    }
}

#[inline(always)]
pub fn with_cstr<C, R>(text: &str, cb: C) -> R where C:FnOnce(*const c_char) -> R {
    let c_text = CString::new(text).unwrap();
    cb(c_text.as_bytes().as_ptr() as *const c_char)
}

#[inline(always)]
pub unsafe fn to_str<'a>(text: *mut c_char) -> &'a str {
    let c_str = CStr::from_ptr(text);
    str::from_utf8_unchecked(c_str.to_bytes())
}

pub unsafe fn to_null_str<'a>(text: *mut c_char) -> Option<&'a str> {
    if text.is_null() {
        None
    } else {
        Some(to_str(text))
    }
}
pub unsafe fn ptr_to_null<P, T>(ptr: *mut P) -> Option<T> where T:From<*mut P> {
    if ptr.is_null() {
        None
    } else {
        Some(ptr.into())
    }
}
