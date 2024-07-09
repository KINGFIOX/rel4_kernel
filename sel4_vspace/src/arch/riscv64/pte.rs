use bitflags::bitflags;
use core::intrinsics::unlikely;

use sel4_common::{
    arch::{vm_rights_t, RISCVGetReadFromVMRights, RISCVGetWriteFromVMRights},
    sel4_config::{seL4_PageBits, seL4_PageTableBits, CONFIG_PT_LEVELS, PT_INDEX_BITS},
    structures::exception_t,
    utils::{convert_to_mut_type_ref, convert_to_type_ref},
    BIT, MASK,
};

use crate::{
    arch::riscv64::{sfence, utils::RISCV_GET_PT_INDEX},
    asid_t, find_vspace_for_asid, lookupPTSlot_ret_t, pte_t, vptr_t,
};

use super::paddr_to_pptr;

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct PTEFlags: usize {
        const V = BIT!(0);
        const R = BIT!(1);
        const W = BIT!(2);
        const X = BIT!(3);
        const U = BIT!(4);
        const G = BIT!(5);
        const A = BIT!(6);
        const D = BIT!(7);

        const VRWX  = Self::V.bits() | Self::R.bits() | Self::W.bits() | Self::X.bits();
        const ADUVRX = Self::A.bits() | Self::D.bits() | Self::U.bits() | Self::V.bits() | Self::R.bits() | Self::X.bits();
        const ADVRWX = Self::A.bits() | Self::D.bits() | Self::VRWX.bits();
        const ADUVRWX = Self::A.bits() | Self::D.bits()| Self::U.bits() | Self::VRWX.bits();
        const ADGVRWX = Self::G.bits() | Self::ADVRWX.bits();
    }
}

impl From<usize> for pte_t {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

impl pte_t {
    #[inline]
    pub fn new(ppn: usize, flags: PTEFlags) -> Self {
        Self(flags.bits() | (ppn << 10))
    }

    /// 创建一个用户使用的页表项（`Global=0`、`User=1`）
    #[inline]
    pub fn make_user_pte(paddr: usize, executable: bool, vm_rights: vm_rights_t) -> Self {
        let write = RISCVGetWriteFromVMRights(&vm_rights);
        let read = RISCVGetReadFromVMRights(&vm_rights);
        if !executable && !read && !write {
            return Self::pte_invalid();
        }
        let mut flag = PTEFlags::V | PTEFlags::D | PTEFlags::A | PTEFlags::U;
        if executable {
            flag |= PTEFlags::X;
        }
        if write {
            flag |= PTEFlags::W;
        }
        if read {
            flag |= PTEFlags::R;
        }
        Self::new(paddr >> seL4_PageBits, flag)
    }

    ///创建内核态页表项（`Global=1`、`User=0`）
    #[inline]
    pub fn pte_next_table(phys_addr: usize, is_leaf: bool) -> Self {
        let ppn = (phys_addr >> 12) as usize;

        let mut flag = PTEFlags::V | PTEFlags::G;
        if is_leaf {
            flag |= PTEFlags::X | PTEFlags::W | PTEFlags::R | PTEFlags::A | PTEFlags::D;
        }
        Self::new(ppn, flag)
    }

    #[inline]
    pub fn update(&mut self, pte: Self) {
        *self = pte;
        sfence();
    }

    pub fn unmap_page_table(&mut self, asid: asid_t, vptr: vptr_t) {
        let target_pt = self as *mut pte_t;
        let find_ret = find_vspace_for_asid(asid);
        if find_ret.status != exception_t::EXCEPTION_NONE {
            return;
        }
        assert_ne!(find_ret.vspace_root.unwrap(), target_pt);
        let mut pt = find_ret.vspace_root.unwrap();
        let mut ptSlot = unsafe { &mut *(pt.add(RISCV_GET_PT_INDEX(vptr, 0))) };
        let mut i = 0;
        while i < CONFIG_PT_LEVELS - 1 && pt != target_pt {
            ptSlot = unsafe { &mut *(pt.add(RISCV_GET_PT_INDEX(vptr, i))) };
            if unlikely(ptSlot.is_pte_table()) {
                return;
            }
            pt = ptSlot.get_pte_from_ppn_mut() as *mut pte_t;
            i += 1;
        }

        if pt != target_pt {
            return;
        }
        *ptSlot = pte_t::new(0, PTEFlags::empty());
        sfence();
    }

    #[inline]
    pub const fn pte_invalid() -> Self {
        Self(0)
    }

    ///判断是页目录节点还是叶子节点，当`valid`置1，`read``write``exec`置0时，代表为叶子节点
    #[inline]
    pub fn is_pte_table(&self) -> bool {
        self.get_valid() != 0
            && !(self.get_read() != 0 || self.get_write() != 0 || self.get_execute() != 0)
    }

    #[inline]
    pub fn get_pte_from_ppn_mut(&self) -> &'static mut Self {
        convert_to_mut_type_ref::<pte_t>(paddr_to_pptr(self.get_ppn() << seL4_PageTableBits))
    }

    #[inline]
    pub fn get_pte_from_ppn(&self) -> &'static Self {
        convert_to_type_ref::<pte_t>(paddr_to_pptr(self.get_ppn() << seL4_PageTableBits))
    }

    #[inline]
    pub fn get_valid(&self) -> usize {
        (self.0 & 0x1) >> 0
    }

    #[inline]
    pub fn get_ppn(&self) -> usize {
        (self.0 & 0x3f_ffff_ffff_fc00usize) >> 10
    }

    #[inline]
    pub fn get_execute(&self) -> usize {
        (self.0 & 0x8usize) >> 3
    }

    #[inline]
    pub fn get_write(&self) -> usize {
        (self.0 & 0x4usize) >> 2
    }

    #[inline]
    pub fn get_read(&self) -> usize {
        (self.0 & 0x2usize) >> 1
    }

    ///用于记录某个虚拟地址`vptr`对应的pte表项在内存中的位置
    pub fn lookup_pt_slot(&self, vptr: vptr_t) -> lookupPTSlot_ret_t {
        let mut level = CONFIG_PT_LEVELS - 1;
        let mut pt = self as *const pte_t as usize as *mut pte_t;
        let mut ret = lookupPTSlot_ret_t {
            ptBitsLeft: PT_INDEX_BITS * level + seL4_PageBits,
            ptSlot: unsafe {
                pt.add((vptr >> (PT_INDEX_BITS * level + seL4_PageBits)) & MASK!(PT_INDEX_BITS))
            },
        };

        while unsafe { (*ret.ptSlot).is_pte_table() } && level > 0 {
            level -= 1;
            ret.ptBitsLeft -= PT_INDEX_BITS;
            pt = unsafe { (*ret.ptSlot).get_pte_from_ppn_mut() as *mut pte_t };
            ret.ptSlot = unsafe { pt.add((vptr >> ret.ptBitsLeft) & MASK!(PT_INDEX_BITS)) };
        }
        ret
    }
}
