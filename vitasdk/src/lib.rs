#![no_std]
#![allow(nonstandard_style)]

pub type c_int = i32;
pub type c_uint = u32;
pub type c_char = i8;
pub type SceUInt = u32;

#[link(name = "SceLibKernel_stub", kind = "static")]
extern "C" {
    pub fn sceKernelExitProcess(exit_code: c_int) -> c_int;
}

#[link(name = "SceKernelThreadMgr_stub", kind = "static")]
extern "C" {
    pub fn sceKernelDelayThread(delay: SceUInt) -> c_int;
}
