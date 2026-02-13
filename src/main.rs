mod bus;
mod cartridge;
mod cpu;
mod ppu;
mod timer;

use bus::Bus;
use cpu::Cpu;

use crate::cartridge::Cartridge;

fn main() {
    let bus = Bus::new(Cartridge::new("cpu_instrs.gb").unwrap());
    let mut cpu = Cpu::new(bus);

    loop {
        let cycles = cpu.step();
        cpu.bus.tick(cycles);
        if cycles == 0 {
            println!("Crash detected!");
            break;
        }
    }
}
