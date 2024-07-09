use crate::config::USER_TOP;
use crate::kernel::boot::get_extra_cap_by_index;
use crate::syscall::get_currenct_thread;
use crate::syscall::invocation::decode::current_syscall_error;
use crate::syscall::ThreadState;
use crate::syscall::{current_lookup_fault, get_syscall_arg, set_thread_state, unlikely};
use log::debug;
use sel4_common::sel4_config::{asidInvalid, seL4_FailedLookup, seL4_RangeError};
use sel4_common::sel4_config::{seL4_DeleteFirst, seL4_InvalidArgument};
use sel4_common::sel4_config::{
    seL4_IllegalOperation, seL4_InvalidCapability, seL4_RevokeFirst, seL4_TruncatedMessage,
    PD_INDEX_OFFSET,
};
use sel4_common::utils::convert_to_mut_type_ref;
use sel4_common::{
    arch::MessageLabel,
    structures::{exception_t, seL4_IPCBuffer},
    MASK,
};
use sel4_cspace::interface::{cap_t, cte_t, CapTag};
use sel4_vspace::{find_vspace_for_asid, pte_t};

use crate::syscall::invocation::invoke_mmu_op::{
    invoke_asid_control, invoke_asid_pool, invoke_page_get_address, invoke_page_map,
    invoke_page_table_map, invoke_page_table_unmap, invoke_page_unmap,
};
use crate::{
    config::maxIRQ,
    interrupt::is_irq_active,
    syscall::{invocation::invoke_irq::invoke_irq_control, irqInvalid, lookupSlotForCNodeOp},
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
    match label {
        MessageLabel::ARMPageTableUnmap => decode_page_table_unmap(cte),

        MessageLabel::ARMPageTableMap => decode_page_table_map(length, cte, buffer),
        _ => {
            debug!("RISCVPageTable: Illegal Operation");
            unsafe {
                current_syscall_error._type = seL4_IllegalOperation;
            }
            return exception_t::EXCEPTION_SYSCALL_ERROR;
        }
    }
}

fn decode_frame_invocation(
    label: MessageLabel,
    length: usize,
    frame_slot: &mut cte_t,
    call: bool,
    buffer: Option<&seL4_IPCBuffer>,
) -> exception_t {
    match label {
        MessageLabel::ARMPageMap => exception_t::EXCEPTION_NONE,
        MessageLabel::ARMPageUnmap => {
            set_thread_state(get_currenct_thread(), ThreadState::ThreadStateRestart);
            invoke_page_unmap(frame_slot)
        }
        MessageLabel::ARMPageClean_Data
        | MessageLabel::ARMPageInvalidate_Data
        | MessageLabel::ARMPageCleanInvalidate_Data
        | MessageLabel::ARMPageUnify_Instruction => exception_t::EXCEPTION_NONE,
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
    todo!();
    exception_t::EXCEPTION_NONE
}

fn decode_asid_pool(label: MessageLabel, cte: &mut cte_t) -> exception_t {
    todo!();
    exception_t::EXCEPTION_NONE
}

fn decode_page_table_unmap(pt_cte: &mut cte_t) -> exception_t {
    if !pt_cte.is_final_cap() {
        debug!("RISCVPageTableUnmap: cannot unmap if more than once cap exists");
        unsafe {
            current_syscall_error._type = seL4_RevokeFirst;
        }
        return exception_t::EXCEPTION_SYSCALL_ERROR;
    }
    let cap = &mut pt_cte.cap;
    // todo: in riscv here exists some more code ,but I don't know what it means and cannot find it in sel4,need check
    set_thread_state(get_currenct_thread(), ThreadState::ThreadStateRestart);

    return invoke_page_table_unmap(cap);
}
fn decode_page_table_map(
    length: usize,
    pt_cte: &mut cte_t,
    buffer: Option<&seL4_IPCBuffer>,
) -> exception_t {
    if unlikely(length < 2 || get_extra_cap_by_index(0).is_none()) {
        debug!("ARMPageTableMap: truncated message");
        unsafe {
            current_syscall_error._type = seL4_TruncatedMessage;
        }
        return exception_t::EXCEPTION_SYSCALL_ERROR;
    }
    let cap = &mut pt_cte.cap;
    if unlikely(cap.get_pt_is_mapped() != 0) {
        debug!("ARMPageTable: PageTable is already mapped.");
        unsafe {
            current_syscall_error._type = seL4_InvalidCapability;
            current_syscall_error.invalidCapNumber = 0;
        }
        return exception_t::EXCEPTION_SYSCALL_ERROR;
    }

    let vaddr = get_syscall_arg(0, buffer);
    if unlikely(vaddr >= USER_TOP) {
        debug!("ARMPageTableMap: Virtual address cannot be in kernel window.");
        unsafe {
            current_syscall_error._type = seL4_InvalidArgument;
            current_syscall_error.invalidCapNumber = 0;
        }
        return exception_t::EXCEPTION_SYSCALL_ERROR;
    }
    let lvl1pt_cap = get_extra_cap_by_index(0).unwrap().cap;

    if let Some((lvl1pt, asid)) = get_vspace(&lvl1pt_cap) {
        let pd_ret = lvl1pt.lookup_pd_slot(vaddr);
        if pd_ret.status != exception_t::EXCEPTION_NONE {
            debug!("ARMPageTableMap: Invalid pd Slot");
            unsafe {
                current_syscall_error._type = seL4_FailedLookup;
                current_syscall_error.failedLookupWasSource = 0;
            }
            return exception_t::EXCEPTION_SYSCALL_ERROR;
        }
        if unsafe { ((*pd_ret.pdSlot).0 & 0x3) != 3 || ((*pd_ret.pdSlot).0 & 0x3) != 1 } {
            debug!("RISCVPageTableMap: All objects mapped at this address");
            unsafe {
                current_syscall_error._type = seL4_DeleteFirst;
            }
            return exception_t::EXCEPTION_SYSCALL_ERROR;
        }
        let pdSlot = convert_to_mut_type_ref::<pte_t>(pd_ret.pdSlot as usize);
        set_thread_state(get_currenct_thread(), ThreadState::ThreadStateRestart);
        return invoke_page_table_map(cap, pdSlot, asid, vaddr & !MASK!(PD_INDEX_OFFSET));
    } else {
        return exception_t::EXCEPTION_SYSCALL_ERROR;
    }
}

fn get_vspace(lvl1pt_cap: &cap_t) -> Option<(&mut pte_t, usize)> {
    if lvl1pt_cap.get_cap_type() != CapTag::CapPageTableCap
        || lvl1pt_cap.get_pt_is_mapped() == asidInvalid
    {
        debug!("ARMMMUInvocation: Invalid top-level PageTable.");
        unsafe {
            current_syscall_error._type = seL4_InvalidCapability;
            current_syscall_error.invalidCapNumber = 1;
        }
        return None;
    }

    let lvl1pt = convert_to_mut_type_ref::<pte_t>(lvl1pt_cap.get_pt_base_ptr());
    let asid = lvl1pt_cap.get_pt_mapped_asid();

    let find_ret = find_vspace_for_asid(asid);
    if find_ret.status != exception_t::EXCEPTION_NONE {
        debug!("ARMMMUInvocation: ASID lookup failed");
        unsafe {
            current_lookup_fault = find_ret.lookup_fault.unwrap();
            current_syscall_error._type = seL4_FailedLookup;
            current_syscall_error.failedLookupWasSource = 0;
        }
        return None;
    }

    if find_ret.vspace_root.unwrap() as usize != lvl1pt.get_ptr() {
        debug!("ARMMMUInvocation: ASID lookup failed");
        unsafe {
            current_syscall_error._type = seL4_InvalidCapability;
            current_syscall_error.invalidCapNumber = 1;
        }
        return None;
    }
    Some((lvl1pt, asid))
}

fn decode_vspace_root_invocation(
    label: MessageLabel,
    length: usize,
    cte: &mut cte_t,
    buffer: Option<&seL4_IPCBuffer>,
) -> exception_t {
    todo!();
    exception_t::EXCEPTION_NONE
}

fn decode_page_upper_directory_invocation(
    label: MessageLabel,
    length: usize,
    cte: &mut cte_t,
    buffer: Option<&seL4_IPCBuffer>,
) -> exception_t {
    todo!();
    exception_t::EXCEPTION_NONE
}
fn decode_page_directory_invocation(
    label: MessageLabel,
    length: usize,
    cte: &mut cte_t,
    buffer: Option<&seL4_IPCBuffer>,
) -> exception_t {
    todo!();
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
