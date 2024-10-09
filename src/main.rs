#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(mold_os::test_runner)]
#![reexport_test_harness_main = "test_main"]
extern crate alloc;

use bootloader::{entry_point, BootInfo};
mod application;
mod panic;
mod vga_buffer;
entry_point!(kernel_main);
fn kernel_main(boot_info: &'static BootInfo) -> ! {
    // Start OS
    mold_os::init();
    application::start(boot_info);

    // Test or/and run
    #[cfg(test)]
    test_main();
    application::run();

    // Quit OS
    application::end();
    mold_os::hlt_loop();
}
