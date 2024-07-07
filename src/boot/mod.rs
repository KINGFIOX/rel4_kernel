mod interface;
mod mm;
mod root_server;
mod untyped;
mod utils;

use core::mem::size_of;

use crate::ffi::tcbDebugAppend;
use crate::{BIT, ROUND_UP};
use log::debug;
use sel4_common::sel4_config::{seL4_PageBits, PADDR_TOP};
use spin::Mutex;

pub use crate::boot::utils::paddr_to_pptr_reg;
use crate::config::*;
use crate::structures::{
    ndks_boot_t, p_region_t, region_t, seL4_BootInfo, seL4_BootInfoHeader, seL4_SlotRegion,
};

#[cfg(target_arch = "aarch64")]
pub use mm::reserve_region;
pub use mm::{avail_p_regs_addr, avail_p_regs_size, res_reg, rust_init_freemem};
pub use root_server::rootserver;
use sel4_task::*;
use sel4_vspace::*;

pub use root_server::root_server_init;
pub use untyped::create_untypeds;

#[cfg(feature = "ENABLE_SMP")]
pub use utils::{provide_cap, write_slot};

#[cfg(feature = "ENABLE_SMP")]
use crate::ffi::{clh_lock_acquire, clh_lock_init};

#[cfg(feature = "ENABLE_SMP")]
use core::arch::asm;

#[cfg(feature = "ENABLE_SMP")]
use sel4_common::utils::cpu_id;

pub static ksNumCPUs: Mutex<usize> = Mutex::new(0);
#[cfg(feature = "ENABLE_SMP")]
pub static node_boot_lock: Mutex<usize> = Mutex::new(0);

#[no_mangle]
#[link_section = ".boot.bss"]
pub static mut ndks_boot: ndks_boot_t = ndks_boot_t {
    reserved: [p_region_t { start: 0, end: 0 }; MAX_NUM_RESV_REG],
    resv_count: 0,
    freemem: [region_t { start: 0, end: 0 }; MAX_NUM_FREEMEM_REG],
    bi_frame: 0 as *mut seL4_BootInfo,
    slot_pos_cur: seL4_NumInitialCaps,
};

pub fn calculate_extra_bi_size_bits(size: usize) -> usize {
    if size == 0 {
        return 0;
    }

    let clzl_ret = ROUND_UP!(size, seL4_PageBits).leading_zeros() as usize;
    let mut msb = seL4_WordBits - 1 - clzl_ret;
    if size > BIT!(msb) {
        msb += 1;
    }
    return msb;
}

pub fn init_dtb(
    dtb_size: usize,
    dtb_phys_addr: usize,
    extra_bi_size: &mut usize,
) -> Option<p_region_t> {
    let mut dtb_p_reg = p_region_t { start: 0, end: 0 };
    if dtb_size > 0 {
        let dtb_phys_end = dtb_phys_addr + dtb_size;
        if dtb_phys_end < dtb_phys_addr {
            debug!(
                "ERROR: DTB location at {}
             len {} invalid",
                dtb_phys_addr, dtb_size
            );
            return None;
        }
        if dtb_phys_end >= PADDR_TOP {
            debug!(
                "ERROR: DTB at [{}..{}] exceeds PADDR_TOP ({})\n",
                dtb_phys_addr, dtb_phys_end, PADDR_TOP
            );
            return None;
        }

        (*extra_bi_size) += size_of::<seL4_BootInfoHeader>() + dtb_size;
        dtb_p_reg = p_region_t {
            start: dtb_phys_addr,
            end: dtb_phys_end,
        };
    }
    Some(dtb_p_reg)
}

pub fn init_bootinfo(dtb_size: usize, dtb_phys_addr: usize, extra_bi_size: usize) {
    let mut extra_bi_offset = 0;
    let mut header: seL4_BootInfoHeader = seL4_BootInfoHeader { id: 0, len: 0 };
    if dtb_size > 0 {
        header.id = SEL4_BOOTINFO_HEADER_FDT;
        header.len = size_of::<seL4_BootInfoHeader>() + dtb_size;
        unsafe {
            *((rootserver.extra_bi + extra_bi_offset) as *mut seL4_BootInfoHeader) = header.clone();
        }
        extra_bi_offset += size_of::<seL4_BootInfoHeader>();
        let src = unsafe {
            core::slice::from_raw_parts(paddr_to_pptr(dtb_phys_addr) as *const u8, dtb_size)
        };
        unsafe {
            let dst = core::slice::from_raw_parts_mut(
                (rootserver.extra_bi + extra_bi_offset) as *mut u8,
                dtb_size,
            );
            dst.copy_from_slice(src);
        }
    }
    if extra_bi_size > extra_bi_offset {
        header.id = SEL4_BOOTINFO_HEADER_PADDING;
        header.len = extra_bi_size - extra_bi_offset;
        unsafe {
            *((rootserver.extra_bi + extra_bi_offset) as *mut seL4_BootInfoHeader) = header.clone();
        }
    }
}

pub fn bi_finalise(dtb_size: usize, dtb_phys_addr: usize, extra_bi_size: usize) {
    unsafe {
        (*ndks_boot.bi_frame).empty = seL4_SlotRegion {
            start: ndks_boot.slot_pos_cur,
            end: BIT!(CONFIG_ROOT_CNODE_SIZE_BITS),
        };
    }
    init_bootinfo(dtb_size, dtb_phys_addr, extra_bi_size);
}

pub fn init_core_state(scheduler_action: *mut tcb_t) {
    unsafe {
        #[cfg(feature = "ENABLE_SMP")]
        if scheduler_action as usize != 0 && scheduler_action as usize != 1 {
            tcbDebugAppend(scheduler_action);
        }
        let idle_thread = {
            #[cfg(not(feature = "ENABLE_SMP"))]
            {
                ksIdleThread as *mut tcb_t
            }
            #[cfg(feature = "ENABLE_SMP")]
            {
                ksSMP[cpu_id()].ksIdleThread as *mut tcb_t
            }
        };

        tcbDebugAppend(idle_thread);
        set_current_scheduler_action(scheduler_action as usize);
        set_current_thread(get_idle_thread());
    }
}

#[cfg(feature = "ENABLE_SMP")]
pub fn try_init_kernel_secondary_core(hartid: usize, core_id: usize) -> bool {
    use core::ops::AddAssign;
    while node_boot_lock.lock().eq(&0) {}
    // debug!("start try_init_kernel_secondary_core");
    init_cpu();
    debug!("init cpu compl");
    unsafe { clh_lock_acquire(cpu_id(), false) }
    ksNumCPUs.lock().add_assign(1);
    init_core_state(SchedulerAction_ResumeCurrentThread as *mut tcb_t);
    debug!("init_core_state compl");

    unsafe {
        asm!("fence.i");
    }
    true
}

#[cfg(feature = "ENABLE_SMP")]
fn release_secondary_cores() {
    use sel4_common::sel4_config::CONFIG_MAX_NUM_NODES;
    *node_boot_lock.lock() = 1;
    unsafe {
        asm!("fence rw, rw");
    }
    while ksNumCPUs.lock().ne(&CONFIG_MAX_NUM_NODES) {}
}
