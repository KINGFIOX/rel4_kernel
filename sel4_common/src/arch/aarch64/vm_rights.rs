use crate::cap_rights::seL4_CapRights_t;

#[repr(usize)]
#[derive(PartialEq, Eq)]
pub enum vm_rights_t {
    VMKernelOnly = 0,
    VMReadWrite = 1,
    VMReadOnly = 3,
}

pub fn maskVMRights(vm_rights: vm_rights_t, rights: seL4_CapRights_t) -> vm_rights_t {
    if vm_rights == vm_rights_t::VMReadOnly && rights.get_allow_read() != 0 {
        return vm_rights_t::VMReadOnly;
    }
    if vm_rights == vm_rights_t::VMReadWrite && rights.get_allow_read() != 0 {
        if rights.get_allow_write() != 0 {
            return vm_rights_t::VMReadWrite;
        } else {
            return vm_rights_t::VMReadOnly;
        }
    }
    vm_rights_t::VMKernelOnly
}
