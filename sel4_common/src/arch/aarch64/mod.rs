#![allow(unused)]
mod arch_tcb;
mod message_info;
mod object;
mod registers;
mod vm_rights;
pub use arch_tcb::ArchTCB;
pub use message_info::*;
pub use object::*;
pub use registers::*;
pub use vm_rights::*;

pub fn set_timer(timer: usize) {
    todo!("set_timer")
}

pub fn console_putchar(c: usize) {
    todo!("console_putchar")
}

pub fn console_getchar() -> usize {
    todo!("console_getchar")
}

pub fn clear_ipi() {
    todo!("clear_ipi")
}
pub fn shutdown() -> ! {
    todo!("shutdown");
    panic!("It should shutdown!");
}

pub fn sys_write(fd: usize, buffer: &[u8]) {
    todo!("sys_write");
}

pub fn remote_sfence_vma(hart_mask: usize, start: usize, size: usize) {
    todo!("remote_sfence_vma");
}

pub fn get_time() -> usize {
    todo!("get_time")
}
