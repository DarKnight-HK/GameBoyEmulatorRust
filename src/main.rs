mod bus;
mod cartridge;
mod cpu;
mod ppu;

use bus::Bus;
use cpu::Cpu;


use crate::cartridge::Cartridge;

fn main() {
    let bus = Bus::new(Cartridge::new("sml.gb").unwrap());
    let mut cpu = Cpu::new(bus);

    loop {
        let pc = cpu.pc;
        let opcode = cpu.bus.read_byte(pc);
        // println!("PC:{:#04X} | OP:{:#02X} | A:{:#02X} | F:{:#02X}", 
        //          pc, opcode, cpu.a, cpu.f);

        // 3. Step
        let cycles = cpu.step();

        if cycles == 0 {
            println!("Crash detected!");
            break;
        }
    
    }
}