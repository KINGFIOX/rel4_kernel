use crate::arch::set_vm_root_for_flush;
use crate::boot::rootserver;
use crate::compatibility::setThreadState;
use crate::config::{seL4_ASIDPoolBits, USER_TOP};
use crate::kernel::boot::{current_extra_caps, get_extra_cap_by_index};
use crate::syscall::invocation::decode::current_syscall_error;
use crate::syscall::ThreadState;
use crate::syscall::{current_lookup_fault, get_syscall_arg, set_thread_state, unlikely};
use crate::syscall::{ensure_empty_slot, get_currenct_thread, lookup_slot_for_cnode_op};
use log::debug;
use sel4_common::arch::maskVMRights;
use sel4_common::cap_rights::seL4_CapRights_t;
use sel4_common::fault::lookup_fault_t;
use sel4_common::sel4_config::{
    asidInvalid, asidLowBits, nASIDPools, seL4_AlignmentError, seL4_FailedLookup, seL4_RangeError,
    ARM_Huge_Page, ARM_Large_Page, ARM_Small_Page, PAGE_BITS, PGD_INDEX_OFFSET, PUD_INDEX_OFFSET,
};
use sel4_common::sel4_config::{seL4_DeleteFirst, seL4_InvalidArgument};
use sel4_common::sel4_config::{
    seL4_IllegalOperation, seL4_InvalidCapability, seL4_RevokeFirst, seL4_TruncatedMessage,
    PD_INDEX_OFFSET,
};
use sel4_common::utils::{
    convert_to_mut_type_ref, global_ops, pageBitsForSize, ptr_to_mut, ptr_to_ref, MAX_FREE_INDEX,
};
use sel4_common::{
    arch::MessageLabel,
    structures::{exception_t, seL4_IPCBuffer},
    MASK,
};
use sel4_common::{BIT, IS_ALIGNED};
use sel4_cspace::interface::{cap_t, cte_insert, cte_t, CapTag};

use sel4_vspace::{
    asid_map_t, asid_pool_t, asid_t, find_vspace_for_asid, get_asid_pool_by_index, makeUser3rdLevel, make_user_1st_level, make_user_2nd_level, paddr_t, pptr_to_paddr, set_asid_pool_by_index, set_vm_root, vm_attributes_t, vptr_t, PDE, PGDE, PTE, PUDE
};

use crate::syscall::invocation::invoke_mmu_op::{
    invoke_huge_page_map, invoke_large_page_map, invoke_page_get_address, invoke_page_table_unmap,
    invoke_page_unmap, invoke_small_page_map,
};
use crate::{
    config::maxIRQ,
    interrupt::is_irq_active,
    syscall::{invocation::invoke_irq::invoke_irq_control, lookupSlotForCNodeOp},
};

pub fn decode_mmu_invocation(
    label: MessageLabel,
    length: usize,
    slot: &mut cte_t,
    call: bool,
    buffer: Option<&seL4_IPCBuffer>,
) -> exception_t {
    match slot.cap.get_cap_type() {
        CapTag::CapPageGlobalDirectoryCap => {
            decode_vspace_root_invocation(label, length, slot, buffer)
        }
        CapTag::CapPageUpperDirectoryCap => {
            decode_page_upper_directory_invocation(label, length, slot, buffer)
        }
        CapTag::CapPageDirectoryCap => {
            decode_page_directory_invocation(label, length, slot, buffer)
        }
        CapTag::CapPageTableCap => decode_page_table_invocation(label, length, slot, buffer),
        CapTag::CapFrameCap => decode_frame_invocation(label, length, slot, call, buffer),
        CapTag::CapASIDControlCap => decode_asid_control(label, length, buffer),
        CapTag::CapASIDPoolCap => decode_asid_pool(label, slot),
        _ => {
            panic!("Invalid arch cap type");
        }
    }
}

fn decode_page_table_invocation(
    label: MessageLabel,
    length: usize,
    cte: &mut cte_t,
    buffer: Option<&seL4_IPCBuffer>,
) -> exception_t {
    /*
        if (invLabel == ARMPageTableUnmap) {
            if (unlikely(!isFinalCapability(cte))) {
                current_syscall_error.type = seL4_RevokeFirst;
                return EXCEPTION_SYSCALL_ERROR;
            }
            setThreadState(NODE_STATE(ksCurThread), ThreadState_Restart);
            return performPageTableInvocationUnmap(cap, cte);
        }
    */

    if label == MessageLabel::ARMPageTableUnmap {
        log::warn!("Need to check is FinalCapability here");
        get_currenct_thread().set_state(ThreadState::ThreadStateRestart);
        unimplemented!("performPageTableInvocationUnmap");
    }

    if unlikely(label != MessageLabel::ARMPageTableMap) {
        global_ops!(current_syscall_error._type = seL4_IllegalOperation);
        return exception_t::EXCEPTION_SYSCALL_ERROR;
    }
    if unlikely(length < 2 || global_ops!(current_extra_caps.excaprefs[0] == 0)) {
        global_ops!(current_syscall_error._type = seL4_TruncatedMessage);
        return exception_t::EXCEPTION_SYSCALL_ERROR;
    }
    if unlikely(cte.cap.get_pt_is_mapped() == 1) {
        global_ops!(current_syscall_error._type = seL4_InvalidCapability);
        global_ops!(current_syscall_error.invalidArgumentNumber = 0);
        return exception_t::EXCEPTION_SYSCALL_ERROR;
    }

    let vaddr = get_syscall_arg(0, buffer) & !(MASK!(PD_INDEX_OFFSET));
    let vspace_root_cap =
        convert_to_mut_type_ref::<cap_t>(global_ops!(current_extra_caps.excaprefs[0]));

    if unlikely(!vspace_root_cap.is_valid_native_root()) {
        global_ops!(current_syscall_error._type = seL4_InvalidCapability);
        global_ops!(current_syscall_error.invalidCapNumber = 1);
        return exception_t::EXCEPTION_SYSCALL_ERROR;
    }

    let vspace_root = vspace_root_cap.get_pgd_base_ptr();
    let asid = vspace_root_cap.get_pgd_mapped_asid();

    if unlikely(vaddr > USER_TOP) {
        global_ops!(current_syscall_error._type = seL4_InvalidArgument);
        global_ops!(current_syscall_error.invalidArgumentNumber = 0);
        return exception_t::EXCEPTION_SYSCALL_ERROR;
    }

    let find_ret = find_vspace_for_asid(asid);

    if unlikely(find_ret.status != exception_t::EXCEPTION_NONE) {
        global_ops!(current_syscall_error._type = seL4_FailedLookup);
        global_ops!(current_syscall_error.failedLookupWasSource = 0);
        return exception_t::EXCEPTION_SYSCALL_ERROR;
    }
    if unlikely(find_ret.vspace_root.unwrap() as usize != vspace_root) {
        global_ops!(current_syscall_error._type = seL4_InvalidCapability);
        global_ops!(current_syscall_error.invalidCapNumber = 1);
        return exception_t::EXCEPTION_SYSCALL_ERROR;
    }

    let pd_slot = PGDE::new_from_pte(vspace_root).lookup_pd_slot(vaddr);

    if pd_slot.status != exception_t::EXCEPTION_NONE {
        global_ops!(current_syscall_error._type = seL4_FailedLookup);
        global_ops!(current_syscall_error.failedLookupWasSource = 0);
        return exception_t::EXCEPTION_SYSCALL_ERROR;
    }
    if unlikely(
        ptr_to_ref(pd_slot.pdSlot).get_present() || ptr_to_ref(pd_slot.pdSlot).is_larger_page(),
    ) {
        global_ops!(current_syscall_error._type = seL4_DeleteFirst);
        return exception_t::EXCEPTION_SYSCALL_ERROR;
    }

    let pde = PDE::new_page(pptr_to_paddr(cte.cap.get_pt_base_ptr()), 0x3);
    cte.cap.set_pt_is_mapped(1);
    cte.cap.set_pt_mapped_asid(asid);
    cte.cap.set_pt_mapped_address(vaddr);
    get_currenct_thread().set_state(ThreadState::ThreadStateRestart);

    *ptr_to_mut(pd_slot.pdSlot) = pde;
    log::warn!("Need to clean D-Cache using cleanByVA_PoU");
    exception_t::EXCEPTION_NONE
}

fn decode_page_clean_invocation(
    label: MessageLabel,
    length: usize,
    cte: &mut cte_t,
    call: bool,
    buffer: Option<&seL4_IPCBuffer>,
) -> exception_t {
    if length < 2 {
        log::error!("[User] Page Flush: Truncated message.");
        global_ops!(current_syscall_error._type = seL4_TruncatedMessage);
        return exception_t::EXCEPTION_SYSCALL_ERROR;
    }
    if unlikely(cte.cap.get_frame_mapped_asid() == 0) {
        log::error!("[User] Page Flush: Frame is not mapped.");
        global_ops!(current_syscall_error._type = seL4_IllegalOperation);
        return exception_t::EXCEPTION_SYSCALL_ERROR;
    }

    let vaddr = cte.cap.get_frame_mapped_address();
    let asid = cte.cap.get_frame_mapped_asid();
    let find_ret = find_vspace_for_asid(asid);

    if unlikely(find_ret.status != exception_t::EXCEPTION_NONE) {
        log::error!("[User] Page Flush: No PGD for ASID");
        global_ops!(current_syscall_error._type = seL4_FailedLookup);
        global_ops!(current_syscall_error.failedLookupWasSource = 0);
        return exception_t::EXCEPTION_SYSCALL_ERROR;
    }

    let start = get_syscall_arg(0, buffer);
    let end = get_syscall_arg(1, buffer);

    if end <= start {
        log::error!("[User] Page Flush: Invalid range");
        global_ops!(current_syscall_error._type = seL4_InvalidArgument);
        global_ops!(current_syscall_error.invalidArgumentNumber = 1);
        return exception_t::EXCEPTION_SYSCALL_ERROR;
    }

    let page_size = BIT!(pageBitsForSize(cte.cap.get_frame_size()));
    if start >= page_size || end > page_size {
        log::error!("[User] Page Flush: Requested range not inside page");
        global_ops!(current_syscall_error._type = seL4_InvalidArgument);
        global_ops!(current_syscall_error.invalidArgumentNumber = 0);
        return exception_t::EXCEPTION_SYSCALL_ERROR;
    }
    let pstart = pptr_to_paddr(cte.cap.get_frame_base_ptr() + start);
    get_currenct_thread().set_state(ThreadState::ThreadStateRestart);

    if start < end {
        let root_switched = set_vm_root_for_flush(find_ret.vspace_root.unwrap() as _, asid);
        log::warn!(
            "need to flush cache for decode_page_clean_invocation label: {:?}",
            label
        );
        if root_switched {
            get_currenct_thread()
                .set_vm_root()
                .expect("can't set vm root for decode_page_clean_invocation");
        }
    }
    exception_t::EXCEPTION_NONE

    /*
        static exception_t performPageFlush(int invLabel, vspace_root_t *vspaceRoot, asid_t asid,
                                    vptr_t start, vptr_t end, paddr_t pstart)
        {
            bool_t root_switched;
                if (start < end) {
                    root_switched = setVMRootForFlush(vspaceRoot, asid);
                    doFlush(invLabel, start, end, pstart);
                    if (root_switched) {
                        setVMRoot(NODE_STATE(ksCurThread));
                    }
                }
            return EXCEPTION_NONE;
        }
    */
    /*
        return performPageFlush(invLabel, find_ret.vspace_root, asid, vaddr + start, vaddr + end - 1,
                                pstart);
    */
}

fn decode_frame_invocation(
    label: MessageLabel,
    length: usize,
    frame_slot: &mut cte_t,
    call: bool,
    buffer: Option<&seL4_IPCBuffer>,
) -> exception_t {
    match label {
        MessageLabel::ARMPageMap => decode_frame_map(length, frame_slot, buffer),
        MessageLabel::ARMPageUnmap => {
            set_thread_state(get_currenct_thread(), ThreadState::ThreadStateRestart);
            invoke_page_unmap(frame_slot)
        }
        MessageLabel::ARMPageClean_Data
        | MessageLabel::ARMPageInvalidate_Data
        | MessageLabel::ARMPageCleanInvalidate_Data
        | MessageLabel::ARMPageUnify_Instruction => {
            decode_page_clean_invocation(label, length, frame_slot, call, buffer)
        }
        MessageLabel::ARMPageGetAddress => {
            set_thread_state(get_currenct_thread(), ThreadState::ThreadStateRestart);
            invoke_page_get_address(frame_slot.cap.get_frame_base_ptr(), call)
        }
        _ => {
            debug!("invalid operation label:{:?}", label);
            unsafe {
                current_syscall_error._type = seL4_IllegalOperation;
            }
            exception_t::EXCEPTION_SYSCALL_ERROR
        }
    }
}

fn decode_asid_control(
    label: MessageLabel,
    length: usize,
    buffer: Option<&seL4_IPCBuffer>,
) -> exception_t {
    if unlikely(label != MessageLabel::ARMASIDControlMakePool) {
        global_ops!(current_syscall_error._type = seL4_IllegalOperation);
        return exception_t::EXCEPTION_SYSCALL_ERROR;
    }
    if unlikely(
        length < 2
            || global_ops!(current_extra_caps.excaprefs[0] == 0)
            || global_ops!(current_extra_caps.excaprefs[1] == 0),
    ) {
        global_ops!(current_syscall_error._type = seL4_TruncatedMessage);
        return exception_t::EXCEPTION_SYSCALL_ERROR;
    }
    let index = get_syscall_arg(0, buffer);
    let depth = get_syscall_arg(1, buffer);
    let parent_slot =
        convert_to_mut_type_ref::<cte_t>(global_ops!(current_extra_caps.excaprefs[0]));
    let untyped = parent_slot.cap;
    let root = convert_to_mut_type_ref::<cte_t>(global_ops!(current_extra_caps.excaprefs[1])).cap;

    let mut i = 0;
    loop {
        if !(i < nASIDPools && get_asid_pool_by_index(i) != 0) {
            break;
        }
        i += 1;
    }
    if unlikely(i == nASIDPools) {
        /* If no unallocated pool is found */
        global_ops!(current_syscall_error._type = seL4_DeleteFirst);
        return exception_t::EXCEPTION_SYSCALL_ERROR;
    }
    let asid_base = i << asidLowBits;
    if unlikely(
        untyped.get_cap_type() != CapTag::CapUntypedCap
            || untyped.get_untyped_block_size() != seL4_ASIDPoolBits
            || untyped.get_untyped_is_device() == 1,
    ) {
        global_ops!(current_syscall_error._type = seL4_InvalidCapability);
        global_ops!(current_syscall_error.invalidCapNumber = 0);
        return exception_t::EXCEPTION_SYSCALL_ERROR;
    }
    let status = parent_slot.ensure_no_children();
    if unlikely(status != exception_t::EXCEPTION_NONE) {
        return status;
    }
    let frame = untyped.get_cap_ptr();
    let lu_ret = lookup_slot_for_cnode_op(false, &root, index, depth);
    if unlikely(lu_ret.status != exception_t::EXCEPTION_NONE) {
        return lu_ret.status;
    }
    let dest_slot = ptr_to_mut(lu_ret.slot);
    let status = ensure_empty_slot(dest_slot);
    if unlikely(status != exception_t::EXCEPTION_NONE) {
        return status;
    }
    get_currenct_thread().set_state(ThreadState::ThreadStateRestart);
    parent_slot
        .cap
        .set_untyped_free_index(MAX_FREE_INDEX(parent_slot.cap.get_untyped_block_size()));
    unsafe {
        core::slice::from_raw_parts_mut(frame as *mut u8, BIT!(seL4_ASIDPoolBits)).fill(0);
    }
    cte_insert(
        &cap_t::new_asid_pool_cap(asid_base, frame),
        parent_slot,
        dest_slot,
    );
    assert!(asid_base & MASK!(asidLowBits) == 0);
    set_asid_pool_by_index(asid_base >> asidLowBits, frame);
    exception_t::EXCEPTION_NONE
}

fn decode_asid_pool(label: MessageLabel, cte: &mut cte_t) -> exception_t {
    if unlikely(label != MessageLabel::ARMASIDPoolAssign) {
        global_ops!(current_syscall_error._type = seL4_IllegalOperation);
        return exception_t::EXCEPTION_SYSCALL_ERROR;
    }
    if unlikely(global_ops!(current_extra_caps.excaprefs[0] == 0)) {
        global_ops!(current_syscall_error._type = seL4_TruncatedMessage);
        return exception_t::EXCEPTION_SYSCALL_ERROR;
    }

    let vspace_cap_slot = global_ops!(current_extra_caps.excaprefs[0]);
    let vspace_cap = convert_to_mut_type_ref::<cap_t>(vspace_cap_slot);

    if unlikely(!vspace_cap.is_vtable_root() || vspace_cap.get_pgd_is_mapped() == 1) {
        log::debug!("is not a valid vtable root");
        global_ops!(current_syscall_error._type = seL4_InvalidCapability);
        global_ops!(current_syscall_error.invalidArgumentNumber = 1);
        return exception_t::EXCEPTION_SYSCALL_ERROR;
    }
    let pool = get_asid_pool_by_index(cte.cap.get_asid_base() >> asidLowBits);

    if unlikely(pool == 0) {
        global_ops!(current_syscall_error._type = seL4_FailedLookup);
        global_ops!(current_syscall_error.failedLookupWasSource = 0);
        unsafe {
            current_lookup_fault = lookup_fault_t::new_root_invalid();
        }
        return exception_t::EXCEPTION_SYSCALL_ERROR;
    }

    if unlikely(pool != cte.cap.get_asid_pool()) {
        global_ops!(current_syscall_error._type = seL4_InvalidCapability);
        global_ops!(current_syscall_error.invalidCapNumber = 0);
        return exception_t::EXCEPTION_SYSCALL_ERROR;
    }

    let mut asid = cte.cap.get_asid_base();

    let pool = convert_to_mut_type_ref::<asid_pool_t>(pool);
    let mut i = 0;

    // TODO: Make pool judge more efficient and pretty.
    while i < BIT!(asidLowBits) && (asid + i == 0 || pool[i].words[0] != 0) {
        i += 1;
    }

    if i == BIT!(asidLowBits) {
        unsafe {
            current_syscall_error._type = seL4_DeleteFirst;
        }
        return exception_t::EXCEPTION_SYSCALL_ERROR;
    }

    asid += i;

    get_currenct_thread().set_state(ThreadState::ThreadStateRestart);
    vspace_cap.set_pgd_mapped_asid(asid);
    vspace_cap.set_pgd_is_mapped(1);
    let asid_map = asid_map_t::new_vspace(vspace_cap.get_pgd_base_ptr());
    pool[asid & MASK!(asidLowBits)] = asid_map;
    exception_t::EXCEPTION_NONE
}

fn decode_frame_map(
    length: usize,
    frame_slot: &mut cte_t,
    buffer: Option<&seL4_IPCBuffer>,
) -> exception_t {
    if length < 3 || get_extra_cap_by_index(0).is_none() {
        debug!("ARMPageMap: Truncated message.");
        global_ops!(current_syscall_error._type = seL4_TruncatedMessage);
        return exception_t::EXCEPTION_SYSCALL_ERROR;
    }
    let vaddr = get_syscall_arg(0, buffer);
    let attr = vm_attributes_t::from_word(get_syscall_arg(2, buffer));
    let vspace_root_cap = get_extra_cap_by_index(0).unwrap().cap;
    let frame_size = frame_slot.cap.get_frame_size();
    let frame_vm_rights = unsafe { core::mem::transmute(frame_slot.cap.get_frame_vm_rights()) };
    let vm_rights = maskVMRights(
        frame_vm_rights,
        seL4_CapRights_t::from_word(get_syscall_arg(1, buffer)),
    );
    if unlikely(!vspace_root_cap.is_valid_native_root()) {
        global_ops!(current_syscall_error._type = seL4_InvalidCapability);
        global_ops!(current_syscall_error.invalidCapNumber = 1);
        return exception_t::EXCEPTION_SYSCALL_ERROR;
    }
    let vspace_root = vspace_root_cap.get_pgd_base_ptr();
    let asid = vspace_root_cap.get_pgd_mapped_asid();
    let find_ret = find_vspace_for_asid(asid);
    if unlikely(find_ret.status != exception_t::EXCEPTION_NONE) {
        global_ops!(current_syscall_error._type = seL4_FailedLookup);
        global_ops!(current_syscall_error.failedLookupWasSource = 0);
        return exception_t::EXCEPTION_SYSCALL_ERROR;
    }
    if unlikely(find_ret.vspace_root.unwrap() as usize != vspace_root) {
        global_ops!(current_syscall_error._type = seL4_InvalidCapability);
        global_ops!(current_syscall_error.invalidCapNumber = 1);
        return exception_t::EXCEPTION_SYSCALL_ERROR;
    }
    if unlikely(!IS_ALIGNED!(vaddr, pageBitsForSize(frame_size))) {
        // global_var!(current_syscall_error)._type = seL4_AlignmentError;
        // Use unsafe here will cause the _type error.
        global_ops!(current_syscall_error._type = seL4_AlignmentError);
        return exception_t::EXCEPTION_SYSCALL_ERROR;
    }
    let frame_asid = frame_slot.cap.get_frame_mapped_asid();
    if frame_asid != asidInvalid {
        if frame_asid != asid {
            log::error!("[User] ARMPageMap: Attempting to remap a frame that does not belong to the passed address space");
            global_ops!(current_syscall_error._type = seL4_InvalidCapability);
            global_ops!(current_syscall_error.invalidArgumentNumber = 0);
            return exception_t::EXCEPTION_SYSCALL_ERROR;
        } else if frame_slot.cap.get_frame_mapped_address() != vaddr {
            log::error!("[User] ARMPageMap: Attempting to map frame into multiple addresses");
            global_ops!(current_syscall_error._type = seL4_InvalidArgument);
            global_ops!(current_syscall_error.invalidArgumentNumber = 2);
            return exception_t::EXCEPTION_SYSCALL_ERROR;
        }
    } else {
        if unlikely(vaddr + BIT!(pageBitsForSize(frame_size)) - 1 > USER_TOP) {
            global_ops!(current_syscall_error._type = seL4_InvalidArgument);
            global_ops!(current_syscall_error.invalidArgumentNumber = 0);
            return exception_t::EXCEPTION_SYSCALL_ERROR;
        }
    }
    // TODO: copy cap in the here. Not write slot when the address is not need to write.
    // frame_slot.cap.set_frame_mapped_asid(asid);
    // frame_slot.cap.set_frame_mapped_address(vaddr);

    let vspace_root = PGDE::new_from_pte(vspace_root);
    let base = pptr_to_paddr(frame_slot.cap.get_frame_base_ptr());
    match frame_size {
        ARM_Small_Page => {
            let lu_ret = vspace_root.lookup_pt_slot(vaddr);
            if lu_ret.status != exception_t::EXCEPTION_NONE {
                unsafe {
                    current_syscall_error._type = seL4_FailedLookup;
                    current_syscall_error.failedLookupWasSource = 0;
                }
                return exception_t::EXCEPTION_SYSCALL_ERROR;
            }
            set_thread_state(get_currenct_thread(), ThreadState::ThreadStateRestart);
            let ptSlot = convert_to_mut_type_ref::<PTE>(lu_ret.ptSlot as usize);
            invoke_small_page_map(
                vaddr,
                asid,
                frame_slot,
                makeUser3rdLevel(base, vm_rights, attr),
                ptSlot,
            )
        }
        ARM_Large_Page => {
            let lu_ret = vspace_root.lookup_pd_slot(vaddr);
            if lu_ret.status != exception_t::EXCEPTION_NONE {
                unsafe {
                    current_syscall_error._type = seL4_FailedLookup;
                    current_syscall_error.failedLookupWasSource = 0;
                }
                return exception_t::EXCEPTION_SYSCALL_ERROR;
            }
            set_thread_state(get_currenct_thread(), ThreadState::ThreadStateRestart);
            let pdSlot = convert_to_mut_type_ref::<PDE>(lu_ret.pdSlot as usize);
            invoke_large_page_map(
                vaddr,
                asid,
                frame_slot,
                make_user_2nd_level(base, vm_rights, attr),
                pdSlot,
            )
        }
        ARM_Huge_Page => {
            let lu_ret = vspace_root.lookup_pud_slot(vaddr);
            if lu_ret.status != exception_t::EXCEPTION_NONE {
                unsafe {
                    current_syscall_error._type = seL4_FailedLookup;
                    current_syscall_error.failedLookupWasSource = 0;
                }
                return exception_t::EXCEPTION_SYSCALL_ERROR;
            }
            set_thread_state(get_currenct_thread(), ThreadState::ThreadStateRestart);
            let pudSlot = convert_to_mut_type_ref::<PUDE>(lu_ret.pudSlot as usize);
            invoke_huge_page_map(
                vaddr,
                asid,
                frame_slot,
                make_user_1st_level(base, vm_rights, attr),
                pudSlot,
            )
        }
        _ => exception_t::EXCEPTION_SYSCALL_ERROR,
    }
    // if length < 3 || get_extra_cap_by_index(0).is_none() {
    //     debug!("ARMPageMap: Truncated message.");
    //     unsafe {
    //         current_syscall_error._type = seL4_TruncatedMessage;
    //     }
    //     return exception_t::EXCEPTION_SYSCALL_ERROR;
    // }
    // let vaddr = get_syscall_arg(0, buffer);
    // log::debug!("map frame: {:#x?}  frame: {:#x?}", frame_slot.cap.get_frame_mapped_address(), vaddr);
    // let attr = vm_attributes_t::from_word(get_syscall_arg(2, buffer));
    // let lvl1pt_cap = get_extra_cap_by_index(0).unwrap().cap;
    // let frame_size = frame_slot.cap.get_frame_size();
    // let frame_vm_rights = unsafe { core::mem::transmute(frame_slot.cap.get_frame_vm_rights()) };
    // let vm_rights = maskVMRights(
    //     frame_vm_rights,
    //     seL4_CapRights_t::from_word(get_syscall_arg(1, buffer)),
    // );
    // let (vspace_root, asid) = match get_vspace(&lvl1pt_cap) {
    //     Some(v) => v,
    //     _ => return exception_t::EXCEPTION_SYSCALL_ERROR,
    // };
    // if unlikely(!checkVPAlignment(frame_size, vaddr)) {
    //     unsafe {
    //         current_syscall_error._type = seL4_AlignmentError;
    //     }
    //     return exception_t::EXCEPTION_SYSCALL_ERROR;
    // }
    // let frame_asid = frame_slot.cap.get_frame_mapped_asid();
    // log::debug!("frame_asid: {:?}", frame_asid);
    // if frame_asid != asidInvalid {
    //     if frame_asid != asid {
    //         debug!("ARMPageMap: Attempting to remap a frame that does not belong to the passed address space");
    //         unsafe {
    //             current_syscall_error._type = seL4_InvalidCapability;
    //             current_syscall_error.invalidArgumentNumber = 0;
    //         }
    //         return exception_t::EXCEPTION_SYSCALL_ERROR;
    //     }
    //     if frame_slot.cap.get_frame_mapped_address() != vaddr {
    //         debug!("ARMPageMap: attempting to map frame into multiple addresses");
    //         unsafe {
    //             current_syscall_error._type = seL4_InvalidArgument;
    //             current_syscall_error.invalidArgumentNumber = 2;
    //         }
    //         return exception_t::EXCEPTION_SYSCALL_ERROR;
    //     }
    // } else {
    //     let vtop = vaddr + BIT!(pageBitsForSize(frame_size)) - 1;
    //     if unlikely(vtop >= USER_TOP) {
    //         unsafe {
    //             current_syscall_error._type = seL4_InvalidArgument;
    //             current_syscall_error.invalidArgumentNumber = 0;
    //         }
    //         return exception_t::EXCEPTION_SYSCALL_ERROR;
    //     }
    // }

    // // frame_slot.cap.set_frame_mapped_address(vaddr);
    // // frame_slot.cap.set_frame_mapped_asid(asid);

    // let base = pptr_to_paddr(frame_slot.cap.get_frame_base_ptr());
    // if frame_size == ARM_Small_Page {
    //     let lu_ret = vspace_root.lookup_pt_slot(vaddr);
    //     if lu_ret.status != exception_t::EXCEPTION_NONE {
    //         unsafe {
    //             current_syscall_error._type = seL4_FailedLookup;
    //             current_syscall_error.failedLookupWasSource = 0;
    //         }
    //         return exception_t::EXCEPTION_SYSCALL_ERROR;
    //     }
    //     set_thread_state(get_currenct_thread(), ThreadState::ThreadStateRestart);
    //     let ptSlot = convert_to_mut_type_ref::<PTE>(lu_ret.ptSlot as usize);
    //     invoke_small_page_map(
    //         vaddr,
    //         asid,
    //         frame_slot,
    //         makeUser3rdLevel(base, vm_rights, attr),
    //         ptSlot,
    //     )
    // } else if frame_size == ARM_Large_Page {
    //     let lu_ret = vspace_root.lookup_pd_slot(vaddr);
    //     if lu_ret.status != exception_t::EXCEPTION_NONE {
    //         unsafe {
    //             current_syscall_error._type = seL4_FailedLookup;
    //             current_syscall_error.failedLookupWasSource = 0;
    //         }
    //         return exception_t::EXCEPTION_SYSCALL_ERROR;
    //     }
    //     set_thread_state(get_currenct_thread(), ThreadState::ThreadStateRestart);
    //     let pdSlot = convert_to_mut_type_ref::<PDE>(lu_ret.pdSlot as usize);
    //     invoke_large_page_map(
    //         vaddr,
    //         asid,
    //         frame_slot,
    //         make_user_2nd_level(base, vm_rights, attr),
    //         pdSlot,
    //     )
    // } else if frame_size == ARM_Huge_Page {
    //     let lu_ret = vspace_root.lookup_pud_slot(vaddr);
    //     if lu_ret.status != exception_t::EXCEPTION_NONE {
    //         unsafe {
    //             current_syscall_error._type = seL4_FailedLookup;
    //             current_syscall_error.failedLookupWasSource = 0;
    //         }
    //         return exception_t::EXCEPTION_SYSCALL_ERROR;
    //     }
    //     set_thread_state(get_currenct_thread(), ThreadState::ThreadStateRestart);
    //     let pudSlot = convert_to_mut_type_ref::<PUDE>(lu_ret.pudSlot as usize);
    //     invoke_huge_page_map(
    //         vaddr,
    //         asid,
    //         frame_slot,
    //         make_user_1st_level(base, vm_rights, attr),
    //         pudSlot,
    //     )
    // } else {
    //     return exception_t::EXCEPTION_SYSCALL_ERROR;
    // }
}

fn decode_page_table_unmap(pt_cte: &mut cte_t) -> exception_t {
    if !pt_cte.is_final_cap() {
        debug!("RISCVPageTableUnmap: cannot unmap if more than once cap exists");
        global_ops!(current_syscall_error._type = seL4_RevokeFirst);
        return exception_t::EXCEPTION_SYSCALL_ERROR;
    }
    let cap = &mut pt_cte.cap;
    // todo: in riscv here exists some more code ,but I don't know what it means and cannot find it in sel4,need check
    set_thread_state(get_currenct_thread(), ThreadState::ThreadStateRestart);

    return invoke_page_table_unmap(cap);
}

// FIXED check pgd_is_mapped
// vtable_root is pgd, not pd,
fn get_vspace(lvl1pt_cap: &cap_t) -> Option<(PTE, usize)> {
    if lvl1pt_cap.get_cap_type() != CapTag::CapPageGlobalDirectoryCap
        || lvl1pt_cap.get_pgd_is_mapped() == asidInvalid
    {
        debug!("ARMMMUInvocation: Invalid top-level PageTable.");
        unsafe {
            current_syscall_error._type = seL4_InvalidCapability;
            current_syscall_error.invalidCapNumber = 1;
        }
        return None;
    }

    let lvl1pt = lvl1pt_cap.get_pgd_base_ptr();
    let asid = lvl1pt_cap.get_pgd_mapped_asid();
    let find_ret = find_vspace_for_asid(asid);
    if find_ret.status != exception_t::EXCEPTION_NONE {
        debug!("ARMMMUInvocation: ASID lookup failed1");
        unsafe {
            current_lookup_fault = find_ret.lookup_fault.unwrap();
            current_syscall_error._type = seL4_FailedLookup;
            current_syscall_error.failedLookupWasSource = 0;
        }
        return None;
    }

    if find_ret.vspace_root.unwrap() as usize != lvl1pt {
        debug!("ARMMMUInvocation: ASID lookup failed");
        unsafe {
            current_syscall_error._type = seL4_InvalidCapability;
            current_syscall_error.invalidCapNumber = 1;
        }
        return None;
    }
    Some((PTE(lvl1pt), asid))
}

fn decode_vspace_root_invocation(
    label: MessageLabel,
    length: usize,
    cte: &mut cte_t,
    buffer: Option<&seL4_IPCBuffer>,
) -> exception_t {
    match label {
        MessageLabel::ARMVSpaceClean_Data
        | MessageLabel::ARMVSpaceInvalidate_Data
        | MessageLabel::ARMVSpaceCleanInvalidate_Data
        | MessageLabel::ARMVSpaceUnify_Instruction => {
            if length < 2 {
                debug!("VSpaceRoot Flush: Truncated message.");
                unsafe {
                    current_syscall_error._type = seL4_TruncatedMessage;
                    return exception_t::EXCEPTION_SYSCALL_ERROR;
                }
            }
            let start = get_syscall_arg(0, buffer);
            let end = get_syscall_arg(1, buffer);
            if end <= start {
                debug!("VSpaceRoot Flush: Invalid range.");
                unsafe {
                    current_syscall_error._type = seL4_InvalidArgument;
                    current_syscall_error.invalidArgumentNumber = 1;
                    return exception_t::EXCEPTION_SYSCALL_ERROR;
                }
            }
            if end > USER_TOP {
                debug!("VSpaceRoot Flush: Exceed the user addressable region.");
                unsafe { current_syscall_error._type = seL4_IllegalOperation };
                return exception_t::EXCEPTION_SYSCALL_ERROR;
            }
            if !cte.cap.is_valid_native_root() {
                unsafe {
                    current_syscall_error._type = seL4_InvalidCapability;
                    current_syscall_error.invalidCapNumber = 0
                };
                return exception_t::EXCEPTION_SYSCALL_ERROR;
            }
			let vspace_root = PGDE::new_from_pte(cte.cap.get_pgd_base_ptr());
			let asid=cte.cap.get_asid_base();
			let find_ret = find_vspace_for_asid(asid);
			if find_ret.status != exception_t::EXCEPTION_NONE{
				debug!("VSpaceRoot Flush: No VSpace for ASID");
				unsafe {
					current_syscall_error._type = seL4_FailedLookup;
					current_syscall_error.failedLookupWasSource = 0;
					return exception_t::EXCEPTION_SYSCALL_ERROR;
				}
			}
			if find_ret.vspace_root.unwrap() as usize  != vspace_root.get_ptr(){
				debug!("VSpaceRoot Flush: Invalid VSpace Cap");
				unsafe{
					current_syscall_error._type = seL4_InvalidCapability;
					current_syscall_error.invalidCapNumber = 0;
				}
				return exception_t::EXCEPTION_SYSCALL_ERROR;
			}
			let resolve_ret = vspace_root.lookup_frame(start);
			if !resolve_ret.valid {
				get_currenct_thread().set_state(ThreadState::ThreadStateRestart);
				return exception_t::EXCEPTION_NONE;
			}
			let page_base_start = start & !MASK!(pageBitsForSize(resolve_ret.frameSize));
			let page_base_end = (end-1) & !MASK!(pageBitsForSize(resolve_ret.frameSize));
			if page_base_start != page_base_end{
				unsafe{
					current_syscall_error._type=seL4_RangeError;
					current_syscall_error.rangeErrorMin = start;
					current_syscall_error.rangeErrorMax = page_base_start + MASK!(pageBitsForSize(resolve_ret.frameSize));
				}
				return exception_t::EXCEPTION_SYSCALL_ERROR;
			}
			let pstart = resolve_ret.frameBase + start & MASK!(pageBitsForSize(resolve_ret.frameSize));
			get_currenct_thread().set_state(ThreadState::ThreadStateRestart);
			return decode_vspace_flush_invocation(label,find_ret.vspace_root.unwrap() as usize,asid,start,end,paddr_t::from(pstart));
        }
        _ => {
            unsafe { current_syscall_error._type = seL4_IllegalOperation };
            return exception_t::EXCEPTION_SYSCALL_ERROR;
        }
    }
}

fn decode_vspace_flush_invocation(
	label: MessageLabel,
	vspace:usize,
	asid:asid_t,
	start:vptr_t,
	end:vptr_t,
	pstart:paddr_t,
) ->exception_t{
	if start < end{		
		let root_switched = set_vm_root_for_flush(vspace, asid);
		log::warn!(
            "need to flush cache for decode_page_clean_invocation label: {:?}",
            label
        );
		todo!();
		// if root_switched {
		// 	set_vm_root(vspace);
		// }
	}
    exception_t::EXCEPTION_NONE
}

fn decode_page_upper_directory_invocation(
    label: MessageLabel,
    length: usize,
    cte: &mut cte_t,
    buffer: Option<&seL4_IPCBuffer>,
) -> exception_t {
    /*
        lookupPGDSlot_ret_t pgdSlot;
        findVSpaceForASID_ret_t find_ret;
        if (invLabel == ARMPageUpperDirectoryUnmap) {
            if (unlikely(!isFinalCapability(cte))) {
                current_syscall_error.type = seL4_RevokeFirst;
                return EXCEPTION_SYSCALL_ERROR;
            }
            setThreadState(NODE_STATE(ksCurThread), ThreadState_Restart);
            return performUpperPageDirectoryInvocationUnmap(cap, cte);
        }
    */
    if label == MessageLabel::ARMPageUpperDirectoryUnmap {
        log::warn!("Need to check is FinalCapability here");
        get_currenct_thread().set_state(ThreadState::ThreadStateRestart);
        unimplemented!("performUpperPageDirectoryInvocationUnmap");
    }

    // Return SYSCALL_ERROR if message is not ARMPageUpperDirectoryUnmap
    if unlikely(label != MessageLabel::ARMPageUpperDirectoryMap) {
        global_ops!(current_syscall_error._type = seL4_IllegalOperation);
        return exception_t::EXCEPTION_SYSCALL_ERROR;
    }

    if unlikely(length < 2 || unsafe { current_extra_caps.excaprefs[0] == 0 }) {
        global_ops!(current_syscall_error._type = seL4_TruncatedMessage);
        return exception_t::EXCEPTION_SYSCALL_ERROR;
    }
    if unlikely(cte.cap.get_pud_is_mapped() == 1) {
        global_ops!(current_syscall_error._type = seL4_InvalidCapability);
        global_ops!(current_syscall_error.invalidCapNumber = 0);
        return exception_t::EXCEPTION_SYSCALL_ERROR;
    }
    let vaddr = get_syscall_arg(0, buffer) & (!MASK!(PGD_INDEX_OFFSET));
    let pgd_cap = convert_to_mut_type_ref::<cap_t>(global_ops!(current_extra_caps.excaprefs[0]));

    if unlikely(!pgd_cap.is_valid_native_root()) {
        global_ops!(current_syscall_error._type = seL4_InvalidCapability);
        global_ops!(current_syscall_error.invalidCapNumber = 1);
        return exception_t::EXCEPTION_SYSCALL_ERROR;
    }

    let pgd = pgd_cap.get_pgd_base_ptr();
    let asid = pgd_cap.get_pgd_mapped_asid();

    if unlikely(vaddr > USER_TOP) {
        global_ops!(current_syscall_error._type = seL4_InvalidArgument);
        global_ops!(current_syscall_error.failedLookupWasSource = 0);
        return exception_t::EXCEPTION_SYSCALL_ERROR;
    }

    let find_ret = find_vspace_for_asid(asid);

    if unlikely(find_ret.status != exception_t::EXCEPTION_NONE) {
        global_ops!(current_syscall_error._type = seL4_FailedLookup);
        global_ops!(current_syscall_error.failedLookupWasSource = 0);
        return exception_t::EXCEPTION_SYSCALL_ERROR;
    }
    // vspace_root is Some(_) when Exception is NONE
    if unlikely(find_ret.vspace_root.unwrap() as usize != pgd) {
        global_ops!(current_syscall_error._type = seL4_InvalidCapability);
        global_ops!(current_syscall_error.invalidCapNumber = 1);
        return exception_t::EXCEPTION_SYSCALL_ERROR;
    }
    // Ensure that pgd is aligned 4K.
    assert!(pgd & MASK!(PAGE_BITS) == 0);

    let pgd_slot = PGDE::new_from_pte(pgd).lookup_pgd_slot(vaddr);

    if unlikely(ptr_to_ref(pgd_slot.pgdSlot).get_present()) {
        global_ops!(current_syscall_error._type = seL4_DeleteFirst);
        return exception_t::EXCEPTION_SYSCALL_ERROR;
    }
    // TODO: make 0x3 in a pagetable-specific position
    let pgde = PGDE::new_page(pptr_to_paddr(cte.cap.get_pud_base_ptr()), 0x3);
    cte.cap.set_pud_is_mapped(1);
    cte.cap.set_pud_mapped_asid(asid);
    cte.cap.set_pud_mapped_address(vaddr);

    get_currenct_thread().set_state(ThreadState::ThreadStateRestart);
    *ptr_to_mut(pgd_slot.pgdSlot) = pgde;
    log::warn!("Need to clean D-Cache using cleanByVA_PoU");
    exception_t::EXCEPTION_NONE
}
fn decode_page_directory_invocation(
    label: MessageLabel,
    length: usize,
    cte: &mut cte_t,
    buffer: Option<&seL4_IPCBuffer>,
) -> exception_t {
    /*
        if (invLabel == ARMPageDirectoryUnmap) {
            if (unlikely(!isFinalCapability(cte))) {
                current_syscall_error.type = seL4_RevokeFirst;
                return EXCEPTION_SYSCALL_ERROR;
            }
            setThreadState(NODE_STATE(ksCurThread), ThreadState_Restart);
            return performPageDirectoryInvocationUnmap(cap, cte);
        }
    */
    // Call performPageDirectoryInvocationUnmap if message is unmap
    if label == MessageLabel::ARMPageDirectoryUnmap {
        log::warn!("Need to check is FinalCapability here");
        get_currenct_thread().set_state(ThreadState::ThreadStateRestart);
        unimplemented!("performPageDirectoryInvocationUnmap");
    }
    // Return SYSCALL_ERROR if message is not ARMPageDirectoryUnmap
    if unlikely(label != MessageLabel::ARMPageDirectoryMap) {
        global_ops!(current_syscall_error._type = seL4_IllegalOperation);
        return exception_t::EXCEPTION_SYSCALL_ERROR;
    }
    if unlikely(length < 2 || global_ops!(current_extra_caps.excaprefs[0] == 0)) {
        global_ops!(current_syscall_error._type = seL4_TruncatedMessage);
        return exception_t::EXCEPTION_SYSCALL_ERROR;
    }
    if unlikely(cte.cap.get_pd_is_mapped() == 1) {
        global_ops!(current_syscall_error._type = seL4_InvalidCapability);
        global_ops!(current_syscall_error.invalidCapNumber = 0);
        return exception_t::EXCEPTION_SYSCALL_ERROR;
    }

    let vaddr = get_syscall_arg(0, buffer) & (!MASK!(PUD_INDEX_OFFSET));
    let vspace_root_cap =
        convert_to_mut_type_ref::<cap_t>(global_ops!(current_extra_caps.excaprefs[0]));

    if unlikely(!vspace_root_cap.is_valid_native_root()) {
        global_ops!(current_syscall_error._type = seL4_InvalidCapability);
        global_ops!(current_syscall_error.invalidCapNumber = 1);
        return exception_t::EXCEPTION_SYSCALL_ERROR;
    }

    let vspace_root = vspace_root_cap.get_pgd_base_ptr();
    let asid = vspace_root_cap.get_pgd_mapped_asid();

    if unlikely(vaddr > USER_TOP) {
        global_ops!(current_syscall_error._type = seL4_InvalidArgument);
        global_ops!(current_syscall_error.failedLookupWasSource = 0);
        return exception_t::EXCEPTION_SYSCALL_ERROR;
    }

    let find_ret = find_vspace_for_asid(asid);

    if unlikely(find_ret.status != exception_t::EXCEPTION_NONE) {
        global_ops!(current_syscall_error._type = seL4_FailedLookup);
        global_ops!(current_syscall_error.failedLookupWasSource = 0);
        return exception_t::EXCEPTION_SYSCALL_ERROR;
    }
    if unlikely(find_ret.vspace_root.unwrap() as usize != vspace_root) {
        global_ops!(current_syscall_error._type = seL4_InvalidCapability);
        global_ops!(current_syscall_error.invalidCapNumber = 1);
        return exception_t::EXCEPTION_SYSCALL_ERROR;
    }

    let pud_slot = PGDE::new_from_pte(vspace_root).lookup_pud_slot(vaddr);

    if pud_slot.status != exception_t::EXCEPTION_NONE {
        global_ops!(current_syscall_error._type = seL4_FailedLookup);
        global_ops!(current_syscall_error.failedLookupWasSource = 0);
        return exception_t::EXCEPTION_SYSCALL_ERROR;
    }
    if unlikely(
        ptr_to_ref(pud_slot.pudSlot).get_present() || ptr_to_ref(pud_slot.pudSlot).is_1g_page(),
    ) {
        global_ops!(current_syscall_error._type = seL4_DeleteFirst);
        return exception_t::EXCEPTION_SYSCALL_ERROR;
    }
    // TODO: make 0x3 in a pagetable-specific position
    let pude = PUDE::new_page(pptr_to_paddr(cte.cap.get_pd_base_ptr()), 0x3);
    cte.cap.set_pd_is_mapped(1);
    cte.cap.set_pd_mapped_asid(asid);
    cte.cap.set_pd_mapped_address(vaddr);
    get_currenct_thread().set_state(ThreadState::ThreadStateRestart);
    *ptr_to_mut(pud_slot.pudSlot) = pude;
    log::warn!("Need to clean D-Cache using cleanByVA_PoU");
    exception_t::EXCEPTION_NONE
}

pub(crate) fn check_irq(irq: usize) -> exception_t {
    if irq > maxIRQ {
        unsafe {
            current_syscall_error._type = seL4_RangeError;
            current_syscall_error.rangeErrorMin = 0;
            current_syscall_error.rangeErrorMax = maxIRQ;
            debug!(
                "Rejecting request for IRQ {}. IRQ is out of range [1..maxIRQ].",
                irq
            );
            return exception_t::EXCEPTION_SYSCALL_ERROR;
        }
    }
    exception_t::EXCEPTION_NONE
}

pub fn arch_decode_irq_control_invocation(
    label: MessageLabel,
    length: usize,
    src_slot: &mut cte_t,
    buffer: Option<&seL4_IPCBuffer>,
) -> exception_t {
    if label == MessageLabel::ARMIRQIssueIRQHandlerTrigger {
        if length < 4 || get_extra_cap_by_index(0).is_none() {
            unsafe {
                current_syscall_error._type = seL4_TruncatedMessage;
            }
            return exception_t::EXCEPTION_SYSCALL_ERROR;
        }
        let irq = get_syscall_arg(0, buffer);
        let _trigger = get_syscall_arg(1, buffer) != 0;
        let index = get_syscall_arg(2, buffer);
        let depth = get_syscall_arg(3, buffer);
        let cnode_cap = get_extra_cap_by_index(0).unwrap().cap;
        let status = check_irq(irq);
        if status != exception_t::EXCEPTION_NONE {
            return status;
        }
        if is_irq_active(irq) {
            unsafe {
                current_syscall_error._type = seL4_RevokeFirst;
            }
            debug!("Rejecting request for IRQ {}. Already active.", irq);
            return exception_t::EXCEPTION_SYSCALL_ERROR;
        }
        let lu_ret = lookupSlotForCNodeOp(false, &cnode_cap, index, depth);
        if lu_ret.status != exception_t::EXCEPTION_NONE {
            debug!("Target slot for new IRQ Handler cap invalid: IRQ {}.", irq);
            return lu_ret.status;
        }
        set_thread_state(get_currenct_thread(), ThreadState::ThreadStateRestart);
        invoke_irq_control(
            irq,
            convert_to_mut_type_ref::<cte_t>(lu_ret.slot as usize),
            src_slot,
        )
    } else {
        unsafe {
            current_syscall_error._type = seL4_IllegalOperation;
        }
        return exception_t::EXCEPTION_SYSCALL_ERROR;
    }
}
