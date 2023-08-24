use core::arch::asm;

use super::{pte::pte_t, set_vm_root};
use common::{BIT, structures::{exception_t, lookup_fault_invalid_root_new, lookup_fault_t}, MASK, sel4_config::{asidHighBits, asidLowBits}};
use cspace::interface::cap_t;

#[no_mangle]
pub static mut riscvKSASIDTable: [*mut asid_pool_t; BIT!(asidHighBits)] =
    [0 as *mut asid_pool_t; BIT!(asidHighBits)];

#[derive(Copy, Clone)]
pub struct asid_pool_t {
    pub array: [*mut pte_t; BIT!(asidLowBits)],
}

pub type asid_t = usize;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct findVSpaceForASID_ret {
    pub status: exception_t,
    pub vspace_root: Option<*mut pte_t>,
    pub lookup_fault: Option<lookup_fault_t>,
}

#[no_mangle]
pub fn find_vspace_for_asid(asid: asid_t) -> findVSpaceForASID_ret {
    let mut ret: findVSpaceForASID_ret = findVSpaceForASID_ret {
        status: exception_t::EXCEPTION_FAULT,
        vspace_root: None,
        lookup_fault: None,
    };

    let poolPtr =  unsafe {
        riscvKSASIDTable[asid >> asidLowBits]
    };
    if poolPtr as usize == 0 {
        ret.lookup_fault = Some(lookup_fault_invalid_root_new());
        ret.vspace_root = None;
        ret.status = exception_t::EXCEPTION_LOOKUP_FAULT;
        return ret;
    }
    let vspace_root = unsafe {
        (*poolPtr).array[asid & MASK!(asidLowBits)]
    };
    if vspace_root as usize == 0 {
        ret.lookup_fault = Some(lookup_fault_invalid_root_new());
        ret.vspace_root = None;
        ret.status = exception_t::EXCEPTION_LOOKUP_FAULT;
        return ret;
    }
    ret.vspace_root = Some(vspace_root);
    ret.status = exception_t::EXCEPTION_NONE;
    // vspace_root0xffffffc17fec1000
    return ret;
}

#[no_mangle]
pub fn findVSpaceForASID(asid: asid_t) -> findVSpaceForASID_ret {
    find_vspace_for_asid(asid)
}

pub fn hwASIDFlush(asid: asid_t) {
    unsafe {
        asm!("sfence.vma x0, {0}",in(reg) asid);
    }
}

pub fn delete_asid_pool(asid_base: asid_t, pool: *mut asid_pool_t, default_vspace_cap: &cap_t) -> Result<(), lookup_fault_t> {
    unsafe {
        if riscvKSASIDTable[asid_base >> asidLowBits] == pool {
            riscvKSASIDTable[asid_base >> asidLowBits] = 0 as *mut asid_pool_t;
            // setVMRoot(ksCurThread);
            set_vm_root(default_vspace_cap)
        } else {
            Ok(())
        }
    }
}

pub fn delete_asid(asid: asid_t, vspace: *mut pte_t, default_vspace_cap: &cap_t) -> Result<(), lookup_fault_t> {
    unsafe {
        let poolPtr = riscvKSASIDTable[asid >> asidLowBits];
        if poolPtr as usize != 0 && (*poolPtr).array[asid & MASK!(asidLowBits)] == vspace {
            hwASIDFlush(asid);
            (*poolPtr).array[asid & MASK!(asidLowBits)] = 0 as *mut pte_t;
            set_vm_root(&default_vspace_cap)
        } else {
            Ok(())
        }
    }
}

