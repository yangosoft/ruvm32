pub const MINIRV32_RAM_IMAGE_OFFSET: u32 = 0x80000000;
pub const MINI_RV32_RAM_SIZE: u32 = 0x00100000; // 1 MiB
pub const UVM32_MEMORY_SIZE: u32 = 65536; // 64 KiB
pub const UVM32_SYSCALL_HALT: u32 = 0x1000000;

fn minirv32_load4(ofs: u32, image: &[u8]) -> u32 {
    let offset = ofs as usize;

    // Get 4 bytes from image at offset
    let byte0 = image[offset] as u32;
    let byte1 = image[offset + 1] as u32;
    let byte2 = image[offset + 2] as u32;
    let byte3 = image[offset + 3] as u32;

    let ret: u32 = (byte0) | (byte1 << 8) | (byte2 << 16) | (byte3 << 24);
    return ret;
}

fn minirv32_load1_signed(ofs: u32, image: &[u8]) -> i8 {
    let offset = ofs as usize;
    let byte = image[offset] as i8;
    return byte;
}

fn minirv32_load1(ofs: u32, image: &[u8]) -> u8 {
    let offset = ofs as usize;
    let byte = image[offset] as u8;
    return byte;
}

fn minirv32_load2(ofs: u32, image: &[u8]) -> u16 {
    let offset = ofs as usize;
    let byte0 = image[offset] as u16;
    let byte1 = image[offset + 1] as u16;
    let halfword = (byte0) | (byte1 << 8);
    return halfword;
}

fn minirv32_load2_signed(ofs: u32, image: &[u8]) -> i16 {
    let offset = ofs as usize;
    let byte0 = image[offset] as u16;
    let byte1 = image[offset + 1] as u16;
    let halfword = (byte0) | (byte1 << 8);
    return halfword as i16;
}

fn minirv32_mmio_range(n: u32) -> bool {
    // Example MMIO range check (to be customized as needed)
    0x10000000 <= n && n < 0x12000000
}

fn minirv32_store1(ofs: u32, val: u8, image: &mut [u8]) {
    let offset = ofs as usize;
    image[offset] = (val & 0xff) as u8;
}

fn minirv32_store2(ofs: u32, val: u16, image: &mut [u8]) {
    let offset = ofs as usize;
    image[offset] = (val & 0xff) as u8;
    image[offset + 1] = ((val >> 8) & 0xff) as u8;
}

fn minirv32_store4(ofs: u32, val: u32, image: &mut [u8]) {
    let offset = ofs as usize;
    image[offset] = (val & 0xff) as u8;
    image[offset + 1] = ((val >> 8) & 0xff) as u8;
    image[offset + 2] = ((val >> 16) & 0xff) as u8;
    image[offset + 3] = ((val >> 24) & 0xff) as u8;
}

#[derive(Clone, Default, Copy)]
pub struct MiniRV32IMAState {
    regs: [u32; 32],
    pc: u32,
    mstatus: u32,

    mscratch: u32,
    mtvec: u32,
    mie: u32,
    mip: u32,
    mepc: u32,
    mtval: u32,
    mcause: u32,

    // Note: only a few bits are used.  (Machine = 3, User = 0)
    // Bits 0..1 = privilege.
    // Bit 2 = WFI (Wait for interrupt)
    // Bit 3+ = Load/Store reservation LSBs.
    extraflags: u32,
}

impl MiniRV32IMAState {
    pub fn new() -> Self {
        let mut me = Self {
            regs: [0; 32],
            pc: MINIRV32_RAM_IMAGE_OFFSET,
            mstatus: 0,
            mscratch: 0,
            mtvec: 0,
            mie: 0,
            mip: 0,
            mepc: 0,
            mtval: 0,
            mcause: 0,
            extraflags: 3,
        };

        // https://projectf.io/posts/riscv-cheat-sheet/
        // setup stack pointer
        // la	sp, _sstack
        // addi	sp,sp,-16
        me.regs[2] = ((MINIRV32_RAM_IMAGE_OFFSET + UVM32_MEMORY_SIZE) & !0xF) - 16; // 16 byte align stack
        return me;
    }

    pub fn get_reg(&self, regnum: usize) -> u32 {
        self.regs[regnum]
    }

    pub fn get_pc(&self) -> u32 {
        self.pc
    }

    pub fn increment_pc(&mut self, delta: u32) {
        self.pc = self.pc.wrapping_add(delta);
    }

    pub fn step(&mut self, image: &mut [u8], _v_proc_address: u32, count: i32) -> i32 {
        let mut trap: u32 = 0;
        let mut rval: u32;
        let mut pc: u32 = self.pc;

        for _icount in 0..count {
            let ir: u32;
            rval = 0;

            let ofs_pc: u32 = pc - MINIRV32_RAM_IMAGE_OFFSET;

            if ofs_pc >= MINI_RV32_RAM_SIZE {
                trap = 1 + 1; // Handle access violation on instruction read.
                break;
            } else if ofs_pc & 3 != 0 {
                trap = 1 + 0; //Handle PC-misaligned access
                break;
            } else {
                ir = minirv32_load4(ofs_pc, image);
                let mut rdid: u32 = (ir >> 7) & 0x1f;

                match ir & 0x7f {
                    0x37 => {
                        // LUI (0b0110111)
                        rval = ir & 0xfffff000;
                    }
                    0x17 => {
                        // AUIPC (0b0010111)
                        rval = pc + (ir & 0xfffff000);
                    }
                    0x6F => {
                        // JAL (0b1101111)
                        let mut reladdy: u32 = ((ir & 0x80000000) >> 11)
                            | ((ir & 0x7fe00000) >> 20)
                            | ((ir & 0x00100000) >> 9)
                            | (ir & 0x000ff000);

                        if (reladdy & 0x00100000) != 0 {
                            reladdy |= 0xffe00000; // Sign extension.
                        }
                        rval = pc + 4;
                        pc = pc.wrapping_add(reladdy as u32).wrapping_sub(4);
                    }

                    0x67 => {
                        // JALR (0b1100111)

                        let imm: u32 = ir >> 20;

                        let mut ext = 0;
                        if (imm & 0x800) != 0 {
                            ext = 0xfffff000;
                        }

                        let imm_se: u32 = imm | ext;
                        rval = pc + 4;
                        // #define REG( x ) state->regs[x]
                        let reg_idx = (ir >> 15) & 0x1f;
                        let reg_val = self.regs[reg_idx as usize];
                        pc = (reg_val.wrapping_add(imm_se)) & !1;
                        pc = pc.wrapping_sub(4);

                        /*pc = ( (REG( (ir >> 15) & 0x1f ) + imm_se) & ~1) - 4;*/
                    }

                    0x63 => {
                        // Branch (0b1100011)

                        let mut immm4: u32 = ((ir & 0xf00) >> 7)
                            | ((ir & 0x7e000000) >> 20)
                            | ((ir & 0x80) << 4)
                            | ((ir >> 31) << 12);
                        if (immm4 & 0x1000) != 0 {
                            immm4 |= 0xffffe000;
                        }
                        let reg_idx1 = (ir >> 15) & 0x1f;
                        let reg_idx2 = (ir >> 20) & 0x1f;
                        let rs1: i32 = self.regs[reg_idx1 as usize] as i32;
                        let rs2: i32 = self.regs[reg_idx2 as usize] as i32;
                        immm4 = pc.wrapping_add(immm4).wrapping_sub(4);

                        rdid = 0;
                        match (ir >> 12) & 0x7 {
                            // BEQ, BNE, BLT, BGE, BLTU, BGEU
                            0 => {
                                if rs1 == rs2 {
                                    pc = immm4;
                                }
                            }
                            1 => {
                                if rs1 != rs2 {
                                    pc = immm4;
                                }
                            }
                            4 => {
                                if rs1 < rs2 {
                                    pc = immm4;
                                }
                            }
                            5 => {
                                if rs1 >= rs2 {
                                    pc = immm4;
                                }
                            } //BGE
                            6 => {
                                if (rs1 as u32) < (rs2 as u32) {
                                    pc = immm4;
                                }
                            } //BLTU
                            7 => {
                                if (rs1 as u32) >= (rs2 as u32) {
                                    pc = immm4;
                                }
                            } //BGEU
                            _ => {
                                trap = 2 + 1;
                            }
                        }
                    }

                    0x03 => {
                        // Load (0b0000011)
                        let reg_idx1 = (ir >> 15) & 0x1f;
                        let rs1: u32 = self.regs[reg_idx1 as usize];
                        let imm: u32 = ir >> 20;
                        let imm_se: u32 = if (imm & 0x800) != 0 {
                            imm | 0xfffff000
                        } else {
                            imm
                        };
                        let mut rsval: u32 = rs1.wrapping_add(imm_se);
                        rsval = rsval.wrapping_sub(MINIRV32_RAM_IMAGE_OFFSET);
                        if rsval >= MINI_RV32_RAM_SIZE - 3 {
                            rsval = rsval.wrapping_add(MINIRV32_RAM_IMAGE_OFFSET);
                            if minirv32_mmio_range(rsval)
                            // UART, CLNT
                            {
                                todo!("Peding to do memory-mapped I/O handling");
                                // MINIRV32_HANDLE_MEM_LOAD_CONTROL( rsval, rval );
                            } else {
                                trap = 5 + 1;
                                rval = rsval;
                            }
                        } else {
                            match (ir >> 12) & 0x7 {
                                //LB, LH, LW, LBU, LHU
                                0 => {
                                    rval = minirv32_load1_signed(rsval, image) as u32;
                                }
                                1 => {
                                    rval = minirv32_load2_signed(rsval, image) as u32;
                                }
                                2 => {
                                    rval = minirv32_load4(rsval, image) as u32;
                                }
                                4 => {
                                    rval = minirv32_load1(rsval, image) as u32;
                                }
                                5 => {
                                    rval = minirv32_load2(rsval, image) as u32;
                                }
                                _ => {
                                    trap = 2 + 1;
                                }
                            }
                        }
                    }

                    0x23 => {
                        // Store 0b0100011
                        let reg1 = (ir >> 15) & 0x1f;
                        let reg2 = (ir >> 20) & 0x1f;

                        let rs1: u32 = self.regs[reg1 as usize];
                        let rs2: u32 = self.regs[reg2 as usize];
                        let mut addy: u32 = ((ir >> 7) & 0x1f) | ((ir & 0xfe000000) >> 20);

                        if addy & 0x800 != 0 {
                            addy |= 0xfffff000;
                        }
                        // addy += rs1 - MINIRV32_RAM_IMAGE_OFFSET;
                        addy = addy.wrapping_add(rs1);
                        addy = addy.wrapping_sub(MINIRV32_RAM_IMAGE_OFFSET);
                        rdid = 0;

                        if addy >= MINI_RV32_RAM_SIZE - 3 {
                            addy += MINIRV32_RAM_IMAGE_OFFSET;
                            if minirv32_mmio_range(addy) {
                                todo!("Peding to do memory-mapped I/O handling");
                                //MINIRV32_HANDLE_MEM_STORE_CONTROL( addy, rs2 );
                            } else {
                                trap = 7 + 1; // Store access fault.
                                rval = addy;
                            }
                        } else {
                            match (ir >> 12) & 0x7 {
                                //SB, SH, SW
                                0 => minirv32_store1(addy, rs2 as u8, image),
                                1 => minirv32_store2(addy, rs2 as u16, image),
                                2 => minirv32_store4(addy, rs2 as u32, image),
                                _ => trap = 2 + 1,
                            }
                        }
                    }

                    0x13 | 0x33 => {
                        // Op-immediate 0b0010011
                        // Op           0b0110011
                        let mut imm: u32 = ir >> 20;
                        let mask = if imm & 0x800 != 0 { 0xfffff000 } else { 0 };
                        imm = imm | mask;
                        let reg = (ir >> 15) & 0x1f;
                        let rs1 = self.regs[reg as usize];
                        let reg2 = imm & 0x1f;

                        let is_reg = (!!(ir & 0x20)) != 0;
                        let rs2 = if is_reg {
                            self.regs[reg2 as usize]
                        } else {
                            imm
                        };

                        if is_reg && (ir & 0x02000000 != 0) {
                            match (ir >> 12) & 7 {
                                //0x02000000 = RV32M
                                0 => {
                                    rval = rs1 * rs2;
                                    // MUL
                                }
                                1 => {
                                    rval = ((rs1 as i32).wrapping_div(rs2 as i32)) as u32;
                                    // MULH
                                }
                                2 => {
                                    rval = ((rs1 as i32).wrapping_div(rs2 as i32)) as u32;
                                    // MULHSU
                                }
                                3 => {
                                    rval = ((rs1 as u32).wrapping_div(rs2 as u32)) as u32;
                                    // MULHU
                                }
                                4 => {
                                    rval = ((rs1 as i32).wrapping_rem(rs2 as i32)) as u32;
                                    // DIV
                                }
                                5 => {
                                    rval = ((rs1 as u32).wrapping_rem(rs2 as u32)) as u32;
                                    // DIVU
                                }
                                6 => {
                                    rval = ((rs1 as i32).wrapping_rem(rs2 as i32)) as u32;
                                    // REM
                                }
                                7 => {
                                    rval = ((rs1 as u32).wrapping_rem(rs2 as u32)) as u32;
                                    // REMU
                                }
                                _ => {
                                    trap = 2 + 2;
                                }
                            }
                        } else {
                            match ir >> 12 & 7 {
                                0 => {
                                    rval = if is_reg && (ir & 0x40000000) != 0 {
                                        rs1 - rs2
                                    } else {
                                        //ignore overflow
                                        rs1.wrapping_add(rs2)
                                    };
                                    // ADD/SUB
                                }
                                1 => {
                                    rval = rs1.wrapping_shl(rs2 & 0x1f);
                                    // SLL
                                }
                                2 => {
                                    rval = ((rs1 as i32).wrapping_shr((rs2 & 0x1f) as u32)) as u32;
                                    // SLT
                                }
                                3 => {
                                    rval = if (rs1 as u32) < (rs2 as u32) { 1 } else { 0 };
                                    // SLTU
                                }
                                4 => {
                                    rval = rs1 ^ rs2;
                                    // XOR
                                }
                                5 => {
                                    rval = if (ir & 0x40000000) != 0 {
                                        ((rs1 as i32).wrapping_shr((rs2 & 0x1f) as u32)) as u32
                                    } else {
                                        rs1.wrapping_shr(rs2 & 0x1f)
                                    };
                                    // SRL/SRA
                                }
                                6 => {
                                    rval = rs1 | rs2;
                                    // OR
                                }
                                7 => {
                                    rval = rs1 & rs2;
                                    // AND
                                }
                                _ => {
                                    trap = 2 + 2; // Illegal instruction
                                }
                            }
                        }
                    }

                    0x0f => {
                        // 0b0001111
                        rdid = 0; // fencetype = (ir >> 12) & 0b111; We ignore fences in this impl.
                    }

                    0x73 => {
                        // Zifencei+Zicsr  (0b1110011)
                        let csrno = ir >> 20;
                        let microop = (ir >> 12) & 0x7;
                        if microop & 3 != 0 {
                            // It's a Zicsr function.
                            let rs1imm: u32 = (ir >> 15) & 0x1f;
                            let rs1 = self.regs[rs1imm as usize];
                            let mut writeval = rs1;
                            match csrno {
                                0x340 => {
                                    rval = self.mscratch;
                                }
                                0x305 => {
                                    rval = self.mtvec;
                                }
                                0x304 => {
                                    rval = self.mie;
                                }
                                0x341 => {
                                    rval = self.mepc;
                                }
                                0x344 => {
                                    rval = self.mip;
                                }
                                0x343 => {
                                    rval = self.mtval;
                                }
                                0xf11 => {
                                    //vendor id
                                    rval = 0xff0ff0ff;
                                }
                                0x301 => {
                                    rval = 0x40401101;
                                    //misa (XLEN=32, IMA+X)
                                }
                                0x300 => {
                                    // Not sure!!!!!
                                    rval = self.mstatus;
                                }
                                _ => {
                                    // MINIRV32_OTHERCSR_READ( csrno, rval );
                                    todo!("CSR not implemented: {:#x}", csrno);
                                }
                            }

                            match microop {
                                1 => {
                                    //CSRRW
                                    writeval = rs1;
                                }
                                2 => {
                                    //CSRRS
                                    writeval = rval | rs1;
                                }
                                3 => {
                                    //CSRRC
                                    writeval = rval & (!rs1);
                                }
                                5 => {
                                    //CSRRWI
                                    writeval = rs1imm;
                                }
                                6 => {
                                    //CSRRSI
                                    writeval = rval | rs1imm;
                                }
                                7 => {
                                    //CSRRCI
                                    writeval = rval & (!rs1imm);
                                }
                                _ => {
                                    trap = 2 + 2; // Illegal instruction
                                }
                            }

                            match csrno {
                                0x340 => {
                                    self.mscratch = writeval;
                                }
                                0x305 => {
                                    self.mtvec = writeval;
                                }
                                0x304 => {
                                    self.mie = writeval;
                                }
                                0x344 => {
                                    self.mip = writeval;
                                }
                                0x341 => {
                                    self.mepc = writeval;
                                }
                                0x342 => {
                                    self.mcause = writeval;
                                }
                                0x343 => {
                                    self.mtval = writeval;
                                }
                                0x300 => {
                                    self.mstatus = writeval;
                                }
                                _ => {
                                    todo!("CSR not implemented: {:#x}", csrno);
                                }
                            }
                        } else if microop == 0x0 {
                            // "SYSTEM" 0b000
                            rdid = 0;

                            if (csrno & 0xff) == 0x02 {
                                // MRET
                                //https://raw.githubusercontent.com/riscv/virtual-memory/main/specs/663-Svpbmt.pdf
                                //Table 7.6. MRET then in mstatus/mstatush sets MPV=0, MPP=0, MIE=MPIE, and MPIE=1. La
                                // Should also update mstatus to reflect correct mode.
                                let startmstatus = self.mstatus;
                                let startextraflags = self.extraflags;
                                self.mstatus = ((startmstatus & 0x80) >> 4)
                                    | ((startextraflags & 3) << 11)
                                    | 0x80;
                                self.extraflags =
                                    (startextraflags & !3) | ((startmstatus >> 11) & 3);
                                //SETCSR( mstatus , (( startmstatus & 0x80) >> 4) | ((startextraflags&3) << 11) | 0x80 );
                                //SETCSR( extraflags, (startextraflags & ~3) | ((startmstatus >> 11) & 3) );
                                pc = self.mepc - 4;
                            } else {
                                match csrno {
                                    0 => {
                                        trap = if self.extraflags & 3 != 0 {
                                            11 + 1
                                        } else {
                                            8 + 1
                                        }; // ECALL; 8 = "Environment call from U-mode"; 11 = "Environment call from M-mode"
                                    }

                                    1 => {
                                        trap = 3 + 1;
                                        // EBREAK 3 = "Breakpoint"
                                    }

                                    0x105 => {
                                        //WFI (Wait for interrupts)
                                        self.mstatus |= 8; //Enable interrupts
                                        self.extraflags |= 4; //Infor environment we want to go to sleep.
                                        self.pc = self.pc + 4;
                                        return 1;
                                    }

                                    _ => {
                                        trap = 2 + 1;
                                    }
                                }
                            }
                        } else if microop == 0x2 {
                            // Zifencei
                            rdid = 0;
                            match csrno {
                                0x300 => {
                                    rval = self.mstatus;
                                }
                                _ => {
                                    trap = 2 + 2; // Illegal instruction
                                }
                            }
                        } else {
                            trap = 2 + 1;
                        }
                    }

                    0x2f => {
                        // RV32A (0b00101111)
                        todo!("Atomic instructions not yet implemented");
                    }
                    _ => {
                        trap = 2 + 2; // Illegal instruction
                    }
                }

                // If there was a trap, do NOT allow register writeback.
                if trap != 0 {
                    self.pc = pc;
                    //MINIRV32_POSTEXEC( pc, ir, trap );
                    break;
                }
                if rdid != 0 {
                    self.regs[rdid as usize] = rval; // Write back register.
                }
            }

            //MINIRV32_POSTEXEC( pc, ir, trap );

            pc = pc.wrapping_add(4);
        }

        if trap != 0 {
            return trap.try_into().unwrap();
            /*if trap & 0x80000000 != 0 {
                self.mcause = trap;
                self.mtval = 0;
                pc += 4; // PC needs to point to where the PC will return to.
            } else {
                self.mcause = trap - 1;
                self.mtval = if trap > 5 && trap <= 8 { rval } else { pc };
            }

            self.mepc = pc; //TRICKY: The kernel advances mepc automatically.
            //CSR( mstatus ) & 8 = MIE, & 0x80 = MPIE
            // On an interrupt, the system moves current MIE into MPIE
            self.mstatus = ((self.mstatus & 0x08) << 4) | ((self.extraflags & 3) << 11);
            pc = (self.mtvec - 4);

            // If trapping, always enter machine mode.
            self.extraflags |= 3;
            trap = 0;
            pc += 4;*/
        }

        self.pc = pc;
        print!("PC = {:#010x}\n", self.pc);
        0
    }
}
