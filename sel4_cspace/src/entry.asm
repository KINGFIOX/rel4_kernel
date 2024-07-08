    .section .text.entry
    .globl _start
    .global trap_entry
    .global c_handle_syscall 
_start:
    la sp, boot_stack_top
    call call_test_main

trap_entry:
    j c_handle_syscall

    .section .bss.stack
    .globl boot_stack_lower_bound
boot_stack_lower_bound:
    .space 4096 * 16
    .globl boot_stack_top
boot_stack_top:
