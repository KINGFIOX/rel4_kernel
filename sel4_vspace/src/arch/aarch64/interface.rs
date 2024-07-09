use core::{intrinsics::unlikely, ops::{Deref, DerefMut}};

use super::{
    machine::{setCurrentUserVSpaceRoot, ttbr_new},
    pte::PTEFlags,
};
use crate::{find_vspace_for_asid, pptr_to_paddr, pte_t};
use sel4_common::{
    fault::lookup_fault_t,
    sel4_config::{seL4_LargePageBits, PADDR_BASE, PADDR_TOP, PPTR_BASE, PPTR_TOP, PT_INDEX_BITS},
    structures::exception_t,
    utils::convert_to_mut_type_ref,
    BIT,
};
use sel4_cspace::interface::{cap_t, CapTag};

use super::utils::{kpptr_to_paddr, GET_KPT_INDEX};


pub const PageAlignedLen: usize = BIT!(PT_INDEX_BITS);
#[repr(align(4096))]
#[derive(Clone, Copy)]
pub struct PageAligned<T>([T; PageAlignedLen]);

impl<T: Copy> PageAligned<T> {
    pub const fn new(v: T) -> Self {
        Self([v; PageAlignedLen])
    }
}

impl<T> Deref for PageAligned<T> {
    type Target = [T; PageAlignedLen];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for PageAligned<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[no_mangle]
#[link_section = ".page_table"]
pub(crate) static mut armKSGlobalKernelPGD: PageAligned<pte_t> = PageAligned::new(pte_t(0));

#[no_mangle]
#[link_section = ".page_table"]
pub(crate) static mut armKSGlobalKernelPUD: PageAligned<pte_t> = PageAligned::new(pte_t(0));

// #[no_mangle]
// #[link_section = ".page_table"]
// pub(crate) static mut armKSGlobalKernelPDs: [[pte_t; BIT!(PT_INDEX_BITS)]; BIT!(PT_INDEX_BITS)] =
//     [[pte_t(0); BIT!(PT_INDEX_BITS)]; BIT!(PT_INDEX_BITS)];
#[no_mangle]
#[link_section = ".page_table"]
pub(crate) static mut armKSGlobalKernelPDs: PageAligned<PageAligned<pte_t>> = 
    PageAligned::new(PageAligned::new(pte_t(0)));

#[no_mangle]
#[link_section = ".page_table"]
pub(crate) static mut armKSGlobalUserVSpace: PageAligned<pte_t> = PageAligned::new(pte_t(0));

#[no_mangle]
#[link_section = ".page_table"]
pub(crate) static mut armKSGlobalKernelPT: PageAligned<pte_t> = PageAligned::new(pte_t(0));

#[no_mangle]
pub fn rust_map_kernel_window() {
    unsafe {
        armKSGlobalKernelPGD[GET_KPT_INDEX(PPTR_BASE, 0)] =
            pte_t::pte_next_table(kpptr_to_paddr(armKSGlobalKernelPUD.as_ptr() as usize), true);
    }

    let mut idx = GET_KPT_INDEX(PPTR_BASE, 1);
    while idx < GET_KPT_INDEX(PPTR_TOP, 1) {
        unsafe {
            armKSGlobalKernelPUD[idx] = pte_t::pte_next_table(
                kpptr_to_paddr(armKSGlobalKernelPDs[idx].as_ptr() as usize),
                true,
            );
        }
        idx += 1;
    }

    let mut vaddr = PPTR_BASE;
    let mut paddr = PADDR_BASE;
    while paddr < PADDR_TOP {
        unsafe {
            let flag = PTEFlags::UXN | PTEFlags::AF | PTEFlags::NORMAL;
            armKSGlobalKernelPDs[GET_KPT_INDEX(vaddr, 1)][GET_KPT_INDEX(vaddr, 2)] =
                pte_t::new(paddr, flag);
            vaddr += BIT!(seL4_LargePageBits);
            paddr += BIT!(seL4_LargePageBits)
        }
    }

    unsafe {
        armKSGlobalKernelPUD[GET_KPT_INDEX(PPTR_TOP, 1)] = pte_t::pte_next_table(
            kpptr_to_paddr(armKSGlobalKernelPDs[BIT!(PT_INDEX_BITS) - 1].as_ptr() as usize),
            true,
        );
        armKSGlobalKernelPDs[BIT!(PT_INDEX_BITS) - 1][BIT!(PT_INDEX_BITS) - 1] =
            pte_t::pte_next_table(kpptr_to_paddr(armKSGlobalKernelPT.as_ptr() as usize), true);
    }

    //FIXME:: map_kernel_window not implemented;
}

///根据给定的`vspace_root`设置相应的页表，会检查`vspace_root`是否合法，如果不合法默认设置为内核页表
///
/// Use page table in vspace_root to set the satp register.
pub fn set_vm_root(vspace_root: &cap_t) -> Result<(), lookup_fault_t> {
    if vspace_root.get_cap_type() != CapTag::CapPageTableCap {
        unsafe {
            setCurrentUserVSpaceRoot(ttbr_new(
                0,
                kpptr_to_paddr(armKSGlobalUserVSpace.as_ptr() as usize),
            ));
            return Ok(());
        }
    }
    let lvl1pt = convert_to_mut_type_ref::<pte_t>(vspace_root.get_pt_base_ptr());
    let asid = vspace_root.get_pt_mapped_asid();
    let find_ret = find_vspace_for_asid(asid);
    let mut ret = Ok(());
    if unlikely(
        find_ret.status != exception_t::EXCEPTION_NONE
            || find_ret.vspace_root.is_none()
            || find_ret.vspace_root.unwrap() != lvl1pt,
    ) {
        unsafe {
            if let Some(lookup_fault) = find_ret.lookup_fault {
                ret = Err(lookup_fault);
            }
            setCurrentUserVSpaceRoot(ttbr_new(
                0,
                kpptr_to_paddr(armKSGlobalUserVSpace.as_ptr() as usize),
            ));
        }
    }
    setCurrentUserVSpaceRoot(ttbr_new(asid, pptr_to_paddr(lvl1pt as *mut pte_t as usize)));
    ret
}

pub fn activate_kernel_window() {
    todo!()
}
