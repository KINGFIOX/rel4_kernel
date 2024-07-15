use crate::syscall::invocation::decode::arch::decode_mmu_invocation;
use sel4_common::arch::MessageLabel;
use sel4_common::structures::exception_t;
use sel4_common::structures::seL4_IPCBuffer;
use sel4_cspace::interface::{cap_t, cte_t};
use sel4_vspace::{PTE, vptr_t};

#[repr(C)]
struct lookupPGDSlot_ret_t {
    status: exception_t,
    pgdSlot: usize, // *mut pgde_t
}

#[repr(C)]
struct lookupPDSlot_ret_t {
    status: exception_t,
    pdSlot: usize, // *mut pde_t
}

#[repr(C)]
struct lookupPUDSlot_ret_t {
    status: exception_t,
    pudSlot: usize, // *mut pude_t
}

#[no_mangle]
extern "C" fn lookupPGDSlot(vspace: *mut PTE, vptr: vptr_t) -> lookupPGDSlot_ret_t {
    // which is realized under sel4_vspace/src/arch/aarch64/pte.rs as a member function of PTE in this commit
    // ZhiyuanSue
    todo!("lookupPGDSlot")
}

#[no_mangle]
extern "C" fn lookupPDSlot(vspace: *mut PTE, vptr: vptr_t) -> lookupPDSlot_ret_t {
    // which is realized under sel4_vspace/src/arch/aarch64/pte.rs as a member function of PTE in this commit
    // ZhiyuanSue
    todo!("lookupPDSlot")
}

#[no_mangle]
extern "C" fn lookupPUDSlot(vspace: *mut PTE, vptr: vptr_t) -> lookupPUDSlot_ret_t {
    // which is realized under sel4_vspace/src/arch/aarch64/pte.rs as a member function of PTE in this commit
    // ZhiyuanSue
    todo!("lookupPUDSlot")
}

#[no_mangle]
// typedef word_t cptr_t;
extern "C" fn decodeARMMMUInvocation(
    invLabel: MessageLabel,
    length: usize,
    cptr: usize,
    cte: &mut cte_t,
    cap: cap_t,
    call: bool,
    buffer: Option<&seL4_IPCBuffer>,
) -> exception_t {
    // todo!("decodeARMMMUInvocation")
    decode_mmu_invocation(invLabel, length, cte, call, buffer)
}
