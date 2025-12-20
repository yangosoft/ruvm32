#include "uvm32_sys.h"

.equ uvm32_syscall_halt,          0x1000000
.equ uvm32_syscall_yield,         0x1000001
.equ uvm32_syscall_stackprotect,  0x1000002

.section .initial_jump , "ax", %progbits
.global _start
.align 4
_start:

# Uncomment below to enable stack protection, requires -DUVM32_STACK_PROTECTION
# la a0, _estack
# li a7, uvm32_syscall_stackprotect
# ecall

# Write 0xAA to mvtec via CSR (assume CSR address 0x7C0)
# Requires Zicsr (for csrw) and Zifencei (for fence.i)
li t1, 0xAA
csrw 0x305, t1      # Zicsr extension


# sp is already setup by vm
sw	ra,12(sp)
jal	ra, main

li a7, uvm32_syscall_halt
ecall

.section .data

/* Macro to handle interrupts: expands to external_interrupt_handler(uint32_t cause) */
#define portasmHANDLE_INTERRUPT external_interrupt_handler

