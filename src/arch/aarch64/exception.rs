use crate::arch::aarch64::consts::ARMDataAbort;
use crate::arch::aarch64::consts::ARMPrefetchAbort;
use crate::kernel::boot::current_fault;
use crate::syscall::handle_fault;
use aarch64_cpu::registers::Readable;
use aarch64_cpu::registers::TTBR0_EL1;
use sel4_common::arch::ArchReg;
use sel4_common::fault::seL4_Fault_t;
use sel4_common::structures::exception_t;
use sel4_common::utils::global_read;
use sel4_task::{activateThread, get_currenct_thread, get_current_domain, schedule};

use super::instruction::*;

#[no_mangle]
pub fn handleUserLevelFault(w_a: usize, w_b: usize) -> exception_t {
    unsafe {
        current_fault = seL4_Fault_t::new_user_exeception(w_a, w_b);
        handle_fault(get_currenct_thread());
    }
    schedule();
    activateThread();
    exception_t::EXCEPTION_NONE
}

#[no_mangle]
pub fn handleVMFaultEvent(vm_faultType: usize) -> exception_t {
    let status = handle_vm_fault(vm_faultType);
    if status != exception_t::EXCEPTION_NONE {
        handle_fault(get_currenct_thread());
    }
    schedule();
    activateThread();
    exception_t::EXCEPTION_NONE
}

pub fn handle_vm_fault(type_: usize) -> exception_t {
    /*
    exception_t handleVMFault(tcb_t *thread, vm_fault_type_t vm_faultType)
    {
        switch (vm_faultType) {
        case ARMDataAbort: {
            word_t addr, fault;
            addr = getFAR();
            fault = getDFSR();
    #ifdef CONFIG_ARM_HYPERVISOR_SUPPORT
            /* use the IPA */
            if (ARCH_NODE_STATE(armHSVCPUActive)) {
                addr = GET_PAR_ADDR(addressTranslateS1(addr)) | (addr & MASK(PAGE_BITS));
            }
    #endif
            current_fault = seL4_Fault_VMFault_new(addr, fault, false);
            return EXCEPTION_FAULT;
        }
        case ARMPrefetchAbort: {
            word_t pc, fault;
            pc = getRestartPC(thread);
            fault = getIFSR();

            current_fault = seL4_Fault_VMFault_new(pc, fault, true);
            return EXCEPTION_FAULT;
        }
        default:
            fail("Invalid VM fault type");
        }
    }
    */
    // ARMDataAbort = seL4_DataFault,               0
    // ARMPrefetchAbort = seL4_InstructionFault     1
    log::debug!(
        "Handle VM fault: {}  domain: {}",
        type_,
        get_current_domain()
    );
    match type_ {
        ARMDataAbort => {
            let addr = get_far();
            let fault = get_esr();
            log::debug!("fault addr: {:#x} esr: {:#x}", addr, fault);
            unsafe {
                current_fault = seL4_Fault_t::new_vm_fault(addr, fault, 0);
            }
            log::debug!("current_fault: {:#x?}", global_read!(current_fault));
            exception_t::EXCEPTION_FAULT
        }
        ARMPrefetchAbort => {
            let pc = get_currenct_thread().tcbArch.get_register(ArchReg::FaultIP);
            let fault = get_esr();
            unsafe {
                current_fault = seL4_Fault_t::new_vm_fault(pc, fault, 1);
            }

            log::debug!("ttbr0_el1: {:#x?}", TTBR0_EL1.get());

            log::debug!("fault pc: {:#x}  fault: {:#x}", pc, fault);
            exception_t::EXCEPTION_FAULT
        }
        _ => panic!("Invalid VM fault type:{}", type_),
    }
}
