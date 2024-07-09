#![no_std]

//! Reexport definitions in the here.

#[macro_use]
extern crate cfg_if;

use core::ptr::NonNull;

pub use serial_frame::SerialDriver;

cfg_if! {
    if #[cfg(target_arch = "aarch64")] {
        /// Use cfg(driver = "pl011") in the future.
        use serial_impl_pl011::Pl011Uart;
        /// Initialize Default Serial Driver
        // pub static DEFAULT_SERIAL: &dyn SerialDriver = Pl011Uart::new(unsafe { NonNull::new_unchecked(0x900_0000 as _) });

        pub fn default_serial() -> impl SerialDriver {
            // Pl011Uart::new(unsafe { NonNull::new_unchecked(0x900_0000 as _) })
            Pl011Uart::new(unsafe { NonNull::new_unchecked(0xffffffffffe00000usize as _) })
        }
    } else if #[cfg(target_arch = "riscv64")] {
        use serial_impl_sbi::SerialSBI;

        /// Initialize Default Serial Driver
        pub fn default_serial() -> impl SerialDriver {
            // 0xf is a random number, the argument of this function will never be used
            SerialSBI::new(unsafe { NonNull::new_unchecked(0xf as _) })
        }
    }
}

/// Initialize The Drivers
pub fn init() {
    default_serial().init();
}
