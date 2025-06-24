#![feature(global_asm)]

use crate::constants::HEAP_SIZE;
use crate::vga::{write_char, write_string};
use crate::ALLOCATOR;
use crate::LAST_KEYCODE;
use crate::TICKS;
use core::alloc::Layout;
use core::arch::global_asm;
use core::ptr::NonNull;

#[no_mangle]
pub extern "C" fn syscall_handler(
    num: u64,
    arg1: u64,
    arg2: u64,
    arg3: u64,
    arg4: u64,
    arg5: u64,
    arg6: u64,
) -> u64 {
    match num {
        0 => {
            // sys_print_char(x, y, ch)
            write_char(arg1 as usize, arg2 as usize, arg3 as u8, arg4 as u8);
            0
        }
        1 => {
            // sys_print_string(x, y, ptr, color)
            unsafe {
                let ptr = arg3 as *const u8;
                let color = arg4 as u8;

                let mut len = 0;
                while *ptr.add(len) != 0 {
                    len += 1;
                }

                let slice = core::slice::from_raw_parts(ptr, len);
                if let Ok(s) = core::str::from_utf8(slice) {
                    write_string(arg1 as usize, arg2 as usize, s, color);
                } else {
                    write_string(arg1 as usize, arg2 as usize, "UTF8 Err", 0x4F);
                    // красно-жёлтый
                }
            }
            0
        }
        2 => TICKS.load(core::sync::atomic::Ordering::Relaxed), // sys_get_ticks()
        3 => {
            // Системный вызов: получить код последней клавиши
            let key = LAST_KEYCODE.load(core::sync::atomic::Ordering::Relaxed);
            key as u64
        }
        0x10 => HEAP_SIZE as u64,
        0x11 => {
            let layout = Layout::from_size_align(arg1 as usize, 8).unwrap_or(Layout::new::<u8>());
            match ALLOCATOR.lock().allocate_first_fit(layout) {
                Ok(ptr) => ptr.as_ptr() as u64, // вернуть адрес
                Err(_) => 0,                    // ошибка — нет памяти
            }
        }
        0x12 => {
            let ptr = NonNull::new(arg1 as *mut u8);
            if let Some(non_null) = ptr {
                let layout =
                    Layout::from_size_align(arg2 as usize, 8).unwrap_or(Layout::new::<u8>());
                unsafe {
                    ALLOCATOR.lock().deallocate(non_null, layout);
                }
            }
            0
        }
        _ => 0,
    }
}

global_asm!(
    r#"
    .att_syntax
.globl syscall_entry
.text
syscall_entry:
    // Сохраняем регистры
    push %rax      // [rsp+0]
    push %rdi      // [rsp+8]
    push %rsi      // [rsp+16]
    push %rdx      // [rsp+24]
    push %rcx      // [rsp+32]
    push %r8       // [rsp+40]
    push %r9       // [rsp+48]
    push %r10
    push %r11

    // Распаковываем аргументы по стеку
    mov 64(%rsp), %rdi   // syscall number (из RAX)
    mov 56(%rsp), %rsi   // x (из RDI)
    mov 48(%rsp), %rdx   // y (из RSI)
    mov 40(%rsp), %rcx   // ch (из RDX)
    mov 32(%rsp), %r8    // color (из RCX)
    mov 24(%rsp), %r9    // 6-й аргумент (опц.)

    // Вызов обработчика
    mov $syscall_handler, %rax
    call *%rax

    // Восстанавливаем регистры
    pop %r11
    pop %r10
    pop %r9
    pop %r8
    pop %rcx
    pop %rdx
    pop %rsi
    pop %rdi
    pop %rax

    iretq

"#
);

extern "C" {
    pub fn syscall_entry(); // определена в syscall.asm
}
