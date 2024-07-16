use sel4_common::sel4_config::tcbVTable;
use sel4_task::get_currenct_thread;
use sel4_vspace::{asid_t, set_vm_root_for_flush_with_thread_root, PTE};

#[no_mangle]
pub fn setVMRootForFlush(vspace: *mut PTE, asid: asid_t) -> bool {
    set_vm_root_for_flush_with_thread_root(
        vspace,
        asid,
        &get_currenct_thread().get_cspace(tcbVTable).cap,
    )
}
