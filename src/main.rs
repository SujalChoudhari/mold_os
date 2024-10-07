#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(mold_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

use mold_os::{console::get_word, println, string};

mod application;
mod panic;
#[no_mangle]
pub extern "C" fn _start() -> ! {
    // Start OS
    mold_os::init();
    application::start();
    
    // Test or/and run
    #[cfg(test)]
    test_main();
    application::run();

    // Quit OS
    application::end();
    mold_os::hlt_loop();
}
