#![no_main]
#![no_std]

#[no_mangle]
pub extern "C" fn main(
    _argc: vitasdk_sys::c_int,
    _argv: *const *const vitasdk_sys::c_char,
) -> vitasdk_sys::c_int {
    let _ = unsafe { vitasdk_sys::sceKernelDelayThread(10_000_000) };
    let _ = unsafe { vitasdk_sys::sceKernelExitProcess(0) };
    loop {}
}

#[panic_handler]
fn panic_handler(_: &core::panic::PanicInfo) -> ! {
    loop {}
}
