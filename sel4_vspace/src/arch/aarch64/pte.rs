use core::intrinsics::unlikely;

use crate::{
    arch::aarch64::{
        machine::{clean_by_va_pou, invalidate_local_tlb_asid},
        structures::{
            lookupPDSlot_ret_t, lookupPGDSlot_ret_t, lookupPTSlot_ret_t, lookupPUDSlot_ret_t,
        },
        utils::{GET_PD_INDEX, GET_PGD_INDEX, GET_PT_INDEX, GET_UPUD_INDEX},
        vm_rights::vm_rights_t,
    },
    asid_t, find_vspace_for_asid, pptr_to_paddr, pte_t, vm_attributes_t, vptr_t,
};
use sel4_common::{
    fault::lookup_fault_t,
    sel4_config::{seL4_PageBits, seL4_PageTableBits, PT_INDEX_BITS},
    structures::exception_t,
    utils::{convert_ref_type_to_usize, convert_to_mut_type_ref},
    BIT,
};

use super::utils::{paddr_to_pptr, GET_UPT_INDEX};

enum vm_page_size {
    ARMSmallPage,
    ARMLargePage,
    ARMHugePage,
}

enum pte_tag {
    pte_pte_table = 3,
    pte_pte_page = 1,
    pte_pte_4k_page = 7,
    pte_pte_invalid = 0,
}

pub const UPT_LEVELS: usize = 4;
bitflags::bitflags! {
    /// Possible flags for a page table entry.
    pub struct PTEFlags: usize {
        // Attribute fields in stage 1 VMSAv8-64 Block and Page descriptors:
        /// Whether the descriptor is valid.
        const VALID =       BIT!(0);
        /// The descriptor gives the address of the next level of translation table or 4KB page.
        /// (not a 2M, 1G block)
        const NON_BLOCK =   BIT!(1);
        /// Memory attributes index field.
        const ATTR_INDX =   0b111 << 2;
        const NORMAL_NONCACHE = 0b010 << 2;
        const NORMAL =      0b100 << 2;
        /// Non-secure bit. For memory accesses from Secure state, specifies whether the output
        /// address is in Secure or Non-secure memory.
        const NS =          BIT!(5);
        /// Access permission: accessable at EL0.
        const AP_EL0 =      BIT!(6);
        /// Access permission: read-only.
        const AP_RO =       BIT!(7);
        /// Shareability: Inner Shareable (otherwise Outer Shareable).
        const INNER =       BIT!(8);
        /// Shareability: Inner or Outer Shareable (otherwise Non-shareable).
        const SHAREABLE =   BIT!(9);
        /// The Access flag.
        const AF =          BIT!(10);
        /// The not global bit.
        const NG =          BIT!(11);
        /// Indicates that 16 adjacent translation table entries point to contiguous memory regions.
        const CONTIGUOUS =  BIT!(52);
        /// The Privileged execute-never field.
        const PXN =         BIT!(53);
        /// The Execute-never or Unprivileged execute-never field.
        const UXN =         BIT!(54);

        // Next-level attributes in stage 1 VMSAv8-64 Table descriptors:

        /// PXN limit for subsequent levels of lookup.
        const PXN_TABLE =           BIT!(59);
        /// XN limit for subsequent levels of lookup.
        const XN_TABLE =            BIT!(60);
        /// Access permissions limit for subsequent levels of lookup: access at EL0 not permitted.
        const AP_NO_EL0_TABLE =     BIT!(61);
        /// Access permissions limit for subsequent levels of lookup: write access not permitted.
        const AP_NO_WRITE_TABLE =   BIT!(62);
        /// For memory accesses from Secure state, specifies the Security state for subsequent
        /// levels of lookup.
        const NS_TABLE =            BIT!(63);

    }
}

impl pte_t {
    pub fn new(addr: usize, flags: PTEFlags) -> Self {
        Self((addr & 0xfffffffff000) | flags.bits())
    }
    pub fn pte_next_table(addr: usize, _: bool) -> Self {
        Self::new(addr, PTEFlags::VALID | PTEFlags::NON_BLOCK)
    }
    fn new_4k_page(addr: usize, flags: PTEFlags) -> Self {
        Self((addr & 0xfffffffff000) | flags.bits() | 0x400000000000003)
    }

    pub fn get_pte_from_ppn_mut(&self) -> &mut pte_t {
        convert_to_mut_type_ref::<pte_t>(paddr_to_pptr(self.get_ppn() << seL4_PageTableBits))
    }

    pub fn get_ppn(&self) -> usize {
        (self.0 & 0xfffffffff000) >> 10
    }

    pub fn get_type(&self) -> usize {
        (self.0 & 0x3) | ((0x1 << 58) >> 56)
    }

    pub fn is_pte_table(&self) -> bool {
        self.get_type() != pte_tag::pte_pte_table as usize
    }
    pub fn get_valid(&self) -> usize {
        (self.get_type() != pte_tag::pte_pte_invalid as usize) as usize
    }

    pub fn pte_table_get_present(&self) -> bool {
        self.get_type() != pte_tag::pte_pte_table as usize
    }

    pub fn new_invalid() -> Self {
        Self::new(0, PTEFlags::empty())
    }

    #[inline]
    pub fn update(&mut self, pte: Self) {
        *self = pte;
        clean_by_va_pou(
            convert_ref_type_to_usize(self),
            convert_ref_type_to_usize(self),
        );
    }

    pub fn ap_from_vm_rights_t(rights: vm_rights_t) -> PTEFlags {
        match rights {
            vm_rights_t::VMKernelOnly => PTEFlags::empty(),
            vm_rights_t::VMReadWrite => PTEFlags::AP_EL0,
            vm_rights_t::VMReadOnly => PTEFlags::AP_EL0 | PTEFlags::AP_RO,
        }
    }

    pub fn make_user_pte(
        paddr: usize,
        rights: vm_rights_t,
        attr: vm_attributes_t,
        page_size: usize,
    ) -> Self {
        let nonexecutable = attr.get_armExecuteNever();
        let cacheable = attr.get_armPageCacheable();
        let mut flags = PTEFlags::NG;
        if cacheable {
            flags |= PTEFlags::NORMAL;
        }
        if nonexecutable {
            flags |= PTEFlags::UXN;
        }
        flags |= Self::ap_from_vm_rights_t(rights);
        if vm_page_size::ARMSmallPage as usize == page_size {
            pte_t::new_4k_page(paddr, flags)
        } else {
            pte_t::new(paddr, flags)
        }
    }

    pub fn unmap_page_table(&mut self, asid: asid_t, vptr: vptr_t) {
        let target_pt = self as *mut pte_t;
        let find_ret = find_vspace_for_asid(asid);
        if unlikely(find_ret.status != exception_t::EXCEPTION_NONE) {
            return;
        }
        let mut pt = find_ret.vspace_root.unwrap();
        let mut ptSlot = unsafe { &mut *(pt.add(GET_UPT_INDEX(vptr, 0))) };
        assert_ne!(find_ret.vspace_root.unwrap(), target_pt);
        for i in 0..UPT_LEVELS - 1 {
            if pt == target_pt {
                break;
            }
            ptSlot = unsafe { &mut *(pt.add(GET_UPT_INDEX(vptr, i))) };
            if unlikely(ptSlot.pte_table_get_present()) {
                return;
            }
        }

        if pt != target_pt {
            return;
        }
        *ptSlot = pte_t::new_invalid();
        invalidate_local_tlb_asid(asid);
        clean_by_va_pou(
            convert_ref_type_to_usize(ptSlot),
            pptr_to_paddr(convert_ref_type_to_usize(ptSlot)),
        )
    }

    ///用于记录某个虚拟地址`vptr`对应的pte表项在内存中的位置
    pub fn lookup_pt_slot(&self, vptr: vptr_t) -> lookupPTSlot_ret_t {
        let pdSlot = self.lookup_pd_slot(vptr);
        if pdSlot.status != exception_t::EXCEPTION_NONE {
            let ret = unsafe {
                lookupPTSlot_ret_t {
                    status: pdSlot.status,
                    ptSlot: 0 as *mut pte_t,
                }
            };
            return ret;
        }
        unsafe {
            if ((*pdSlot.pdSlot).0 & 0x3) != 3 {
                // todo!() I cannot use current_lookup_fault here
                // current_lookup_fault =lookup_fault_t::new_missing_cap(seL4_PageBits+PT_INDEX_BITS);
                let ret = unsafe {
                    lookupPTSlot_ret_t {
                        status: exception_t::EXCEPTION_LOOKUP_FAULT,
                        ptSlot: 0 as *mut pte_t,
                    }
                };
                return ret;
            }
        }
        let ptIndex = GET_PT_INDEX(vptr);
        let pt = unsafe { paddr_to_pptr((*pdSlot.pdSlot).0 & 0xfffffffff000) as *mut pte_t };

        let ret = lookupPTSlot_ret_t {
            status: exception_t::EXCEPTION_NONE,
            ptSlot: unsafe { pt.add(ptIndex) },
        };
        ret
    }
    // acturally the lookup pd slot can only be seen under aarch64 and x86 in sel4
    // and in sel4, it should be the impl function of vspace_root_t
    // but as it define the pde_t as vspace_root_t and define pte_t as vspace_root_t
    // so I think it is reasonable here to let those functions as a member funcion of pte_t
    // commented by ZhiyuanSue
    pub fn lookup_pd_slot(&self, vptr: vptr_t) -> lookupPDSlot_ret_t {
        let pudSlot = self.lookup_pud_slot(vptr);
        if pudSlot.status != exception_t::EXCEPTION_NONE {
            let ret = unsafe {
                lookupPDSlot_ret_t {
                    status: pudSlot.status,
                    pdSlot: unsafe { 0 as *mut pte_t },
                }
            };
            return ret;
        }
        unsafe {
            if ((*pudSlot.pudSlot).0 & 0x3) != 3 {
                // todo!() I cannot use current_lookup_fault here
                // current_lookup_fault =lookup_fault_t::new_missing_cap(seL4_PageBits+PT_INDEX_BITS);
                let ret = lookupPDSlot_ret_t {
                    status: exception_t::EXCEPTION_LOOKUP_FAULT,
                    pdSlot: 0 as *mut pte_t,
                };
                return ret;
            }
        }
        let pdIndex = GET_PD_INDEX(vptr);
        let pd = unsafe { paddr_to_pptr((*pudSlot.pudSlot).0 & 0xfffffffff000) as *mut pte_t };

        let ret = lookupPDSlot_ret_t {
            status: exception_t::EXCEPTION_NONE,
            pdSlot: unsafe { pd.add(pdIndex) },
        };
        ret
    }
    pub fn lookup_pud_slot(&self, vptr: vptr_t) -> lookupPUDSlot_ret_t {
        let pgdSlot = self.lookup_pgd_slot(vptr);
        unsafe {
            if ((*pgdSlot.pgdSlot).0 & 0x3) != 3 {
                // todo!() I cannot use current_lookup_fault here
                // current_lookup_fault =lookup_fault_t::new_missing_cap(seL4_PageBits+PT_INDEX_BITS);
                let ret = lookupPUDSlot_ret_t {
                    status: exception_t::EXCEPTION_LOOKUP_FAULT,
                    pudSlot: 0 as *mut pte_t,
                };
                return ret;
            }
        }
        let pudIndex = GET_UPUD_INDEX(vptr);
        let pud = unsafe { paddr_to_pptr((*pgdSlot.pgdSlot).0 & 0xfffffffff000) as *mut pte_t };

        let ret = lookupPUDSlot_ret_t {
            status: exception_t::EXCEPTION_NONE,
            pudSlot: unsafe { pud.add(pudIndex) },
        };
        ret
    }
    pub fn lookup_pgd_slot(&self, vptr: vptr_t) -> lookupPGDSlot_ret_t {
        let pgdIndex = GET_PGD_INDEX(vptr);
        let ret = lookupPGDSlot_ret_t {
            status: exception_t::EXCEPTION_NONE,
            pgdSlot: unsafe { (self.0 as *mut pte_t).add(pgdIndex) },
        };
        ret
    }
}
