#[cfg(target_arch = "aarch64")]
mod decode_aarch_mmu_invocation;
#[cfg(target_arch = "riscv64")]
mod decode_riscv_mmu_invocation;

#[cfg(target_arch = "aarch64")]
pub use decode_aarch_mmu_invocation::*;
#[cfg(target_arch = "riscv64")]
pub use decode_riscv_mmu_invocation::*;
