use sel4_common::{sel4_config::*, MASK};

use crate::pte_t;
pub const KPT_LEVELS: usize = 4;
pub const seL4_VSpaceIndexBits: usize = 9;
#[inline]
pub fn GET_PT_INDEX(addr: usize) -> usize {
    (addr >> PT_INDEX_OFFSET) & MASK!(PT_INDEX_BITS)
}
#[inline]
pub fn GET_PD_INDEX(addr: usize) -> usize {
    (addr >> PD_INDEX_OFFSET) & MASK!(PD_INDEX_BITS)
}
#[inline]
pub fn GET_UPUD_INDEX(addr: usize) -> usize {
    (addr >> PUD_INDEX_OFFSET) & MASK!(UPUD_INDEX_BITS)
}
#[inline]
pub fn GET_PUD_INDEX(addr: usize) -> usize {
    (addr >> PUD_INDEX_OFFSET) & MASK!(PUD_INDEX_BITS)
}
#[inline]
pub fn GET_PGD_INDEX(addr: usize) -> usize {
    (addr >> PGD_INDEX_OFFSET) & MASK!(PGD_INDEX_BITS)
}

#[inline]
pub fn GET_KPT_INDEX(addr: usize, n: usize) -> usize {
    ((addr) >> (((PT_INDEX_BITS) * (((KPT_LEVELS) - 1) - (n))) + seL4_PageBits))
        & MASK!(PT_INDEX_BITS)
}

#[inline]
pub fn GET_UPT_INDEX(addr: usize, n: usize) -> usize {
    ((addr) >> (((PT_INDEX_BITS) * (((KPT_LEVELS) - 1) - (n))) + seL4_PageBits))
        & MASK!(PT_INDEX_BITS)
}

#[inline]
pub fn kpptr_to_paddr(x: usize) -> usize {
    x - KERNEL_ELF_BASE_OFFSET
}

///计算以`PPTR_BASE`作为偏移的指针虚拟地址对应的物理地址
#[inline]
pub fn pptr_to_paddr(x: usize) -> usize {
    x - PPTR_BASE_OFFSET
}

///计算物理地址对应的虚拟地址，以`PPTR_BASE`作为偏移
#[inline]
pub fn paddr_to_pptr(x: usize) -> usize {
    x + PPTR_BASE_OFFSET
}
