#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(mold_os::test_runner)]
#![reexport_test_harness_main = "test_main"]
extern crate alloc;

use alloc::{boxed::Box, vec, vec::Vec, rc::Rc};
use bootloader::{entry_point, BootInfo};
use mold_os::{memory::translate_addr, println};

mod application;
mod panic;

entry_point!(kernel_main);
fn kernel_main(boot_info: &'static BootInfo) -> ! {
    use mold_os::allocator;
    use mold_os::memory;
    use mold_os::memory::BootInfoFrameAllocator;
    use x86_64::{structures::paging::Page, VirtAddr}; // new import

    // Start OS
    mold_os::init();
    application::start();

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = memory::init(phys_mem_offset);
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_map) };

    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("heap initialization failed");

    let heap_value = Box::new(41);
    println!("heap_value at {:p}", heap_value);

    let mut vec = Vec::new();
    for i in 0..500 {
        vec.push(i);
    }
    println!("vec at {:p}", vec.as_slice());

    // create a reference counted vector -> will be freed when count reaches 0
    let reference_counted = Rc::new(vec![1, 2, 3]);
    let cloned_reference = reference_counted.clone();
    println!("current reference count is {}", Rc::strong_count(&cloned_reference));
    core::mem::drop(reference_counted);
    println!("reference count is {} now", Rc::strong_count(&cloned_reference));


    
    // Test or/and run
    #[cfg(test)]
    test_main();
    application::run();

    // Quit OS
    application::end();
    mold_os::hlt_loop();
}
