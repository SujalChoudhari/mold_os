#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![reexport_test_harness_main = "test_main"]
#![test_runner(mold_os::test_runner)]

use core::panic::PanicInfo;
use mold_os::println;

#[no_mangle] // don't mangle the name of this function
pub extern "C" fn _start() -> ! {
    test_main();

    loop {}
}

#[allow(dead_code)]
fn test_runner(tests: &[&dyn Fn()]) {
    let _ = tests;
    unimplemented!();
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    mold_os::test_panic_handler(info)
}

#[test_case]
fn test_println() {
    println!("test_println output");
}
