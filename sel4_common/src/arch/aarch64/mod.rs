#![allow(unused)]
mod arch_tcb;
mod message_info;
mod object;
mod registers;
mod vm_rights;
pub use arch_tcb::ArchTCB;
pub use message_info::*;
pub use object::*;
pub use registers::*;
pub use vm_rights::*;

pub const PSCI_0_2_FN_BASE: u32 = 0x84000000;
pub const PSCI_0_2_64BIT: u32 = 0x40000000;
pub const PSCI_0_2_FN_CPU_SUSPEND: u32 = PSCI_0_2_FN_BASE + 1;
pub const PSCI_0_2_FN_CPU_OFF: u32 = PSCI_0_2_FN_BASE + 2;
pub const PSCI_0_2_FN_CPU_ON: u32 = PSCI_0_2_FN_BASE + 3;
pub const PSCI_0_2_FN_MIGRATE: u32 = PSCI_0_2_FN_BASE + 5;
pub const PSCI_0_2_FN_SYSTEM_OFF: u32 = PSCI_0_2_FN_BASE + 8;
pub const PSCI_0_2_FN_SYSTEM_RESET: u32 = PSCI_0_2_FN_BASE + 9;
pub const PSCI_0_2_FN64_CPU_SUSPEND: u32 = PSCI_0_2_FN_BASE + PSCI_0_2_64BIT + 1;
pub const PSCI_0_2_FN64_CPU_ON: u32 = PSCI_0_2_FN_BASE + PSCI_0_2_64BIT + 3;
pub const PSCI_0_2_FN64_MIGRATE: u32 = PSCI_0_2_FN_BASE + PSCI_0_2_64BIT + 5;

/// PSCI return values, inclusive of all PSCI versions.
#[derive(PartialEq, Debug)]
#[repr(i32)]
pub enum PsciError {
    NotSupported = -1,
    InvalidParams = -2,
    Denied = -3,
    AlreadyOn = -4,
    OnPending = -5,
    InternalFailure = -6,
    NotPresent = -7,
    Disabled = -8,
    InvalidAddress = -9,
}

impl From<i32> for PsciError {
    fn from(code: i32) -> PsciError {
        use PsciError::*;
        match code {
            -1 => NotSupported,
            -2 => InvalidParams,
            -3 => Denied,
            -4 => AlreadyOn,
            -5 => OnPending,
            -6 => InternalFailure,
            -7 => NotPresent,
            -8 => Disabled,
            -9 => InvalidAddress,
            _ => panic!("Unknown PSCI error code: {}", code),
        }
    }
}

/// psci "hvc" method call
fn psci_hvc_call(func: u32, arg0: usize, arg1: usize, arg2: usize) -> usize {
    let ret;
    unsafe {
        core::arch::asm!(
            "hvc #0",
            inlateout("x0") func as usize => ret,
            in("x1") arg0,
            in("x2") arg1,
            in("x3") arg2,
        )
    }
    ret
}

fn psci_call(func: u32, arg0: usize, arg1: usize, arg2: usize) -> Result<(), PsciError> {
    // let ret = match axconfig::PSCI_METHOD {
    //     "smc" => arm_smccc_smc(func, arg0, arg1, arg2),
    //     "hvc" => psci_hvc_call(func, arg0, arg1, arg2),
    //     _ => panic!("Unknown PSCI method: {}", axconfig::PSCI_METHOD),
    // };
    let ret = psci_hvc_call(func, arg0, arg1, arg2);
    if ret == 0 {
        Ok(())
    } else {
        Err(PsciError::from(ret as i32))
    }
}

pub fn set_timer(timer: usize) {
    todo!("set_timer")
}

pub fn clear_ipi() {
    todo!("clear_ipi")
}
pub fn shutdown() -> ! {
    log::info!("Shutting down...");
    psci_call(PSCI_0_2_FN_SYSTEM_OFF, 0, 0, 0).ok();
    panic!("It should shutdown!");
}

pub fn sys_write(fd: usize, buffer: &[u8]) {
    todo!("sys_write");
}

pub fn remote_sfence_vma(hart_mask: usize, start: usize, size: usize) {
    todo!("remote_sfence_vma");
}

pub fn get_time() -> usize {
    todo!("get_time")
}
