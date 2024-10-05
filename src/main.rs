#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(mold_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use mold_os::println;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    println!("Hello World{}", "!");
    mold_os::init();
    x86_64::instructions::interrupts::int3(); // new

    #[cfg(test)]
    test_main();
    println!("It did not crash!");

    loop {}
}

/// This function is called on panic.
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    mold_os::test_panic_handler(info)
}
