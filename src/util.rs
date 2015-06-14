use libc::c_char;
use std::ffi::{CStr, CString};
use std::str;

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
