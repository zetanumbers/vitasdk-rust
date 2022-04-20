#![no_main]
#![no_std]

mod types {
    #![allow(non_camel_case_types)]

    pub type c_int = i32;
    pub type c_uint = u32;
    pub type c_char = i8;
    pub type SceUInt = u32;
}
use types::*;

#[link(name = "SceLibKernel_stub", kind = "static")]
extern "C" {
    fn sceKernelExitProcess(exit_code: c_int) -> c_int;
}

#[link(name = "SceKernelThreadMgr_stub", kind = "static")]
extern "C" {
    fn sceKernelDelayThread(delay: SceUInt) -> c_int;
}

#[no_mangle]
pub extern "C" fn _start(_args: c_uint, _argp: *const c_char) -> isize {
    let _ = unsafe { sceKernelDelayThread(10_000_000) };
    let _ = unsafe { sceKernelExitProcess(0) };
    loop {}
}

#[panic_handler]
fn panic_handler(_: &core::panic::PanicInfo) -> ! {
    loop {}
}
