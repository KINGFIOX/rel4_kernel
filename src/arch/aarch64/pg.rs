use sel4_common::structures::exception_t;
use sel4_cspace::interface::{cap_t, cte_t};
use sel4_vspace::{pte_t, vptr_t};

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
extern "C" fn lookupPGDSlot(vspace: *mut pte_t, vptr: vptr_t) -> lookupPGDSlot_ret_t {
    todo!("lookupPGDSlot")
}

#[no_mangle]
extern "C" fn lookupPDSlot(vspace: *mut pte_t, vptr: vptr_t) -> lookupPDSlot_ret_t {
    todo!("lookupPDSlot")
}

#[no_mangle]
extern "C" fn lookupPUDSlot(vspace: *mut pte_t, vptr: vptr_t) -> lookupPUDSlot_ret_t {
    todo!("lookupPUDSlot")
}

#[no_mangle]
// typedef word_t cptr_t;
extern "C" fn decodeARMMMUInvocation(
    invLabel: usize,
    length: usize,
    cptr: usize,
    cte: *mut cte_t,
    cap: cap_t,
    call: bool,
    buffer: *mut usize,
) -> exception_t {
    todo!("decodeARMMMUInvocation")
}
