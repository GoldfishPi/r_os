
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};
use crate::{println, print};
use crate::gdt;
use lazy_static::lazy_static;
use pic8259_simple::ChainedPics;
use spin;

use crate::hlt_loop;

pub const PIC_1_OFFSET:u8 = 32;
pub const PIC_2_OFFSET:u8 = PIC_1_OFFSET + 8;

pub static PICS: spin::Mutex<ChainedPics> = 
    spin::Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InteruptsIndex {
    Timer = PIC_1_OFFSET,
    Keyboard,
}

impl InteruptsIndex {
    fn as_u8(self) -> u8 {
        self as u8
    }

    fn as_usize(self) -> usize {
        usize::from(self.as_u8())
    }
}

pub fn init_idt() {
    IDT.load();
}

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();

        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt.page_fault.set_handler_fn(page_fault_handler);

        idt[InteruptsIndex::Timer.as_usize()]
            .set_handler_fn(timer_interupt_handler);

        idt[InteruptsIndex::Keyboard.as_usize()]
            .set_handler_fn(keyboard_interupt_handler);

        unsafe {
            idt.double_fault.set_handler_fn(double_fault_handler)
                .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        }
        idt
    };
}


extern "x86-interrupt" fn breakpoint_handler (
    stack_frames: &mut InterruptStackFrame
) {
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frames);
}

extern "x86-interrupt" fn timer_interupt_handler (
    _stack_frames: &mut InterruptStackFrame
) {
    print!(".");
    
    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InteruptsIndex::Timer.as_u8());
    }
}

extern "x86-interrupt" fn keyboard_interupt_handler (
    _stack_frames: &mut InterruptStackFrame
) {
    use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1};
    use spin::Mutex;
    use x86_64::instructions::port::Port;

    lazy_static! {
        static ref KEYBOARD: Mutex<Keyboard<layouts::Us104Key, ScancodeSet1>> = {
            Mutex::new(Keyboard::new(layouts::Us104Key, ScancodeSet1, HandleControl::Ignore))
        };
    }

    let mut keyboard = KEYBOARD.lock();
    let mut port = Port::new(0x60);

    let scancode: u8 = unsafe { port.read() };
    if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
        if let Some(key) = keyboard.process_keyevent(key_event) {
            match key {
                DecodedKey::Unicode(character) => print!("{}", character),
                DecodedKey::RawKey(key) => print!("{:?}", key),
            }
        }
    }

    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InteruptsIndex::Keyboard.as_u8());
    }
}

extern "x86-interrupt" fn double_fault_handler (
    stack_frames: &mut InterruptStackFrame, _error_code:u64) -> ! {
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frames);
}

extern "x86-interrupt" fn page_fault_handler (
    stack_frames: &mut InterruptStackFrame, 
    error_code:PageFaultErrorCode
) {
    use x86_64::registers::control::Cr2;

    println!("EXCEPTION: PAGE FAULT");
    println!("Acessed Address: {:?}", Cr2::read());
    println!("Error Code: {:?}", error_code);
    println!("{:#?}", stack_frames);
    hlt_loop();
}

#[test_case]
fn test_breakpoint_exception() {
    x86_64::instructions::interrupts::int3();
}
