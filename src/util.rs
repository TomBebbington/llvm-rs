use libc::c_char;
use std::ffi::{CStr, CString};
use std::mem;
use std::str;
/// Indicates that this structure is a substructure of another.
pub unsafe trait Sub<T>: Sized {
    /// Check if the given super value is an instance of this type.
    fn is(c: &T) -> bool;
    /// Attempt to cast the given super value into an instance of this type.
    fn from_super(c: &T) -> Option<&Self> {
        if Self::is(c) {
            Some(unsafe { mem::transmute(c) })
        } else {
            None
        }
    }
    /// Cast this value to a super value.
    fn to_super(&self) -> &T {
        unsafe { mem::transmute(self) }
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
