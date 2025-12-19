use std::env;

mod rv32ima;

fn main() {
    let args: Vec<String> = env::args().collect();
    let path: String;
    if args.len() < 2 {
        //path = "/home/yango/proj/ruvm32/freertos/FreeRTOS-LTS/FreeRTOS/FreeRTOS-Kernel/test1.bin"
        //    .to_string()
        path = "/home/yango/proj/ruvm32/example_in_c/test1.bin".to_string()
    } else {
        path = args[1].clone();
    }

    let rom = std::fs::read(path).expect("Failed to read ROM file");

    let mut cpu = rv32ima::MiniRV32IMAState::new();

    let mut memory: Vec<u8> = vec![0; rv32ima::UVM32_MEMORY_SIZE as usize];
    memory[0..rom.len()].copy_from_slice(&rom);

    loop {
        let ret = cpu.step(&mut memory, 0, 1);
        match ret {
            0 => {
                println!("Stepped successfully to PC={:08x}", cpu.get_pc());
                if cpu.get_pc() == 0x80000138 {
                    println!("PING!!!!");
                }
            }
            12 => {
                /*
                // Fetch registers used by syscall
                const uint32_t syscall = vmst->_core.regs[17];  // a7
                // on exception we should jump to mtvec, but we handle directly
                // and skip over the ecall instruction
                vmst->_core.pc += 4; */

                let syscall = cpu.get_reg(17); // a7 
                match syscall {
                    rv32ima::UVM32_SYSCALL_HALT => {
                        println!("SYSCALL HALT encountered at PC={:08x}", cpu.get_pc());
                        break;
                    }
                    64 => {
                        println!("TICK1 {:08x} at PC={:08x}", syscall, cpu.get_pc());
                        let mvtec = cpu.get_mvtec();
                        println!(" mtvec = {:08x}", mvtec);

                        cpu.increment_pc(4);
                        //break;
                    }
                    65 => {
                        println!("TICK2 {:08x} at PC={:08x}", syscall, cpu.get_pc());
                        cpu.increment_pc(4);
                        //break;
                    }
                    66 => {
                        println!(
                            "External interrupt handler {:08x} at PC={:08x}",
                            syscall,
                            cpu.get_pc()
                        );
                        cpu.increment_pc(4);
                        //break;
                    }
                    _ => {
                        println!("Unknown SYSCALL {:08x} at PC={:08x}", syscall, cpu.get_pc());
                        cpu.increment_pc(4);
                        break;
                    }
                }
            }
            _ => {
                println!("Halting with code {}", ret);
                break;
            }
        }
    }
}
