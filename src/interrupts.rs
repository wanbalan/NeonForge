use crate::datetime::{CURRENT_TIME, TICKS};
use crate::pic::{ChainedPics, PIC_1_OFFSET, PIC_2_OFFSET};
use core::sync::atomic::Ordering;
use x86_64::instructions::port::Port;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

use crate::{print_key, KEY_HELD, LAST_KEYCODE, LAST_REPEAT_TICK};

static mut IDT: InterruptDescriptorTable = InterruptDescriptorTable::new();
static PICS: spin::Mutex<ChainedPics> =
    spin::Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET,
    Keyboard = PIC_1_OFFSET + 1,
}

impl InterruptIndex {
    fn as_u8(self) -> u8 {
        self as u8
    }

    fn as_usize(self) -> usize {
        usize::from(self.as_u8())
    }
}

extern "x86-interrupt" fn pit_interrupt_handler(_stack_frame: InterruptStackFrame) {
    let ticks = TICKS.fetch_add(1, Ordering::Relaxed) + 1;
    let key = LAST_KEYCODE.load(Ordering::Relaxed);

    if key != 0 && KEY_HELD[key as usize].load(Ordering::Relaxed) {
        let last_tick = LAST_REPEAT_TICK.load(Ordering::Relaxed);

        if ticks.wrapping_sub(last_tick.try_into().unwrap()) >= 200 {
            // 200 тиков = задержка между повторениями
            LAST_REPEAT_TICK.store(ticks as u64, Ordering::Relaxed);

            print_key(key, 80, 25);
        }
    }

    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
    }
}

extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame) {
    unsafe {
        let mut port = Port::new(0x60);
        let _scancode: u8 = port.read();

        // Здесь можно добавить обработку кода клавиши

        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Keyboard.as_u8());
    }
}

pub fn init_idt() {
    unsafe {
        IDT[InterruptIndex::Timer.as_usize()].set_handler_fn(pit_interrupt_handler);
        IDT[InterruptIndex::Keyboard.as_usize()].set_handler_fn(keyboard_interrupt_handler);
        let idt = &raw mut IDT;
        idt.as_ref().expect("IDT is None").load();
        PICS.lock().initialize();
    }
}

pub fn enable_interrupts() {
    x86_64::instructions::interrupts::enable();
}
