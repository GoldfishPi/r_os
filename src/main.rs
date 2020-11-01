#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(r_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use bootloader::{BootInfo, entry_point};

use r_os::println;

entry_point!(kernal_main);

fn kernal_main(boot_info:&'static BootInfo) -> ! {
    println!("Hello World!");

    r_os::init();

    use x86_64::registers::control::Cr3;

    let (level_4_page_table, _) = Cr3::read();
    println!("Level 4 page table at {:?}", level_4_page_table.start_address());

    #[cfg(test)]
    test_main();

    r_os::hlt_loop();
}

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    r_os::hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    r_os::test_panic_handler(info)
}
