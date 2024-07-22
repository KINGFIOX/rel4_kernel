mod boot;
mod c_traps;
mod consts;
mod exception;
mod ffi;
pub(self) mod instruction;
mod pg;
mod platform;

pub mod arm_gic;

use aarch64_cpu::registers::{Writeable, CNTV_CTL_EL0, CNTV_TVAL_EL0};
pub use boot::try_init_kernel;
pub use c_traps::restore_user_context;
pub(crate) use pg::set_vm_root_for_flush;
pub use platform::init_freemem;

pub fn read_sip() -> usize {
    // let temp: usize;
    // unsafe {
    //     asm!("csrr {}, sip",out(reg)temp);
    // }
    // temp
    todo!("read_sip")
}

/// Reset the current Timer
#[no_mangle]
pub fn resetTimer() {
    /*
        SYSTEM_WRITE_WORD(CNT_TVAL, TIMER_RELOAD);
        SYSTEM_WRITE_WORD(CNT_CTL, BIT(0));
    */
    const TIMER_CLOCK_HZ: u64 = 62500000;
    // TODO: Set a proper timer clock
    CNTV_TVAL_EL0.set(TIMER_CLOCK_HZ / 1000 * 10);
    CNTV_CTL_EL0.set(1);
}
