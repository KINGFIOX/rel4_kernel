#[cfg(target_arch = "aarch64")]
use core::arch::asm;
use sel4_common::arch::{maskVMRights, ArchReg};
use sel4_common::cap_rights::seL4_CapRights_t;
use sel4_common::utils::MAX_FREE_INDEX;
#[cfg(target_arch = "aarch64")]
use sel4_common::BIT;
use sel4_common::MASK;
use sel4_common::{
    message_info::seL4_MessageInfo_t,
    sel4_config::*,
    structures::exception_t,
    utils::{convert_to_mut_type_ref, pageBitsForSize},
};
use sel4_cspace::interface::{cap_t, cte_insert, cte_t};
use sel4_task::{get_currenct_thread, set_thread_state, ThreadState};
use sel4_vspace::{
    asid_pool_t, pptr_t, pptr_to_paddr, set_asid_pool_by_index, unmapPage, vm_attributes_t,
    PTEFlags, PTE,
};
#[cfg(target_arch = "riscv64")]
use sel4_vspace::{copyGlobalMappings, sfence};
#[cfg(target_arch = "aarch64")]
use sel4_vspace::{invalidate_tlb_by_asid_va, PDE, PUDE};

use crate::{kernel::boot::current_lookup_fault, utils::clear_memory};

pub fn invoke_page_table_unmap(cap: &mut cap_t) -> exception_t {
    if cap.get_pt_is_mapped() != 0 {
        let pt = convert_to_mut_type_ref::<PTE>(cap.get_pt_base_ptr());
        pt.unmap_page_table(cap.get_pt_mapped_asid(), cap.get_pt_mapped_address());
        clear_memory(pt.get_ptr() as *mut u8, seL4_PageTableBits)
    }
    cap.set_pt_is_mapped(0);
    exception_t::EXCEPTION_NONE
}
#[cfg(target_arch = "riscv64")]
pub fn invoke_page_table_map(
    pt_cap: &mut cap_t,
    pt_slot: &mut PTE,
    asid: usize,
    vaddr: usize,
) -> exception_t {
    let paddr = pptr_to_paddr(pt_cap.get_pt_base_ptr());
    let pte = PTE::new(paddr >> seL4_PageBits, PTEFlags::V);
    *pt_slot = pte;
    pt_cap.set_pt_is_mapped(1);
    pt_cap.set_pt_mapped_asid(asid);
    pt_cap.set_pt_mapped_address(vaddr);
    sfence();
    exception_t::EXCEPTION_NONE
}
#[cfg(target_arch = "aarch64")]
pub fn invoke_page_table_map(
    pt_cap: &mut cap_t,
    pd_slot: &mut PDE,
    asid: usize,
    vaddr: usize,
) -> exception_t {
    let paddr = pptr_to_paddr(pt_cap.get_pt_base_ptr());
    let pde = PDE::new_small(paddr >> seL4_PageBits);
    *pd_slot = pde;
    pt_cap.set_pt_is_mapped(1);
    pt_cap.set_pt_mapped_asid(asid);
    pt_cap.set_pt_mapped_address(vaddr);
    unsafe {
        asm!(
            "dc cvau, {}",
            "dmb sy",
            in(reg) pd_slot,
        );
    }
    exception_t::EXCEPTION_NONE
}

pub fn invoke_page_get_address(vbase_ptr: usize, call: bool) -> exception_t {
    let thread = get_currenct_thread();
    if call {
        thread.tcbArch.set_register(ArchReg::Badge, 0);
        let length = thread.set_mr(0, vbase_ptr);
        thread.tcbArch.set_register(
            ArchReg::MsgInfo,
            seL4_MessageInfo_t::new(0, 0, 0, length).to_word(),
        );
    }
    set_thread_state(thread, ThreadState::ThreadStateRestart);
    exception_t::EXCEPTION_NONE
}

pub fn invoke_page_unmap(frame_slot: &mut cte_t) -> exception_t {
    if frame_slot.cap.get_pt_mapped_asid() != asidInvalid {
        match unmapPage(
            frame_slot.cap.get_frame_size(),
            frame_slot.cap.get_frame_mapped_asid(),
            frame_slot.cap.get_pt_mapped_address(),
            frame_slot.cap.get_frame_base_ptr(),
        ) {
            Err(lookup_fault) => unsafe {
                current_lookup_fault = lookup_fault;
            },
            _ => {}
        }
    }
    frame_slot.cap.set_frame_mapped_address(0);
    frame_slot.cap.set_pt_mapped_asid(asidInvalid);
    exception_t::EXCEPTION_NONE
}
pub fn invoke_page_map(
    _frame_cap: &mut cap_t,
    w_rights_mask: usize,
    vaddr: usize,
    asid: usize,
    attr: vm_attributes_t,
    pt_slot: &mut PTE,
    frame_slot: &mut cte_t,
) -> exception_t {
    let frame_vm_rights = unsafe { core::mem::transmute(frame_slot.cap.get_frame_vm_rights()) };
    let vm_rights = maskVMRights(frame_vm_rights, seL4_CapRights_t::from_word(w_rights_mask));
    let frame_addr = pptr_to_paddr(frame_slot.cap.get_frame_base_ptr());
    frame_slot.cap.set_frame_mapped_address(vaddr);
    frame_slot.cap.set_frame_mapped_asid(asid);
    #[cfg(target_arch = "riscv64")]
    let executable = attr.get_execute_never() == 0;
    #[cfg(target_arch = "riscv64")]
    let pte = PTE::make_user_pte(frame_addr, executable, vm_rights);
    #[cfg(target_arch = "aarch64")]
    let pte = PTE::make_user_pte(frame_addr, vm_rights, attr, frame_slot.cap.get_frame_size());
    set_thread_state(get_currenct_thread(), ThreadState::ThreadStateRestart);
    pt_slot.update(pte);
    exception_t::EXCEPTION_NONE
}
#[cfg(target_arch = "aarch64")]
pub fn invoke_huge_page_map(
    vaddr: usize,
    asid: usize,
    frame_slot: &mut cte_t,
    pude: PUDE,
    pudSlot: &mut PUDE,
) -> exception_t {
    frame_slot.cap.set_frame_mapped_address(vaddr);
    frame_slot.cap.set_frame_mapped_asid(asid);
    *pudSlot = pude;
    unsafe {
        asm!(
            "dc cvau, {}",
            "dmb sy",
            in(reg) pudSlot,
        );
    }
    let tlbflush_required = pudSlot.get_pude_type() == 1;
    if tlbflush_required {
        assert!(asid < BIT!(16));
        invalidate_tlb_by_asid_va(asid, vaddr);
    }
    exception_t::EXCEPTION_NONE
}

#[cfg(target_arch = "aarch64")]
pub fn invoke_large_page_map(
    vaddr: usize,
    asid: usize,
    frame_slot: &mut cte_t,
    pde: PDE,
    pdSlot: &mut PDE,
) -> exception_t {
    frame_slot.cap.set_frame_mapped_address(vaddr);
    frame_slot.cap.set_frame_mapped_asid(asid);
    *pdSlot = pde;
    unsafe {
        asm!(
            "dc cvau, {}",
            "dmb sy",
            in(reg) pdSlot,
        );
    }
    let tlbflush_required = pdSlot.get_pde_type() == 1;
    if tlbflush_required {
        assert!(asid < BIT!(16));
        invalidate_tlb_by_asid_va(asid, vaddr);
    }
    exception_t::EXCEPTION_NONE
}

#[cfg(target_arch = "aarch64")]
pub fn invoke_small_page_map(
    vaddr: usize,
    asid: usize,
    frame_slot: &mut cte_t,
    pte: PTE,
    ptSlot: &mut PTE,
) -> exception_t {
    frame_slot.cap.set_frame_mapped_address(vaddr);
    frame_slot.cap.set_frame_mapped_asid(asid);
    *ptSlot = pte;
    unsafe {
        asm!(
            "dc cvau, {}",
            "dmb sy",
            in(reg) ptSlot,
        );
    }
    let tlbflush_required = ptSlot.pte_ptr_get_present();
    if tlbflush_required {
        assert!(asid < BIT!(16));
        invalidate_tlb_by_asid_va(asid, vaddr);
    }
    exception_t::EXCEPTION_NONE
}

pub fn invoke_asid_control(
    frame_ptr: pptr_t,
    slot: &mut cte_t,
    parent_slot: &mut cte_t,
    asid_base: usize,
) -> exception_t {
    parent_slot
        .cap
        .set_untyped_free_index(MAX_FREE_INDEX(parent_slot.cap.get_untyped_block_size()));
    clear_memory(frame_ptr as *mut u8, pageBitsForSize(RISCV_4K_Page));
    cte_insert(
        &cap_t::new_asid_pool_cap(asid_base, frame_ptr),
        parent_slot,
        slot,
    );
    assert_eq!(asid_base & MASK!(asidLowBits), 0);
    set_asid_pool_by_index(asid_base >> asidLowBits, frame_ptr);
    exception_t::EXCEPTION_NONE
}

#[cfg(target_arch = "riscv64")]
pub fn invoke_asid_pool(
    asid: usize,
    pool: &mut asid_pool_t,
    vspace_slot: &mut cte_t,
) -> exception_t {
    let region_base = vspace_slot.cap.get_pt_base_ptr();
    vspace_slot.cap.set_pt_is_mapped(1);
    vspace_slot.cap.set_pt_mapped_address(0);
    vspace_slot.cap.set_pt_mapped_asid(asid);

    copyGlobalMappings(region_base);
    pool.set_vspace_by_index(asid & MASK!(asidLowBits), region_base);
    exception_t::EXCEPTION_NONE
}

#[cfg(target_arch = "aarch64")]
pub fn invoke_asid_pool(
    asid: usize,
    pool: &mut asid_pool_t,
    vspace_slot: &mut cte_t,
) -> exception_t {
    todo!()
}
