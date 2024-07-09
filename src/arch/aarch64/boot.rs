use driver_collect::SerialDriver;
use log::debug;
use sel4_common::{sel4_config::PAGE_BITS, BIT};
use sel4_task::create_idle_thread;
use sel4_vspace::{kpptr_to_paddr, rust_map_kernel_window};

use crate::arch::aarch64::platform::{cleanInvalidateL1Caches, invalidateLocalTLB};
use crate::{
    arch::{init_cpu, init_freemem},
    boot::{
        bi_finalise, calculate_extra_bi_size_bits, create_untypeds, init_core_state, init_dtb,
        ksNumCPUs, ndks_boot, paddr_to_pptr_reg, root_server_init,
    },
    config::{BI_FRAME_SIZE_BITS, USER_TOP},
    ffi::init_plat,
    structures::{p_region_t, seL4_SlotRegion, v_region_t},
};

use super::consts::KERNEL_ELF_BASE;

pub fn try_init_kernel(
    ui_p_reg_start: usize,
    ui_p_reg_end: usize,
    pv_offset: isize,
    v_entry: usize,
    dtb_phys_addr: usize,
    dtb_size: usize,
    ki_boot_end: usize,
) -> bool {
    // Init logging for log crate
    sel4_common::logging::init();
    let boot_mem_reuse_p_reg = p_region_t {
        start: kpptr_to_paddr(KERNEL_ELF_BASE),
        end: kpptr_to_paddr(ki_boot_end as usize),
    };
    let boot_mem_reuse_reg = paddr_to_pptr_reg(&boot_mem_reuse_p_reg);
    let ui_p_reg = p_region_t {
        start: ui_p_reg_start,
        end: ui_p_reg_end,
    };
    let ui_reg = paddr_to_pptr_reg(&ui_p_reg);

    let mut extra_bi_size = 0;
    let ui_v_reg = v_region_t {
        start: (ui_p_reg_start as isize - pv_offset) as usize,
        end: (ui_p_reg_end as isize - pv_offset) as usize,
    };
    let ipcbuf_vptr = ui_v_reg.end;
    let bi_frame_vptr = ipcbuf_vptr + BIT!(PAGE_BITS);
    let extra_bi_frame_vptr = bi_frame_vptr + BIT!(BI_FRAME_SIZE_BITS);

    // Map kernel window area
    sel4_common::ffi_call!(map_kernel_window);

    // Initialize cpu
    let inited = sel4_common::ffi_call!(init_cpu -> bool);

    // Initialize the drivers used by the kernel.
    driver_collect::init();
    log::debug!("init_cpu: {}", inited);

    // Initialize platform
    sel4_common::ffi_call!(init_plat);

    let dtb_p_reg = init_dtb(dtb_size, dtb_phys_addr, &mut extra_bi_size);
    if dtb_p_reg.is_none() {
        return false;
    }

    let extra_bi_size_bits = calculate_extra_bi_size_bits(extra_bi_size);

    let it_v_reg = v_region_t {
        start: ui_v_reg.start,
        end: extra_bi_frame_vptr + BIT!(extra_bi_size_bits),
    };

    if it_v_reg.end >= USER_TOP {
        debug!(
            "ERROR: userland image virt [{}..{}]
        exceeds USER_TOP ({})\n",
            it_v_reg.start, it_v_reg.end, USER_TOP
        );
        return false;
    }

    // TODO: init free memory regions
    if !init_freemem(ui_reg.clone(), dtb_p_reg.unwrap().clone()) {
        debug!("ERROR: free memory management initialization failed\n");
        return false;
    }

    if let Some((initial_thread, root_cnode_cap)) = root_server_init(
        it_v_reg,
        extra_bi_size_bits,
        ipcbuf_vptr,
        bi_frame_vptr,
        extra_bi_size,
        extra_bi_frame_vptr,
        ui_reg,
        pv_offset,
        v_entry,
    ) {
        create_idle_thread();
        cleanInvalidateL1Caches();
        init_core_state(initial_thread);
        if !create_untypeds(&root_cnode_cap, boot_mem_reuse_reg) {
            debug!("ERROR: could not create untypteds for kernel image boot memory");
        }
        unsafe {
            (*ndks_boot.bi_frame).sharedFrames = seL4_SlotRegion { start: 0, end: 0 };

            bi_finalise(dtb_size, dtb_phys_addr, extra_bi_size);
        }
        cleanInvalidateL1Caches();
        invalidateLocalTLB();
        // debug!("release_secondary_cores start");
        *ksNumCPUs.lock() = 1;
        #[cfg(feature = "ENABLE_SMP")]
        {
            unsafe {
                clh_lock_init();
                release_secondary_cores();
                clh_lock_acquire(cpu_id(), false);
            }
        }

        debug!("Booting all finished, dropped to user space");
        debug!("\n");
    } else {
        return false;
    }

    true
}
