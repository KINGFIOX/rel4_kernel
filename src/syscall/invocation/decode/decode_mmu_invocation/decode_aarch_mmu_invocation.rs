use sel4_common::{
    message_info::MessageLabel,
    structures::{exception_t, seL4_IPCBuffer},
};
use sel4_cspace::interface::cte_t;

pub fn decode_mmu_invocation(
    label: MessageLabel,
    length: usize,
    slot: &mut cte_t,
    call: bool,
    buffer: Option<&seL4_IPCBuffer>,
) -> exception_t {
    exception_t::EXCEPTION_NONE
}
