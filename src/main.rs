#![no_main]
#![no_std]

#[no_mangle]
pub extern "C" fn main(
    _argc: vitasdk::c_int,
    _argv: *const *const vitasdk::c_char,
) -> vitasdk::c_int {
    let _ = unsafe { vitasdk::sceKernelDelayThread(10_000_000) };
    let _ = unsafe { vitasdk::sceKernelExitProcess(0) };
    loop {}
}

#[panic_handler]
fn panic_handler(_: &core::panic::PanicInfo) -> ! {
    loop {}
}
