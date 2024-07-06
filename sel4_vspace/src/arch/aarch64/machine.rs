use core::arch::asm;

use aarch64_cpu::registers::{Writeable, TTBR0_EL1, TTBR1_EL1};
#[inline]
pub fn setCurrentKernelVSpaceRoot(val: usize) {
    TTBR1_EL1.set(val as _);
}

#[inline]
pub fn setCurrentUserVSpaceRoot(val: usize) {
    TTBR0_EL1.set(val as _);
}

#[inline]
pub fn ttbr_new(asid: usize, addr: usize) -> usize {
    (asid & 0xffff) << 48 | (addr & 0xffffffffffff)
}

#[inline]
pub fn dsb() {
    unsafe {
        asm!("dsb", options(nostack, preserves_flags));
    }
}

#[inline]
pub fn isb() {
    unsafe {
        asm!("isb", options(nostack, preserves_flags));
    }
}

#[inline]
pub fn invalidate_local_tlb_asid(asid: usize) {
    assert!(asid < (1 << 16)); // BIT(16) 相当于 1 << 16

    dsb();
    unsafe {
        asm!("tlbi aside1, {}", in(reg) (asid << 48));
    }
    dsb();
    isb();
}

#[inline(always)]
pub fn clean_by_va_pou(vaddr: usize, _paddr: usize) {
    unsafe {
        asm!("dc cvau, {}", in(reg) vaddr);
    }
    dmb();
}

#[inline(always)]
pub fn dmb() {
    unsafe {
        asm!("dmb", options(nostack, preserves_flags));
    }
}

// pub fn cleanInvalidateL1Caches()
// {
//     dsb();
//     cleanInvalidate_D_PoC();
//     dsb();
//     invalidate_I_PoU();
//     dsb();
// }

// void cleanInvalidate_D_PoC(void)
// {
//     int clid = readCLID();
//     int loc = LOC(clid);

//     for (int l = 0; l < loc; l++) {
//         if (CTYPE(clid, l) > ARMCacheI) {
//             cleanInvalidate_D_by_level(l);
//         }
//     }
// }
