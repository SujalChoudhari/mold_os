#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(mold_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use mold_os::{console::get_word, println, string};

#[no_mangle]
pub extern "C" fn _start() -> ! {
    mold_os::init();

    #[cfg(test)]
    test_main();
    let content: string::String = get_word();

    println!("You entered! {:?}", content);
    mold_os::hlt_loop();
}

/// This function is called on panic.
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    mold_os::hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    mold_os::test_panic_handler(info)
}
