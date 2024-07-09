#[derive(Eq, PartialEq, Debug, Clone, Copy, PartialOrd, Ord)]
/// The label of a message.
pub enum MessageLabel {
    InvalidInvocation = 0,
    UntypedRetype,
    TCBReadRegisters,
    TCBWriteRegisters,
    TCBCopyRegisters,
    TCBConfigure,
    TCBSetPriority,
    TCBSetMCPriority,
    TCBSetSchedParams,
    TCBSetIPCBuffer,
    TCBSetSpace,
    TCBSuspend,
    TCBResume,
    TCBBindNotification,
    TCBUnbindNotification,
    #[cfg(feature = "ENABLE_SMP")]
    TCBSetAffinity,
    TCBSetTLSBase,
    CNodeRevoke,
    CNodeDelete,
    CNodeCancelBadgedSends,
    CNodeCopy,
    CNodeMint,
    CNodeMove,
    CNodeMutate,
    CNodeRotate,
    CNodeSaveCaller,
    IRQIssueIRQHandler,
    IRQAckIRQ,
    IRQSetIRQHandler,
    IRQClearIRQHandler,
    DomainSetSet,
    RISCVPageTableMap,
    RISCVPageTableUnmap,
    RISCVPageMap,
    RISCVPageUnmap,
    RISCVPageGetAddress,
    RISCVASIDControlMakePool,
    RISCVASIDPoolAssign,
    RISCVIRQIssueIRQHandlerTrigger,
    #[cfg(target_arch = "aarch64")]
    ARMPageTableMap,
    #[cfg(target_arch = "aarch64")]
    ARMPageTableUnmap,
    #[cfg(target_arch = "aarch64")]
    ARMIOPageTableMap,
    #[cfg(target_arch = "aarch64")]
    ARMIOPageTableUnmap,
    #[cfg(target_arch = "aarch64")]
    ARMPageMap,
    #[cfg(target_arch = "aarch64")]
    ARMPageUnmap,
    #[cfg(target_arch = "aarch64")]
    ARMPageMapIO,
    #[cfg(target_arch = "aarch64")]
    ARMPageClean_Data,
    #[cfg(target_arch = "aarch64")]
    ARMPageInvalidate_Data,
    #[cfg(target_arch = "aarch64")]
    ARMPageCleanInvalidate_Data,
    #[cfg(target_arch = "aarch64")]
    ARMPageUnify_Instruction,
    #[cfg(target_arch = "aarch64")]
    ARMPageGetAddress,
    #[cfg(target_arch = "aarch64")]
    ARMASIDControlMakePool,
    #[cfg(target_arch = "aarch64")]
    ARMASIDPoolAssign,
    #[cfg(target_arch = "aarch64")]
    ARMVCPUSetTCB,
    #[cfg(target_arch = "aarch64")]
    ARMVCPUInjectIRQ,
    #[cfg(target_arch = "aarch64")]
    ARMVCPUReadReg,
    #[cfg(target_arch = "aarch64")]
    ARMVCPUWriteReg,
    #[cfg(target_arch = "aarch64")]
    ARMVCPUAckVPPI,
    #[cfg(target_arch = "aarch64")]
    ARMIRQIssueIRQHandlerTrigger,
    #[cfg(target_arch = "aarch64")]
    ARMIRQIssueIRQHandlerTriggerCore,
    #[cfg(target_arch = "aarch64")]
    ARMSIDIssueSIDManager,
    #[cfg(target_arch = "aarch64")]
    ARMSIDGetFault,
    #[cfg(target_arch = "aarch64")]
    ARMSIDClearFault,
    #[cfg(target_arch = "aarch64")]
    ARMSIDBindCB,
    #[cfg(target_arch = "aarch64")]
    ARMSIDUnbindCB,
    #[cfg(target_arch = "aarch64")]
    ARMCBIssueCBManager,
    #[cfg(target_arch = "aarch64")]
    ARMCBTLBInvalidateAll,
    #[cfg(target_arch = "aarch64")]
    ARMCBAssignVspace,
    #[cfg(target_arch = "aarch64")]
    ARMCBUnassignVspace,
    #[cfg(target_arch = "aarch64")]
    ARMCBTLBInvalidate,
    #[cfg(target_arch = "aarch64")]
    ARMCBGetFault,
    #[cfg(target_arch = "aarch64")]
    ARMCBClearFault,
    nArchInvocationLabels,
}
