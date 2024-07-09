#![feature(core_intrinsics)]
#![no_std]
#![no_main]
#![feature(asm_const)]
#![allow(internal_features)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::tests::test_runner)]
#![reexport_test_harness_main = "test_main"]

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
    use arch::cap_t;
    use cap::same_object_as;
    use core::arch::global_asm;
    use cte::cte_t;
    use mdb::mdb_node_t;
    use riscv::register::{stvec, utvec::TrapMode};
    use sel4_common::{arch::shutdown, println};
    global_asm!(include_str!("entry.asm"));

    use super::*;
    pub fn test_runner(tests: &[&dyn Fn()]) {
        println!("Running {} tests", tests.len());
        for test in tests {
            test();
        }
    }

    #[no_mangle]
    #[test_case]
    pub fn same_object_as_test() {
        println!("Entering same_object_as_test case");
        let cap1 = cap_t::new_cnode_cap(1, 1, 1, 1);
        let cap3 = cap_t::new_cnode_cap(2, 1, 1, 1);
        let mdb = mdb_node_t::new(0, 0, 0, 0);
        let mut cte1 = cte_t {
            cap: cap1,
            cteMDBNode: mdb,
        };
        let cap2 = cte1.derive_cap(&cap3).cap;
        assert_eq!(same_object_as(&cte1.cap, &cap2), false);
        assert_eq!(same_object_as(&cap2, &cap3), true);
        println!("Test same_object_as_test passed");
    }
    #[test_case]
    pub fn shutdown_test() {
        println!("All Test Cases passed, shutdown");
        shutdown();
    }
    #[panic_handler]
    fn panic(info: &core::panic::PanicInfo) -> ! {
        println!("{}", info);
        shutdown()
    }
    const STACK_SIZE: usize = 4096;
    #[link_section = ".bss.stack"]
    static mut BOOT_STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];
    #[no_mangle]
    pub fn call_test_main() {
        extern "C" {
            fn trap_entry();
        }
        unsafe {
            stvec::write(trap_entry as usize, TrapMode::Direct);
        }
        crate::test_main();
    }
    #[no_mangle]
    pub fn c_handle_syscall() {
        unsafe {
            core::arch::asm!("sret");
        }
    }
    // #[no_mangle]
    // pub extern "C" fn _start() -> ! {
    //     unsafe {
    //         core::arch::asm!(
    //             // 1. 设置栈信息
    //             // sp = bootstack + (hartid + 1) * 0x10000
    //             "2:
    //             la      sp, {boot_stack}
    //             li      t0, {stack_size}
    //             add     sp, sp, t0              // set boot stack
    //         ",
    //             // 3. 跳到 rust_main 函数，绝对路径
    //             "
    //             la      a2, {entry}
    //             or      a2, a2, s0
    //             jalr    a2                      // call rust_main
    //         ",
    //             stack_size = const crate::tests::STACK_SIZE,
    //             boot_stack = sym crate::tests::BOOT_STACK,
    //             entry = sym crate::tests::test_main,
    //             options(noreturn),
    //         )
    //     }
    // }

    // #[no_mangle]
    // pub fn test_main() {
    //     println!("hello world!");
    //     crate::tests::test_runner(&[&(crate::tests::same_object_as_test)]);
    // }
}
