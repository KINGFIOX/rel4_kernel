#[cfg(target_arch = "riscv64")]
mod riscv_irq_invocation;
#[cfg(target_arch = "riscv64")]
pub use riscv_irq_invocation::*;
#[cfg(target_arch = "aarch64")]
mod aarch_irq_invocation;
#[cfg(target_arch = "aarch64")]
pub use aarch_irq_invocation::*;
