#![no_std]

use core::ptr::NonNull;

use serial_frame::SerialDriver;

/// SBI putchar
const SBI_CONSOLE_PUTCHAR: usize = 1;
/// SBI getchar
const SBI_CONSOLE_GETCHAR: usize = 2;

/// Call sbi interface.
pub fn sbi_call(which: usize, arg0: usize, arg1: usize, arg2: usize) -> usize {
    let mut ret;
    unsafe {
        core::arch::asm!(
        "ecall",
        inlateout("x10") arg0 => ret,
        in("x11") arg1,
        in("x12") arg2,
        in("x17") which,
        );
    }
    ret
}

pub struct SerialSBI;

/// Implementation Serial driver for SerialSBI
impl SerialDriver for SerialSBI {
    fn new(_addr: NonNull<usize>) -> Self {
        SerialSBI
    }

    fn init(&self) {}

    /// Output a char c to data register
    fn putchar(&self, c: u8) {
        sbi_call(SBI_CONSOLE_PUTCHAR, c as _, 0, 0);
    }

    /// Return a byte if pl011 has received, or it will return `None`.
    fn getchar(&self) -> Option<u8> {
        let r = sbi_call(SBI_CONSOLE_GETCHAR, 0, 0, 0);
        // if r != -1
        if r != usize::MAX {
            Some(r as u8)
        } else {
            None
        }
    }
}
