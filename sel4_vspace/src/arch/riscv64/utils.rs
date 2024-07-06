use sel4_common::sel4_config::{
    seL4_PageBits, CONFIG_PT_LEVELS, KERNEL_ELF_BASE_OFFSET, PPTR_BASE_OFFSET, PT_INDEX_BITS,
};
use sel4_common::{BIT, MASK};

///获得虚拟地址`addr`对应的`n`级VPN，
/// 具体对应关系为:
/// ```
/// VPN[2] <=> n = 0
/// VPN[1] <=> n = 1
/// VPN[0] <=> n = 2
/// ```
#[inline]
pub fn RISCV_GET_PT_INDEX(addr: usize, n: usize) -> usize {
    ((addr) >> (((PT_INDEX_BITS) * (((CONFIG_PT_LEVELS) - 1) - (n))) + seL4_PageBits))
        & MASK!(PT_INDEX_BITS)
}

/// 获得第n级页表对应的虚拟地址空间的大小位数
/// 根页表对应2^30=1GB,30位
/// 一级页表对应2^21=2MB，21位
/// 二级页表对应2^12=4KB，12位
///
/// Get n levels page bit size
#[inline]
pub fn RISCV_GET_LVL_PGSIZE_BITS(n: usize) -> usize {
    ((PT_INDEX_BITS) * (((CONFIG_PT_LEVELS) - 1) - (n))) + seL4_PageBits
}

/// 获得第n级页表对应的虚拟地址空间的大小
/// 根页表对应2^30=1GB,30位
/// 一级页表对应2^21=2MB，21位
/// 二级页表对应2^12=4KB，12位
///
/// Get n levels page size
#[inline]
pub fn RISCV_GET_LVL_PGSIZE(n: usize) -> usize {
    BIT!(RISCV_GET_LVL_PGSIZE_BITS(n))
}

///在`reL4`内核页表中，内核代码，在内核地址空间中被映射了两次，
/// 一次映射到`KERNEL_ELF_BASE`开始的虚拟地址上，
/// 由于整个物理地址空间会在内核虚拟地址空间中做一次完整的映射，映射到`PPTR_BASE`开始的虚拟地址上，
/// 所以会再一次将内核映射地内核地址空间中。
/// `reL4`地址空间的布局可以参考`map_kernel_window`函数的`doc`
/// 内核本身的指针类型，采用以`KERNEL_ELF_BASE_OFFSET`
/// 该函数作用就是计算以`KERNEL_ELF_BASE`开始的内核的虚拟地址的真实物理地址。
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
