#![no_std]
#![allow(non_snake_case)]
#![allow(internal_features)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
#![feature(core_intrinsics)]
mod arch;
mod asid;
pub mod interface;
mod pagetable;
mod pte;
mod structures;
mod utils;

#[cfg(target_arch = "aarch64")]
pub use arch::aarch64::*;
#[cfg(target_arch = "riscv64")]
pub use arch::riscv64::*;
pub use asid::*;
pub use interface::unmapPage;
pub use pagetable::PageTable;
pub use pte::pte_t;
pub use structures::*;
pub use utils::checkVPAlignment;
// pub use riscv::*;
