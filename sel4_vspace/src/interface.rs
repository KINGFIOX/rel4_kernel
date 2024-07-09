// use crate::{common::{sel4_config::*, structures::exception_t, utils::{convert_to_mut_type_ref, pageBitsForSize}, fault::*}, BIT, ROUND_DOWN};
use crate::arch::pptr_to_paddr;
#[cfg(target_arch = "riscv64")]
use crate::arch::riscv64::sfence;
use crate::{asid_t, find_vspace_for_asid, pptr_t, vptr_t};
use sel4_common::fault::lookup_fault_t;
use sel4_common::sel4_config::seL4_PageBits;
use sel4_common::structures::exception_t;
use sel4_common::utils::pageBitsForSize;
// use crate::arch
// use crate::arch::find_vspace_for_asid;

/// 清除页表中对应的页表项。
///
/// `page_size`:在页表中寻找`vptr`对应的`pte`剩余的位数
///
/// `vptr`:该页表项对应的应用程序访问的虚拟地址（mapped_address）
///
/// `pptr`:分配的页面对应的虚拟地址(frame_base_ptr)
#[no_mangle]
pub fn unmapPage(
    page_size: usize,
    asid: asid_t,
    vptr: vptr_t,
    pptr: pptr_t,
) -> Result<(), lookup_fault_t> {
    let find_ret = find_vspace_for_asid(asid);
    if find_ret.status != exception_t::EXCEPTION_NONE {
        return Err(find_ret.lookup_fault.unwrap());
    }

    let lu_ret = unsafe { (*find_ret.vspace_root.unwrap()).lookup_pt_slot(vptr) };

    #[cfg(target_arch = "riscv64")]
    if lu_ret.ptBitsLeft != pageBitsForSize(page_size) {
        return Ok(());
    }

    let slot = unsafe { &(*lu_ret.ptSlot) };

    if slot.get_valid() == 0
        || slot.is_pte_table()
        || slot.get_ppn() << seL4_PageBits != pptr_to_paddr(pptr)
    {
        return Ok(());
    }

    unsafe {
        let slot = lu_ret.ptSlot as *mut usize;
        *slot = 0;
        #[cfg(target_arch = "riscv64")]
        sfence();
    }
    Ok(())
}
