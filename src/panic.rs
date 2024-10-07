use core::panic::PanicInfo;
use mold_os::println;

/// This function is called on panic in non-test mode.
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    mold_os::hlt_loop();
}

/// This function is called on panic in test mode.
#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    mold_os::test_panic_handler(info)
}
