#![feature(core_intrinsics)]
// #![no_std]
#![allow(internal_features)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
#![cfg_attr(not(test), no_std)]

// #[cfg(test)]
// #[macro_use]
// extern crate std;

mod cap;
mod cte;
mod mdb;
mod structures;

/// 需要外部实现的接口
pub mod deps;
/// 暴露给外部的接口
pub mod interface;

/// 兼容c风格的接口，后续会删除
pub mod compatibility;

pub mod arch;



#[cfg(test)]
mod tests {
    // use super::*;

    #[test]
    pub fn test1() {
        assert!(true);
    }
}