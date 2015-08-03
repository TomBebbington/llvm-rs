macro_rules! native_ref(
    (&$name:ident = $alias:ty) => (
        impl Eq for $name {}
        impl PartialEq<$name> for $name {
            fn eq(&self, other: &$name) -> bool {
                use std::mem;
                unsafe { mem::transmute::<_, isize>(self) == mem::transmute(other) }
            }
        }
        impl<'a> PartialEq<$name> for &'a $name {
            fn eq(&self, other: &$name) -> bool {
                use std::mem;
                unsafe { mem::transmute::<_, isize>(self) == mem::transmute(other) }
            }
        }
        impl<'a> From<&'a $name> for $alias {
            fn from(ty: &'a $name) -> $alias {
                use std::mem;
                unsafe { mem::transmute(ty) }
            }
        }
        impl<'a> From<&'a mut $name> for $alias {
            fn from(ty: &'a mut $name) -> $alias {
                use std::mem;
                unsafe { mem::transmute(ty) }
            }
        }
        impl<'a> From<$alias> for &'a $name {
            fn from(ty: $alias) -> &'a $name {
                use std::mem;
                unsafe { mem::transmute(ty) }
            }
        }
        impl<'a> From<$alias> for &'a mut $name {
            fn from(ty: $alias) -> &'a mut $name {
                use std::mem;
                unsafe { mem::transmute(ty) }
            }
        }
    );
    ($name:ident, $field:ident: $pointer_ty:ty) => (
        impl<'a> From<&'a mut $name> for $pointer_ty {
            /// Convert into a native pointer
            fn from(thing: &'a mut $name) -> $pointer_ty {
                thing.$field
            }
        }
        impl<'a> From<&'a $name> for $pointer_ty {
            /// Convert into a native pointer
            fn from(thing: &'a $name) -> $pointer_ty {
                thing.$field
            }
        }
        impl From<$name> for $pointer_ty {
            /// Convert into a native pointer
            fn from(thing: $name) -> $pointer_ty {
                thing.$field
            }
        }
        impl From<$pointer_ty> for $name {
            /// Convert from a native pointer
            fn from(ptr: $pointer_ty) -> $name {
                $name {
                    $field: ptr
                }
            }
        }
    );
    ($name:ident, $field:ident: $pointer_ty:ty, $($ofield:ident = $expr:expr),*) => (
        impl<'a> From<&'a mut $name> for $pointer_ty {
            /// Convert into a native pointer
            fn from(thing: &'a mut $name) -> $pointer_ty {
                thing.$field
            }
        }
        impl<'a> From<&'a $name> for $pointer_ty {
            /// Convert into a native pointer
            fn from(thing: &'a $name) -> $pointer_ty {
                thing.$field
            }
        }
        impl From<$name> for $pointer_ty {
            /// Convert into a native pointer
            fn from(thing: $name) -> $pointer_ty {
                thing.$field
            }
        }
        impl From<$pointer_ty> for $name {
            /// Convert from a native pointer
            fn from(ptr: $pointer_ty) -> $name {
                $name {
                    $field: ptr,
                    $($ofield: $expr),*
                }
            }
        }
    );
    ($name:ident<$ty:ident>, $field:ident: $pointer_ty:ty, $($ofield:ident = $expr:expr),*) => (
        impl<'a, $ty> From<&'a mut $name<$ty>> for $pointer_ty {
            /// Convert into a native pointer
            fn from(thing: &'a mut $name<$ty>) -> $pointer_ty {
                thing.$field
            }
        }
        impl<'a, $ty> From<&'a $name<$ty>> for $pointer_ty {
            /// Convert into a native pointer
            fn from(thing: &'a $name<$ty>) -> $pointer_ty {
                thing.$field
            }
        }
        impl<$ty> From<$name<$ty>> for $pointer_ty {
            /// Convert into a native pointer
            fn from(thing: $name<$ty>) -> $pointer_ty {
                thing.$field
            }
        }
        impl<$ty> From<$pointer_ty> for $name<$ty> {
            /// Convert from a native pointer
            fn from(ptr: $pointer_ty) -> $name<$ty> {
                $name {
                    $field: ptr,
                    $($ofield: $expr),*
                }
            }
        }
    );
    (contra $name:ident, $field:ident: $pointer_ty:ty) => (
        impl<'a, 'b> From<&'a mut $name<'b>> for $pointer_ty {
            /// Convert into a native pointer
            fn from(thing: &'a mut $name<'b>) -> $pointer_ty {
                thing.$field
            }
        }
        impl<'a, 'b> From<&'a $name<'b>> for $pointer_ty {
            /// Convert into a native pointer
            fn from(thing: &'a $name<'b>) -> $pointer_ty {
                thing.$field
            }
        }
        impl<'a> From<$name<'a>> for $pointer_ty {
            /// Convert into a native pointer
            fn from(thing: $name<'a>) -> $pointer_ty {
                thing.$field
            }
        }
        impl<'a> From<$pointer_ty> for $name<'a> {
            /// Convert from a native pointer
            fn from(ptr: $pointer_ty) -> $name<'a> {
                $name {
                    $field: ptr,
                    marker: PhantomData
                }
            }
        }
    )
);
macro_rules! to_str(
    ($ty:ty, $func:ident) => (
        impl fmt::Debug for $ty {
            fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
                fmt.write_str(unsafe {
                    let c_str = core::$func(self.into());
                    util::to_str(c_str)
                })
            }
        }
    );
);
macro_rules! get_context(
    ($ty:ty, $func:ident) => (
        impl GetContext for $ty {
            fn get_context(&self) -> &Context {
                unsafe { core::$func(self.into()) }.into()
            }
        }
    );
);
macro_rules! deref(
    ($ty:ty, $to:ty) => (
        impl Deref for $ty {
            type Target = $to;
            fn deref(&self) -> &$to {
                unsafe { mem::transmute(self) }
            }
        }
    );
);
macro_rules! dispose(
    ($ty:ty, $ref_ty:ty, $func:expr) => (
        impl ::cbox::DisposeRef for $ty {
            type RefTo = $ref_ty;
            #[inline(always)]
            unsafe fn dispose(ptr: *mut $ref_ty) {
                $func(ptr)
            }
        }
    );
);
