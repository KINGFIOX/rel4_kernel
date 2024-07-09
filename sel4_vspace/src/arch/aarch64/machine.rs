use core::arch::asm;

use aarch64_cpu::registers::{Writeable, TTBR0_EL1, TTBR1_EL1};
use sel4_common::MASK;
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

/**
 * sy（System）: 确保所有CPU都看到之前的存储操作的效果，这是最常用的级别，提供全系统范围的数据同步。
 * st（Store）: 确保之前的所有存储操作对其他处理器可见，主要用于控制存储操作的完成。
 * ld（Load）: 确保之前的所有加载操作完成，主要用于加载操作。
 * ish（Inner Shareable）: 仅确保同一内存共享域内的处理器看到之前的存储操作的效果。
 * ishst（Inner Shareable for Stores）: 类似于ish，但仅适用于存储操作。
 * nsh（Non-shareable）: 仅在非共享内存区域内确保之前的操作完成。
 * nshst（Non-shareable for Stores）: 类似于nsh，但仅适用于存储操作。
 * osh（Outer Shareable）: 确保操作对外部共享内存域内的所有处理器可见。
 * oshst（Outer Shareable for Stores）: 类似于osh，但仅适用于存储操作。
*/
#[inline]
pub fn dsb() {
    unsafe {
        asm!("dsb sy", options(nostack, preserves_flags));
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

// cache

fn LOUU(x: usize) -> usize {
    (x >> 27) & MASK!(3)
}

pub fn cleanCacheRange_RAM(start: usize, end: usize, pstart: usize) {
    cleanCacheRange_PoC(start, end, pstart);

    dsb();

    plat_cleanL2Range(pstart, pstart + (end - start));
}

pub fn cleanCacheRange_PoC(start:usize,end:usize,pstart: usize){
    
}

pub fn plat_cleanL2Range(pstart: usize, pend: usize) {
    
}

fn LOC(x: usize) -> usize {
    (x >> 24) & MASK!(3)
}

fn LOUIS(x: usize) -> usize {
    (x >> 21) & MASK!(3)
}

fn CTYPE(x: usize, n: usize) -> usize {
    (x >> (n * 3)) & MASK!(3)
}

fn LINEBITS(s: usize) -> usize {
    (s & MASK!(3)) + 4
}

fn ASSOC(s: usize) -> usize {
    ((s >> 3) & MASK!(10)) + 1
}

fn NSETS(s: usize) -> usize {
    ((s >> 13) & MASK!(15)) + 1
}

pub enum arm_cache_type {
    ARMCacheI = 1,
    ARMCacheD = 2,
    ARMCacheID = 3,
}

pub fn clean_invalidate_l1_caches() {
    dsb();
    clean_invalidate_D_PoC();
    dsb();
    invalidate_I_PoU();
    dsb();
}

#[inline]
pub fn invalidate_I_PoU() {
    unsafe {
        asm!("ic iallu");
    }
    isb();
}

pub fn clean_invalidate_D_PoC() {
    let clid = read_clid();
    let loc = LOC(clid);

    for l in 0..loc {
        if CTYPE(clid, l) > arm_cache_type::ARMCacheI as usize {
            clean_invalidate_D_by_level(l);
        }
    }
}

#[inline]
fn clean_invalidate_D_by_level(l: usize) {
    let lsize = read_cache_size(l, 0);
    let lbits = LINEBITS(lsize);
    let assoc = ASSOC(lsize);
    let assoc_bits = 64 - (assoc - 1).leading_zeros() as usize;
    let nsets = NSETS(lsize);

    for w in 0..assoc {
        for s in 0..nsets {
            clean_invalidate_by_wsl((w << (32 - assoc_bits)) | (s << lbits) | (l << 1));
        }
    }
}

#[inline]
fn clean_invalidate_by_wsl(wsl: usize) {
    unsafe {
        asm!("dc cisw, {}", in(reg) wsl);
    }
}

#[inline]
fn read_cache_size(level: usize, instruction: usize) -> usize {
    let size: usize;
    let csselr_old: usize;
    unsafe {
        // save CSSELR
        asm!("mrs {}, csselr_el1", out(reg) csselr_old);
        // select cache level
        asm!("msr csselr_el1, {}", in(reg) ((level << 1) | instruction));
        // read 'size'
        asm!("mrs {}, ccsidr_el1", out(reg) size);
        // restore CSSELR
        asm!("msr csselr_el1, {}", in(reg) csselr_old);
    }
    size
}

#[inline]
fn read_clid() -> usize {
    let clid: usize;
    unsafe {
        asm!("mrs {}, clidr_el1", out(reg) clid);
    }
    clid
}

#[inline]
pub fn invalidate_local_tlb() {
    dsb();
    unsafe {
        asm!("tlbi vmalle1");
    }
    dsb();
    isb();
}
