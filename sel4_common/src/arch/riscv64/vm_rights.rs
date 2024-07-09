use crate::cap_rights::seL4_CapRights_t;

#[repr(usize)]
#[derive(PartialEq, Eq)]
pub enum vm_rights_t {
    VMKernelOnly = 1,
    VMReadOnly = 2,
    VMReadWrite = 3,
}

///判断应用程序是否要求页面可写
pub fn RISCVGetWriteFromVMRights(vm_rights: &vm_rights_t) -> bool {
    return *vm_rights == vm_rights_t::VMReadWrite;
}

///判断应用程序是否要求页面可读
pub fn RISCVGetReadFromVMRights(vm_rights: &vm_rights_t) -> bool {
    return *vm_rights != vm_rights_t::VMKernelOnly;
}

/// 当进行进行`map`操作时，会检查应用程序希望获得的读写权限与`frame`本身拥有的权限，
/// 依据两者的权限来进行选择，页表项应该具有的权限
///
/// Balance the rights program want and the rights pages have, decide which rights return to new alloced page.
#[no_mangle]
pub fn maskVMRights(vm_rights: vm_rights_t, rights: seL4_CapRights_t) -> vm_rights_t {
    if vm_rights == vm_rights_t::VMReadOnly && rights.get_allow_read() != 0 {
        return vm_rights_t::VMReadOnly;
    }
    if vm_rights == vm_rights_t::VMReadWrite && rights.get_allow_read() != 0 {
        return if rights.get_allow_write() == 0 {
            vm_rights_t::VMReadOnly
        } else {
            vm_rights_t::VMReadWrite
        };
    }
    vm_rights_t::VMKernelOnly
}
