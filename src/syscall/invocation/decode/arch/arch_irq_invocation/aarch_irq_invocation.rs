use sel4_common::arch::MessageLabel;
use sel4_common::structures::{exception_t, seL4_IPCBuffer};
use sel4_cspace::interface::cte_t;
pub fn arch_decode_irq_control_invocation(
    label: MessageLabel,
    length: usize,
    src_slot: &mut cte_t,
    buffer: Option<&seL4_IPCBuffer>,
) -> exception_t {
    todo!()
}
