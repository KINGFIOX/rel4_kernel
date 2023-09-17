
use crate::{
    config::{
        tcbCNodeEntries, IRQInactive,
    },
    kernel::{
        boot::current_lookup_fault,
        thread::doReplyTransfer,
        vspace::{
            deleteASID, deleteASIDPool,
        },
    }, syscall::safe_unbind_notification,
};

use task_manager::*;
use ipc::*;
use vspace::*;
use cspace::compatibility::*;
use super::{
    endpoint::cancelAllIPC,
    interrupt::{
        deletingIRQHandler, setIRQState,
    },
    notification::cancelAllSignals
};

use common::{structures::exception_t, sel4_config::*, object::*, utils::convert_to_mut_type_ref};
use cspace::interface::*;

#[no_mangle]
pub fn Arch_finaliseCap(cap: &cap_t, final_: bool) -> finaliseCap_ret {
    let mut fc_ret = finaliseCap_ret::default();
    match cap.get_cap_type() {
        CapTag::CapFrameCap => {
            if cap.get_frame_mapped_asid() != 0 {
                match unmapPage(cap.get_frame_size(), cap.get_frame_mapped_asid(), 
                    cap.get_frame_mapped_address(), cap.get_frame_base_ptr()) {
                    Err(lookup_fault) => {
                        unsafe {
                            current_lookup_fault = lookup_fault
                        }
                    }
                    _ => {} 
                }
            }
        }

        CapTag::CapPageTableCap => {
            if final_ && cap.get_pt_is_mapped() != 0 {
                let asid = cap.get_pt_mapped_asid();
                let find_ret = find_vspace_for_asid(asid);
                let pte = cap.get_pt_base_ptr();
                if find_ret.status == exception_t::EXCEPTION_NONE && find_ret.vspace_root.unwrap() as usize == pte {
                    deleteASID(asid, pte as *mut pte_t);
                } else {
                    convert_to_mut_type_ref::<pte_t>(pte).unmap_page_table(asid, cap.get_pt_mapped_address());
                }
                if let Some(lookup_fault) = find_ret.lookup_fault {
                    unsafe {
                        current_lookup_fault = lookup_fault;
                    }
                }
            }
        }

        CapTag::CapASIDPoolCap => {
            if final_ {
                deleteASIDPool(
                    cap.get_asid_base(),
                    cap.get_asid_pool() as *mut asid_pool_t,
                );
            }
        }
        _ => {}
    }
    fc_ret.remainder = cap_t::new_null_cap();
    fc_ret.cleanupInfo = cap_t::new_null_cap();
    fc_ret
}

#[link(name = "kernel_all.c")]
extern "C" {
    fn tcbDebugRemove(tcb: *mut tcb_t);
    fn tcbDebugAppend(tcb: *mut tcb_t);
}
#[no_mangle]
pub fn finaliseCap(cap: &cap_t, _final: bool, _exposed: bool) -> finaliseCap_ret {
    let mut fc_ret = finaliseCap_ret::default();

    if cap.isArchCap() {
        return Arch_finaliseCap(cap, _final);
    }
    match cap.get_cap_type() {
        CapTag::CapEndpointCap => {
            if _final {
                cancelAllIPC(cap.get_ep_ptr() as *mut endpoint_t);
            }
            fc_ret.remainder = cap_t::new_null_cap();
            fc_ret.cleanupInfo = cap_t::new_null_cap();
            return fc_ret;
        }
        CapTag::CapNotificationCap => {
            if _final {
                let ntfn =  convert_to_mut_type_ref::<notification_t>(cap.get_nf_ptr());
                ntfn.safe_unbind_tcb();
                cancelAllSignals(ntfn);
            }
            fc_ret.remainder = cap_t::new_null_cap();
            fc_ret.cleanupInfo = cap_t::new_null_cap();
            return fc_ret;
        }
        CapTag::CapReplyCap | CapTag::CapNullCap | CapTag::CapDomainCap => {
            fc_ret.remainder = cap_t::new_null_cap();
            fc_ret.cleanupInfo = cap_t::new_null_cap();
            return fc_ret;
        }
        _ => {
            if _exposed {
                panic!("finaliseCap: failed to finalise immediately.");
            }
        }
    }

    match cap.get_cap_type() {
        CapTag::CapCNodeCap => {
            if _final {
                fc_ret.remainder = Zombie_new(
                    1usize << cap.get_cnode_radix(),
                    cap.get_cnode_radix(),
                    cap.get_cnode_ptr(),
                );
                fc_ret.cleanupInfo = cap_t::new_null_cap();
                return fc_ret;
            } else {
                fc_ret.remainder = cap_t::new_null_cap();
                fc_ret.cleanupInfo = cap_t::new_null_cap();
                return fc_ret;
            }
        }
        CapTag::CapThreadCap => {
            if _final {
                let tcb = convert_to_mut_type_ref::<tcb_t>(cap.get_tcb_ptr());
                let cte_ptr = tcb.get_cspace_mut_ref(tcbCTable);
                safe_unbind_notification(tcb);
                cancel_ipc(tcb);
                tcb.suspend();
                unsafe {
                    tcbDebugRemove(tcb as *mut tcb_t);
                }
                fc_ret.remainder =
                    Zombie_new(tcbCNodeEntries, ZombieType_ZombieTCB, cte_ptr.get_ptr());
                fc_ret.cleanupInfo = cap_t::new_null_cap();
                return fc_ret;
            }
        }
        CapTag::CapZombieCap => {
            fc_ret.remainder = cap.clone();
            fc_ret.cleanupInfo = cap_t::new_null_cap();
            return fc_ret;
        }
        CapTag::CapIrqHandlerCap => {
            if _final {
                let irq = cap.get_irq_handler();
                deletingIRQHandler(irq);
                fc_ret.remainder = cap_t::new_null_cap();
                fc_ret.cleanupInfo = cap.clone();
                return fc_ret;
            }
        }
        _ => {
            fc_ret.remainder = cap_t::new_null_cap();
            fc_ret.cleanupInfo = cap_t::new_null_cap();
            return fc_ret;
        }
    }
    fc_ret.remainder = cap_t::new_null_cap();
    fc_ret.cleanupInfo = cap_t::new_null_cap();
    return fc_ret;
}

#[no_mangle]
pub fn post_cap_deletion(cap: &cap_t) {
    if cap_get_capType(cap) == cap_irq_handler_cap {
        let irq = cap.get_irq_handler();
        setIRQState(IRQInactive, irq);
    }
}

pub fn hasCancelSendRight(cap: &cap_t) -> bool {
    match cap_get_capType(cap) {
        cap_endpoint_cap => {
            cap_endpoint_cap_get_capCanSend(cap) != 0
                && cap_endpoint_cap_get_capCanReceive(cap) != 0
                && cap_endpoint_cap_get_capCanGrantReply(cap) != 0
                && cap_endpoint_cap_get_capCanGrant(cap) != 0
        }
        _ => false,
    }
}

#[no_mangle]
pub fn performInvocation_Reply(
    thread: *mut tcb_t,
    slot: *mut cte_t,
    canGrant: bool,
) -> exception_t {
    unsafe {
        doReplyTransfer(ksCurThread, thread, slot, canGrant);
    }
    exception_t::EXCEPTION_NONE
}

#[no_mangle]
pub fn Arch_isFrameType(_type: usize) -> bool {
    match _type {
        seL4_RISCV_4K_Page | seL4_RISCV_Giga_Page | seL4_RISCV_Mega_Page => true,
        _ => false,
    }
}
