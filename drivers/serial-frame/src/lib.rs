#![no_std]

//! This is a Serial Framework

use core::ptr::NonNull;

/// This is a serial interface difinition.
pub trait SerialDriver {
    /// Get the serial driver
    fn new(addr: NonNull<usize>) -> Self;
    /// Initialize the serial driver.
    /// Why not implement with in [SerialDriver::new]?
    /// Answer: We want to reuse the driver in the user space.
    fn init(&self);
    /// Put a character to serial.
    fn putchar(&self, c: u8);
    /// Get a character from serial
    fn getchar(&self) -> Option<u8>;
}
