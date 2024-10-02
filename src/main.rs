#![no_main]
#![no_std]
use core::panic::PanicInfo;

mod vga_buffer;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    println!(
        "Print line is set up! \nThis was {} times faster to make ",
        100
    );

    panic!("the fuck");
    loop {}
}
