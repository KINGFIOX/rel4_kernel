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
    use arch::{cap_t, CapTag};
    use cap::same_object_as;
    use core::arch::global_asm;
    use cte::{cte_insert, cte_move, cte_swap, cte_t, insert_new_cap, resolve_address_bits};
    use mdb::mdb_node_t;
    use riscv::register::{stvec, utvec::TrapMode};
    use sel4_common::{arch::shutdown, println, utils::convert_to_mut_type_ref};
    global_asm!(include_str!("entry.asm"));

    use super::*;
    pub fn test_runner(tests: &[&dyn Fn()]) {
        println!("Running {} tests", tests.len());
        for test in tests {
            test();
        }
    }

    #[test_case]
    pub fn same_object_as_test() {
        println!("-----------------------------------");
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
        println!("-----------------------------------");
    }

    #[test_case]
    pub fn cte_insert_test() {
        println!("-----------------------------------");
        println!("Entering cte_insert_test case");
        let cap1 = cap_t::new_asid_control_cap();
        let cap2 = cap_t::new_domain_cap();
        let mut cte1 = cte_t {
            cap: cap_t::new_null_cap(),
            cteMDBNode: mdb_node_t::new(0, 0, 0, 0),
        };
        let mut cte2 = cte_t {
            cap: cap_t::new_null_cap(),
            cteMDBNode: mdb_node_t::new(0, 0, 0, 0),
        };
        let mut cte3 = cte_t {
            cap: cap_t::new_null_cap(),
            cteMDBNode: mdb_node_t::new(0, 0, 0, 0),
        };
        cte_insert(&cap1, &mut cte1, &mut cte2);
        cte_insert(&cap2, &mut cte2, &mut cte3);
        assert_eq!(cte2.cap.get_cap_type(), CapTag::CapASIDControlCap);
        assert_eq!(cte3.cap.get_cap_type(), CapTag::CapDomainCap);
        assert_eq!(cte1.cteMDBNode.get_next(), &mut cte2 as *mut cte_t as usize);
        assert_eq!(cte2.cteMDBNode.get_next(), &mut cte3 as *mut cte_t as usize);
        assert_eq!(cte2.cteMDBNode.get_prev(), &mut cte1 as *mut cte_t as usize);
        assert_eq!(cte3.cteMDBNode.get_prev(), &mut cte2 as *mut cte_t as usize);
        println!("Test cte_insert_test passed");
    }

    #[test_case]
    pub fn cte_move_test() {
        println!("-----------------------------------");
        println!("Entering cte_move_test case");
        let cap1 = cap_t::new_asid_control_cap();
        let cap2 = cap_t::new_domain_cap();
        let cap3 = cap_t::new_irq_control_cap();
        let mut cte1 = cte_t {
            cap: cap_t::new_null_cap(),
            cteMDBNode: mdb_node_t::new(0, 0, 0, 0),
        };
        let mut cte2 = cte_t {
            cap: cap_t::new_null_cap(),
            cteMDBNode: mdb_node_t::new(0, 0, 0, 0),
        };
        let mut cte3 = cte_t {
            cap: cap_t::new_null_cap(),
            cteMDBNode: mdb_node_t::new(0, 0, 0, 0),
        };
        let mut cte4 = cte_t {
            cap: cap_t::new_null_cap(),
            cteMDBNode: mdb_node_t::new(0, 0, 0, 0),
        };
        cte_insert(&cap1, &mut cte1, &mut cte2);
        cte_insert(&cap2, &mut cte2, &mut cte3);
        assert_eq!(cte1.cteMDBNode.get_next(), &mut cte2 as *mut cte_t as usize);
        assert_eq!(cte2.cteMDBNode.get_next(), &mut cte3 as *mut cte_t as usize);
        assert_eq!(cte2.cteMDBNode.get_prev(), &mut cte1 as *mut cte_t as usize);
        assert_eq!(cte3.cteMDBNode.get_prev(), &mut cte2 as *mut cte_t as usize);
        cte_move(&cap3, &mut cte2, &mut cte4);
        assert_eq!(cte4.cap.get_cap_type(), CapTag::CapIrqControlCap);
        assert_eq!(cte4.cteMDBNode.get_next(), &mut cte3 as *mut cte_t as usize);
        assert_eq!(cte4.cteMDBNode.get_prev(), &mut cte1 as *mut cte_t as usize);
        assert_eq!(cte1.cteMDBNode.get_next(), &mut cte4 as *mut cte_t as usize);
        assert_eq!(cte3.cteMDBNode.get_prev(), &mut cte4 as *mut cte_t as usize);
        assert_ne!(cte1.cteMDBNode.get_next(), &mut cte2 as *mut cte_t as usize);
        assert_ne!(cte3.cteMDBNode.get_prev(), &mut cte2 as *mut cte_t as usize);
        assert_ne!(cte2.cteMDBNode.get_next(), &mut cte3 as *mut cte_t as usize);
        assert_ne!(cte2.cteMDBNode.get_prev(), &mut cte1 as *mut cte_t as usize);
        println!("Test cte_move_test passed");
    }

    #[test_case]
    pub fn cte_swap_test() {
        println!("-----------------------------------");
        println!("Entering cte_swap_test case");
        let cap1 = cap_t::new_asid_control_cap();
        let cap2 = cap_t::new_domain_cap();
        let mut cte1 = cte_t {
            cap: cap_t::new_null_cap(),
            cteMDBNode: mdb_node_t::new(0, 0, 0, 0),
        };
        let mut cte2 = cte_t {
            cap: cap_t::new_null_cap(),
            cteMDBNode: mdb_node_t::new(0, 0, 0, 0),
        };

        let mut cte3 = cte_t {
            cap: cap_t::new_null_cap(),
            cteMDBNode: mdb_node_t::new(0, 0, 0, 0),
        };
        let mut cte4 = cte_t {
            cap: cap_t::new_null_cap(),
            cteMDBNode: mdb_node_t::new(0, 0, 0, 0),
        };

        cte_insert(&cap1, &mut cte1, &mut cte2);
        cte_insert(&cap2, &mut cte3, &mut cte4);
        assert_eq!(cte1.cteMDBNode.get_next(), &mut cte2 as *mut cte_t as usize);
        assert_eq!(cte2.cteMDBNode.get_prev(), &mut cte1 as *mut cte_t as usize);
        assert_eq!(cte3.cteMDBNode.get_next(), &mut cte4 as *mut cte_t as usize);
        assert_eq!(cte4.cteMDBNode.get_prev(), &mut cte3 as *mut cte_t as usize);
        cte_swap(&cap1, &mut cte2, &cap2, &mut cte4);
        assert_eq!(cte2.cap.get_cap_type(), CapTag::CapDomainCap);
        assert_eq!(cte4.cap.get_cap_type(), CapTag::CapASIDControlCap);
        assert_eq!(cte4.cteMDBNode.get_prev(), &mut cte1 as *mut cte_t as usize);
        assert_eq!(cte1.cteMDBNode.get_next(), &mut cte4 as *mut cte_t as usize);
        assert_eq!(cte2.cteMDBNode.get_prev(), &mut cte3 as *mut cte_t as usize);
        assert_eq!(cte3.cteMDBNode.get_next(), &mut cte2 as *mut cte_t as usize);

        println!("Test cte_swap_test passed");
    }

    #[test_case]
    pub fn insert_new_cap_test() {
        println!("-----------------------------------");
        println!("Entering insert_new_cap_test case");
        let cap1 = cap_t::new_asid_control_cap();
        let cap2 = cap_t::new_domain_cap();
        let mut cte1 = cte_t {
            cap: cap_t::new_null_cap(),
            cteMDBNode: mdb_node_t::new(0, 0, 0, 0),
        };
        let mut cte2 = cte_t {
            cap: cap_t::new_null_cap(),
            cteMDBNode: mdb_node_t::new(0, 0, 0, 0),
        };
        let mut cte3 = cte_t {
            cap: cap_t::new_null_cap(),
            cteMDBNode: mdb_node_t::new(0, 0, 0, 0),
        };
        cte_insert(&cap1, &mut cte1, &mut cte2);
        assert_eq!(cte2.cap.get_cap_type(), CapTag::CapASIDControlCap);
        assert_eq!(cte1.cteMDBNode.get_next(), &mut cte2 as *mut cte_t as usize);
        assert_eq!(cte2.cteMDBNode.get_prev(), &mut cte1 as *mut cte_t as usize);
        insert_new_cap(&mut cte1, &mut cte3, &cap2);
        assert_eq!(cte1.cteMDBNode.get_next(), &mut cte3 as *mut cte_t as usize);
        assert_eq!(cte3.cteMDBNode.get_next(), &mut cte2 as *mut cte_t as usize);
        assert_eq!(cte2.cteMDBNode.get_prev(), &mut cte3 as *mut cte_t as usize);
        assert_eq!(cte3.cteMDBNode.get_prev(), &mut cte1 as *mut cte_t as usize);
        println!("Test insert_new_cap_test passed");
    }

    #[test_case]
    pub fn resolve_address_bits_test() {
        println!("-----------------------------------");
        println!("Entering resolve_address_bits_test case");
        //cap_ptr structure:
        // guard1(2 bits)|offset1(3 bits)|guard2(2 bits)|offset2(3 bits)
        let buffer: [u8; 1024] = [0; 1024];
        let guardSize = 2;
        let guard1 = 2;
        let guard2 = 3;
        let cap1 = cap_t::new_cnode_cap(3, guardSize, guard1, buffer.as_ptr() as usize);
        let cap2 = cap_t::new_cnode_cap(3, guardSize, guard2, buffer.as_ptr() as usize);
        let mut cte1 = cte_t {
            cap: cap_t::new_null_cap(),
            cteMDBNode: mdb_node_t::new(0, 0, 0, 0),
        };
        let cap3 = cap_t::new_domain_cap();
        let idx: usize = 2;
        let cap_ptr = (guard1 << 8) | (idx << 5) | (guard2 << 3) | idx;
        insert_new_cap(
            &mut cte1,
            convert_to_mut_type_ref(cap1.get_cnode_ptr() + idx * 32),
            &cap2,
        );
        insert_new_cap(
            &mut cte1,
            convert_to_mut_type_ref(cap2.get_cnode_ptr() + idx * 32),
            &cap3,
        );
        let res_ret = resolve_address_bits(&cap1, cap_ptr, 10);
        let ret_cap = unsafe { (*(res_ret.slot)).cap };
        assert_eq!(ret_cap.get_cap_type(), CapTag::CapDomainCap);
        println!("Test resolve_address_bits_test passed");
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
}
