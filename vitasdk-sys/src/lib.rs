#![no_std]
#![allow(nonstandard_style)]

mod ctypes {
    pub use ::core::ffi::c_void;

    pub type NonZero_c_ptrdiff_t = ::core::num::NonZeroI32;
    pub type NonZero_c_size_t = ::core::num::NonZeroI32;
    pub type NonZero_c_ssize_t = ::core::num::NonZeroI32;
    pub type NonZero_c_char = ::core::num::NonZeroU8;
    pub type NonZero_c_int = ::core::num::NonZeroI32;
    pub type NonZero_c_long = ::core::num::NonZeroI32;
    pub type NonZero_c_longlong = ::core::num::NonZeroI64;
    pub type NonZero_c_schar = ::core::num::NonZeroI8;
    pub type NonZero_c_short = ::core::num::NonZeroI32;
    pub type NonZero_c_uchar = ::core::num::NonZeroU8;
    pub type NonZero_c_uint = ::core::num::NonZeroU32;
    pub type NonZero_c_ulong = ::core::num::NonZeroU32;
    pub type NonZero_c_ulonglong = ::core::num::NonZeroU64;
    pub type NonZero_c_ushort = ::core::num::NonZeroU16;

    pub type c_ptrdiff_t = i32;
    pub type c_size_t = i32;
    pub type c_ssize_t = i32;
    pub type c_char = u8;
    pub type c_double = f64;
    pub type c_float = f32;
    pub type c_int = i32;
    pub type c_long = i32;
    pub type c_longlong = i64;
    pub type c_schar = i8;
    pub type c_short = i32;
    pub type c_uchar = u8;
    pub type c_uint = u32;
    pub type c_ulong = u32;
    pub type c_ulonglong = u64;
    pub type c_ushort = u16;
}

pub use ctypes::*;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
