use crate::interrupt::handler::handleInterruptEntry;
use crate::syscall::slowpath;
use core::arch::asm;

#[cfg(feature = "ENABLE_SMP")]
use crate::{
    deps::{clh_is_self_in_queue, clh_lock_acquire, clh_lock_release},
    interrupt::getActiveIRQ,
};

#[cfg(feature = "ENABLE_SMP")]
use sel4_common::utils::cpu_id;
use sel4_task::get_currenct_thread;

#[no_mangle]
pub fn restore_user_context() {
    // NODE_UNLOCK_IF_HELD;

    // this is just a empty "do {} while (0)", I think it is only meaningfully under multi core case
    // at that case the micro NODE_UNLOCK_IF_HELD is
    // do {                         \
    //     if(clh_is_self_in_queue()) {                         \
    //         NODE_UNLOCK;                                     \
    //     }                                                    \
    // } while(0)

    // c_exit_hook();
    get_currenct_thread().tcbArch.load_thread_local();

    // #ifdef CONFIG_HAVE_FPU
    //     lazyFPURestore(NODE_STATE(ksCurThread));
    // #endif /* CONFIG_HAVE_FPU */
    unsafe {
        asm!(
                "mov     sp, {}                     \n",

                /* Restore thread's SPSR, LR, and SP */
                "ldp     x21, x22, [sp, #31 * 8] \n",
                "ldr     x23, [sp, #33 * 8]    \n",
                "msr     sp_el0, x21                \n",
        // #ifdef CONFIG_ARM_HYPERVISOR_SUPPORT
        //         "msr     elr_el2, x22               \n"
        //         "msr     spsr_el2, x23              \n"
        // #else
                "msr     elr_el1, x22               \n",
                "msr     spsr_el1, x23              \n",
        // #endif
                /* Restore remaining registers */
                "ldp     x0,  x1,  [sp, #16 * 0]    \n",
                "ldp     x2,  x3,  [sp, #16 * 1]    \n",
                "ldp     x4,  x5,  [sp, #16 * 2]    \n",
                "ldp     x6,  x7,  [sp, #16 * 3]    \n",
                "ldp     x8,  x9,  [sp, #16 * 4]    \n",
                "ldp     x10, x11, [sp, #16 * 5]    \n",
                "ldp     x12, x13, [sp, #16 * 6]    \n",
                "ldp     x14, x15, [sp, #16 * 7]    \n",
                "ldp     x16, x17, [sp, #16 * 8]    \n",
                "ldp     x18, x19, [sp, #16 * 9]    \n",
                "ldp     x20, x21, [sp, #16 * 10]   \n",
                "ldp     x22, x23, [sp, #16 * 11]   \n",
                "ldp     x24, x25, [sp, #16 * 12]   \n",
                "ldp     x26, x27, [sp, #16 * 13]   \n",
                "ldp     x28, x29, [sp, #16 * 14]   \n",
                "ldr     x30, [sp, #30 * 8]          \n",
                "eret",
                in(reg) get_currenct_thread().tcbArch.raw_ptr()
            );
    }
    panic!("unreachable")
}

#[no_mangle]
pub fn c_handle_interrupt() {
    // log::debug!("c_handle_interrupt");
    // if hart_id() != 0 {
    //     debug!("c_handle_interrupt");
    // }
    entry_hook();

    #[cfg(feature = "ENABLE_SMP")]
    {
        use crate::config::INTERRUPT_IPI_0;
        if getActiveIRQ() != INTERRUPT_IPI_0 {
            unsafe {
                clh_lock_acquire(cpu_id(), true);
            }
        }
    }
    // debug!("c_handle_interrupt");
    handleInterruptEntry();
    restore_user_context();
}

#[no_mangle]
pub fn c_handle_syscall(_cptr: usize, _msgInfo: usize, syscall: usize) {
    #[cfg(feature = "ENABLE_SMP")]
    unsafe {
        clh_lock_acquire(cpu_id(), false);
    }
    entry_hook();
    // ffi_call!(c_entry_hook);
    // if hart_id() == 0 {
    //     debug!("c_handle_syscall: syscall: {},", syscall as isize);
    // }
    slowpath(syscall);
    // debug!("c_handle_syscall complete");
}

/// This function should be the first thing called from after entry.
/// This function Save TPIDR(TLS) in aarch64.
#[inline]
pub fn entry_hook() {
    get_currenct_thread().tcbArch.save_thread_local();
}
