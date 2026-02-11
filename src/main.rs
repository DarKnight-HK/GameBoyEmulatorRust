mod bus;
mod cartridge;
mod cpu;
mod ppu;

use bus::Bus;
use cpu::Cpu;

use crate::cartridge::Cartridge;

fn main() {
    let bus = Bus::new(Cartridge::new("tetris.gb").unwrap());
    let mut cpu = Cpu::new(bus);

    loop {
        let cycles = cpu.step();

        if cycles == 0 {
            println!("Crash detected!");
            break;
        }
    }
}
