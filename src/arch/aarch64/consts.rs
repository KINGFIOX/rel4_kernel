use sel4_common::MASK;

pub const PPTR_TOP: usize = 0xFFFFFFFF80000000;
pub const physBase: usize = 0x80000000;
pub const KERNEL_ELF_PADDR_BASE: usize = physBase + 0x4000000;
pub const KERNEL_ELF_BASE: usize = PPTR_TOP + (KERNEL_ELF_PADDR_BASE & MASK!(30));
