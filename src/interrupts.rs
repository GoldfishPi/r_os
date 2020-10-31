
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};
use crate::{println, print};
use crate::gdt;
use lazy_static::lazy_static;
use pic8259_simple::ChainedPics;
use spin;

pub const PIC_1_OFFSET:u8 = 32;
pub const PIC_2_OFFSET:u8 = PIC_1_OFFSET + 8;

pub static PICS: spin::Mutex<ChainedPics> = 
    spin::Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InteruptsIndex {
    Timer = PIC_1_OFFSET,
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
        idt[InteruptsIndex::Timer.as_usize()]
            .set_handler_fn(timer_interupt_handler);
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

extern "x86-interrupt" fn double_fault_handler (
    stack_frames: &mut InterruptStackFrame, _error_code:u64) -> ! {
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frames);
}

#[test_case]
fn test_breakpoint_exception() {
    x86_64::instructions::interrupts::int3();
}
