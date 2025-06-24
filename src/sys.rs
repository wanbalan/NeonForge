#[inline(always)]
pub fn sys_print_char(x: u64, y: u64, ch: u8, color: u8) {
    unsafe {
        core::arch::asm!(
            "int 0x80",
            in("rax") 0,              // syscall number
            in("rdi") x,              // x
            in("rsi") y,              // y
            in("rdx") ch as u64,      // ch
            in("rcx") color as u64,   // color
            options(nostack)
        );
    }
}

pub fn sys_print_string(x: u64, y: u64, s: &'static str, color: u8) {
    unsafe {
        core::arch::asm!(
            "int 0x80",
            in("rax") 1,                       // syscall number
            in("rdi") x,                       // x
            in("rsi") y,                       // y
            in("rdx") s.as_ptr() as u64,       // ptr to str
            in("rcx") color as u64,            // color
            options(nostack)
        );
    }
}

pub fn sys_get_ticks() -> u64 {
    let ticks: u64;
    unsafe {
        core::arch::asm!(
            "int 0x80",
            in("rax") 2,     // syscall number
            lateout("rax") ticks, // результат будет в rax
            options(nostack)
        );
    }
    ticks
}

// Размер кучи
pub fn sys_heap_size() -> u64 {
    let size: u64;
    unsafe {
        core::arch::asm!(
            "int 0x80",
            in("rax") 0x10,
            lateout("rax") size,
            options(nostack)
        );
    }
    size
}

// Выделение
pub fn sys_alloc(size: u64) -> u64 {
    let addr: u64;
    unsafe {
        core::arch::asm!(
            "int 0x80",
            in("rax") 0x11,
            in("rdi") size,
            lateout("rax") addr,
            options(nostack)
        );
    }
    addr
}

// Освобождение
pub fn sys_dealloc(ptr: u64, size: u64) {
    unsafe {
        core::arch::asm!(
            "int 0x80",
            in("rax") 0x12,
            in("rdi") ptr,
            in("rsi") size,
            options(nostack)
        );
    }
}

