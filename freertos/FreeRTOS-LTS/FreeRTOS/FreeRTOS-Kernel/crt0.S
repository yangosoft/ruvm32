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

# TODO change 0xAA for the rigth address of the PC 
# 0x305 writes into mvtec
li t1, 0xAA
csrw 0x305, t1      # Zicsr extension

# sp is already setup by vm
sw	ra,12(sp)
jal	ra, main

li a7, uvm32_syscall_halt
ecall

.section .data

