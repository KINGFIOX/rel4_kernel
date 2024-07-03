mod c_traps;
mod exception;
mod platform;

pub use c_traps::restore_user_context;
use core::arch::asm;
pub use platform::init_cpu;

use crate::config::RESET_CYCLES;
use sel4_common::arch::sbi::set_timer;

core::arch::global_asm!(include_str!("restore_fp.S"));

pub fn read_stval() -> usize {
    let temp: usize;
    unsafe {
        asm!("csrr {}, stval",out(reg)temp);
    }
    temp
}

pub extern "C" fn write_stvec(val: usize) {
    unsafe {
        asm!("csrw stvec , {}",in(reg) val);
    }
}

pub fn read_sip() -> usize {
    let temp: usize;
    unsafe {
        asm!("csrr {}, sip",out(reg)temp);
    }
    temp
}

pub fn read_time() -> usize {
    let temp: usize;
    unsafe {
        asm!("rdtime {}",out(reg)temp);
    }
    temp
}

pub fn read_scause() -> usize {
    let temp: usize;
    unsafe {
        asm!("csrr {}, scause",out(reg)temp);
    }
    temp
}

pub fn read_sepc() -> usize {
    let temp: usize;
    unsafe {
        asm!("csrr {}, sepc",out(reg)temp);
    }
    temp
}

pub fn read_sstatus() -> usize {
    let temp: usize;
    unsafe {
        asm!("csrr {}, sstatus",out(reg)temp);
    }
    temp
}

#[no_mangle]
pub fn resetTimer() {
    let mut target = read_time() + RESET_CYCLES;
    set_timer(target);
    while read_time() > target {
        target = read_time() + RESET_CYCLES;
        set_timer(target);
    }
}
