PREFIX:=riscv64-elf-
OPT ?= -Os
CFLAGS+=-I. -I./include -I./portable/GCC/RISC-V/ -I./portable\GCC\RISC-V-RV32\chip_specific_extensions\RV32I_CLINT_no_extensions -I./portable/GCC/RISC-V/chip_specific_extensions/RISCV_no_extensions/
CFLAGS+=${OPT} -fno-stack-protector -fno-builtin-memcpy -fno-builtin
CFLAGS+=-static-libgcc -fdata-sections -ffunction-sections
CFLAGS+=-g -march=rv32im_zicsr -mabi=ilp32 -static
LDFLAGS:= -T linker.ld -nostdlib -Wl,--gc-sections
LIBS:= -lgcc # needed for softfp
# rv32imc_zicsr

PROJECT:=test1
SRCS:=test1.c crt0.S \
	croutine.c \
	event_groups.c\
	list.c\
	queue.c\
	tasks.c\
	timers.c\
	string.c\
	portable/GCC/RISC-V/portASM.S \
	portable/GCC/RISC-V/port.c

# check if the compiler is installed
UNAME_S := $(shell uname -s)
ifeq ($(UNAME_S),Darwin)
    PREFIX=riscv64-elf-
    ifeq (,$(shell which ${PREFIX}gcc))
        $(error "Try brew install riscv64-elf-gcc riscv64-elf-binutils")
    endif
else
    PREFIX=riscv64-unknown-elf-
    ifeq (,$(shell which ${PREFIX}gcc))
        $(error "Try apt-get install gcc-riscv64-unknown-elf")
    endif
endif

all_common:
	${PREFIX}gcc -o ${PROJECT}.elf ${CFLAGS} ${LDFLAGS} ${SRCS} ${LIBS}
	@$(PREFIX)objcopy ${PROJECT}.elf -O binary ${PROJECT}.bin

disasm_common: ${PROJECT}.elf
	$(PREFIX)objdump -S -d -f ${PROJECT}.elf

test_common: all
	${TOPDIR}/hosts/host/host ${HOST_EXTRA} ${PWD}/${PROJECT}.bin

clean_common:
	rm -f ${PROJECT}.o ${PROJECT}.elf ${PROJECT}.bin

