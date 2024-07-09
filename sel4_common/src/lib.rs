//! This crate contains the common code for the seL4 kernel.
//! Such as the seL4 kernel configuration(`Registers`, `Constants`), the seL4 structures(`MessageInfo`, `ObjectType`, `Error`, 'Exception', 'Fault'), and the seL4 utils(`Logging`, `SBI`).
#![no_std]
#![feature(decl_macro)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]

pub mod arch;
pub mod console;
pub mod fault;
mod ffi;
pub mod logging;
pub mod message_info;
pub mod object;
pub mod sel4_config;
#[cfg(feature = "ENABLE_SMP")]
pub mod smp;
pub mod structures;
pub mod utils;
pub mod cap_rights;
pub mod vm_rights;

/// The ffi_call macro, It declares the function and call it
///
/// Usage:
///
/// ```rust
/// ffi_call!(map_kernel_devices);
///
/// // call with arguments
/// // Format is fname(arg_name:type => value) -> ret_type
/// ffi_call!(handleUnknownSyscall, a1:usize => 1);
///
/// ffi_call!(some_function);
///
/// ffi_call!(another_function -> i32);
///
/// ffi_call!(multi_arg_function(a: i32 => 1, b: f64 => 3.14));
///
/// ffi_call!(multi_arg_function_with_return(a: i32 => 1, b: f64 => 3.14) -> i64);
/// ```
pub macro ffi_call {
    ($fname:ident) => {
        {
            extern "C" {
                fn $fname();
            }
            unsafe {
                $fname();
            }
        }
    },
    ($fname:ident->$r:ty) => {
        {
            extern "C" {
                fn $fname() -> $r;
            }
            unsafe {
                $fname()
            }
        }
    },
    // TODO: Add for multi argument functions
    ($fname:ident($( $aname:ident:$t:ty=>$v:expr ),*)) => {
        {
            extern "C" {
                fn $fname($($aname:$t),*);
            }
            unsafe {
                $fname($($v),*);
            }
        }
    },
    ($fname:ident($( $aname:ident:$t:ty=>$v:expr ),*)->$r:ty) => {
        {
            extern "C" {
                fn $fname($($aname:$t),*) -> $r;
            }
            unsafe {
                $fname($($v),*);
            }
        }
    },
}
