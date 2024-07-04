use aarch64_cpu::registers::Writeable;
use aarch64_cpu::registers::{TPIDR_EL1, VBAR_EL1};
use core::arch::asm;
use sel4_common::sel4_config::CONFIG_KERNEL_STACK_BITS;
use sel4_common::utils::cpu_id;

use super::ffi::*;
use crate::boot::{
    avail_p_regs_addr, avail_p_regs_size, paddr_to_pptr_reg, res_reg, reserve_region,
    rust_init_freemem,
};
use crate::config::*;
use crate::ffi::*;
use crate::structures::*;
use log::debug;
use sel4_common::sel4_config::KERNEL_ELF_BASE;
use sel4_vspace::*;

use super::arm_gic::gic_v2::gic_v2::cpu_initLocalIRQController;
pub fn init_cpu() -> bool {
    // use arch::aarch64::arm_gic::gic_v2;

    // Setup kernel stack pointer.

    // Wrapping_add, first argument is CURRENT_CPU_INDEX
    //
    let mut stack_top =
        (kernel_stack_alloc as *mut u8).wrapping_add(0 + (1 << CONFIG_KERNEL_STACK_BITS)) as u64;
    stack_top |= cpu_id() as u64; //the judge of enable smp have done in cpu_id

    TPIDR_EL1.set(stack_top);
    // CPU's exception vector table
    unsafe {
        asm!("dsb sy;"); // DSB SY
        VBAR_EL1.set(arm_vector_table as u64);
        asm!("isb;"); // ISB SY
    }
    // initLocalIRQController
    cpu_initLocalIRQController();
    // armv_init_user_access
    // user_access::armv_init_user_access();
    //initTimer

    unsafe {
        initTimer();
    }
    true
}
pub fn init_freemem(ui_reg: region_t, dtb_p_reg: p_region_t) -> bool {
    extern "C" {
        fn ki_end();
    }
    unsafe {
        res_reg[0].start = paddr_to_pptr(kpptr_to_paddr(KERNEL_ELF_BASE));
        res_reg[0].end = paddr_to_pptr(kpptr_to_paddr(ki_end as usize));
    }

    let mut index = 1;

    if dtb_p_reg.start != 0 {
        if index >= NUM_RESERVED_REGIONS {
            debug!("ERROR: no slot to add DTB to reserved regions\n");
            return false;
        }
        unsafe {
            res_reg[index] = paddr_to_pptr_reg(&dtb_p_reg);
            index += 1;
        }
    }

    // here use the MODE_RESERVED:ARRAY_SIZE(mode_reserved_region) to judge
    // but in aarch64, the array size is always 0
    // so eliminate some code
    if ui_reg.start < PADDR_TOP {
        if (index >= NUM_RESERVED_REGIONS) {
            debug!("ERROR: no slot to add the user image to the reserved regions");
            return false;
        }
        res_reg[index] = paddr_to_pptr_reg(&dtb_p_reg);
        index += 1;
    } else {
        reserve_region(ui_p_reg);
    }

    unsafe { rust_init_freemem(avail_p_regs_size, avail_p_regs_addr, index, res_reg.clone()) }
}