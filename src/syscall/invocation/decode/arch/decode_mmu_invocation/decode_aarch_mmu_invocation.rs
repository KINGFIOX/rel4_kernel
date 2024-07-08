use crate::syscall::invocation::decode::current_syscall_error;
use log::debug;
use sel4_common::sel4_config::{seL4_IllegalOperation,seL4_RevokeFirst,seL4_TruncatedMessage,seL4_InvalidCapability};
use sel4_common::{
    arch::MessageLabel,
    structures::{exception_t, seL4_IPCBuffer},
};
use sel4_cspace::interface::{cap_t, cte_t, CapTag};
use crate::syscall::{set_thread_state,unlikely,get_syscall_arg};
use crate::syscall::ThreadState;
use crate::syscall::invocation::invoke_mmu_op::invoke_page_table_unmap;
use crate::kernel::boot::get_extra_cap_by_index;
use crate::config::USER_TOP;
use sel4_common::sel4_config::seL4_InvalidArgument;
use crate::syscall::get_currenct_thread;

pub fn decode_mmu_invocation(
    label: MessageLabel,
    length: usize,
    slot: &mut cte_t,
    call: bool,
    buffer: Option<&seL4_IPCBuffer>,
) -> exception_t {
    match slot.cap.get_cap_type() {
		CapTag::CapPageGlobalDirectoryCap => decode_vspace_root_invocation(label,length,slot,buffer),
		CapTag::CapPageUpperDirectoryCap => decode_page_upper_directory_invocation(label,length,slot,buffer),
		CapTag::CapPageDirectoryCap => decode_page_directory_invocation(label,length,slot,buffer),
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
    exception_t::EXCEPTION_NONE
}

fn decode_asid_control(
    label: MessageLabel,
    length: usize,
    buffer: Option<&seL4_IPCBuffer>,
) -> exception_t {
    exception_t::EXCEPTION_NONE
}

fn decode_asid_pool(label: MessageLabel, cte: &mut cte_t) -> exception_t {
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
    set_thread_state(get_currenct_thread(), ThreadState::ThreadStateRestart);
	let cap = &mut pt_cte.cap;
	// todo: in riscv here exists some more code ,but I don't know what it means and cannot find it in sel4,need check
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

	let lvl1pt_cap = get_extra_cap_by_index(0).unwrap().cap;



    if unlikely(vaddr >= USER_TOP) {
        debug!("ARMPageTableMap: Virtual address cannot be in kernel window.");
        unsafe {
            current_syscall_error._type = seL4_InvalidArgument;
            current_syscall_error.invalidCapNumber = 0;
        }
        return exception_t::EXCEPTION_SYSCALL_ERROR;
    }

    exception_t::EXCEPTION_NONE
}
fn decode_vspace_root_invocation(
	label: MessageLabel,
    length: usize,
    cte: &mut cte_t,
    buffer: Option<&seL4_IPCBuffer>
) -> exception_t {
	exception_t::EXCEPTION_NONE
}

fn decode_page_upper_directory_invocation(
	label: MessageLabel,
    length: usize,
    cte: &mut cte_t,
    buffer: Option<&seL4_IPCBuffer>
) -> exception_t {
	exception_t::EXCEPTION_NONE
}
fn decode_page_directory_invocation(
	label: MessageLabel,
    length: usize,
    cte: &mut cte_t,
    buffer: Option<&seL4_IPCBuffer>
) -> exception_t {
	exception_t::EXCEPTION_NONE
}