//     X0                          = 0,    /* 0x00 */
pub const capRegister: usize = 0;
pub const badgeRegister: usize = 0;
//     X1                          = 1,    /* 0x08 */
pub const msgInfoRegister: usize = 1;
//     X2                          = 2,    /* 0x10 */
//     X3                          = 3,    /* 0x18 */
//     X4                          = 4,    /* 0x20 */
//     X5                          = 5,    /* 0x28 */
//     X6                          = 6,    /* 0x30 */
// #ifdef CONFIG_KERNEL_MCS
//     replyRegister               = 6,
// #endif
//     X7                          = 7,    /* 0x38 */
//     X8                          = 8,    /* 0x40 */
// #ifdef CONFIG_KERNEL_MCS
//     nbsendRecvDest              = 8,
// #endif
//     X9                          = 9,    /* 0x48 */
//     X10                         = 10,   /* 0x50 */
//     X11                         = 11,   /* 0x58 */
//     X12                         = 12,   /* 0x60 */
//     X13                         = 13,   /* 0x68 */
//     X14                         = 14,   /* 0x70 */
//     X15                         = 15,   /* 0x78 */
//     X16                         = 16,   /* 0x80 */
//     X17                         = 17,   /* 0x88 */
//     X18                         = 18,   /* 0x90 */
//     X19                         = 19,   /* 0x98 */
//     X20                         = 20,   /* 0xa0 */
//     X21                         = 21,   /* 0xa8 */
//     X22                         = 22,   /* 0xb0 */
//     X23                         = 23,   /* 0xb8 */
//     X24                         = 24,   /* 0xc0 */
//     X25                         = 25,   /* 0xc8 */
//     X26                         = 26,   /* 0xd0 */
//     X27                         = 27,   /* 0xd8 */
//     X28                         = 28,   /* 0xe0 */
//     X29                         = 29,   /* 0xe8 */
//     X30                         = 30,   /* 0xf0 */
//     LR                          = 30,

//     /* End of GP registers, the following are additional kernel-saved state. */
pub const SP_EL0: usize = 31;
pub const ELR_EL1: usize = 32;
pub const NextIP: usize = 32;
pub const SPSR_EL1: usize = 33;
pub const FaultIP: usize = 34;
//     /* user readable/writable thread ID register.
//      * name comes from the ARM manual */
pub const TPIDR_EL0: usize = 35;
//     TLS_BASE                    = TPIDR_EL0,
pub const TLS_BASE: usize = TPIDR_EL0;
//     /* user readonly thread ID register. */
//     TPIDRRO_EL0                 = 36,
pub const n_contextRegisters: usize = 37;
pub const n_exceptionMessage: usize = 3;
pub const n_syscallMessage: usize = 12;
pub const n_msgRegisters: usize = 4;
pub const msgRegister: [usize; n_msgRegisters] = [2, 3, 4, 5];
pub const MAX_MSG_SIZE: usize = n_syscallMessage;
pub const fault_messages: [[usize; MAX_MSG_SIZE]; 2] = [
    [0, 1, 2, 3, 4, 5, 6, 7, 34, 31, 32, 33],
    [34, 31, 33, 0, 0, 0, 0, 0, 0, 0, 0, 0],
];