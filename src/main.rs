mod bus;
mod cartridge;
mod cpu;
mod mmu;
mod ppu;

use bus::Bus;
use cpu::Cpu;
use mmu::Mmu;

use crate::cartridge::Cartridge;

fn main() {
    let cart = Cartridge::new("sml.gb");
    println!("{:#?}", cart.unwrap().header);
}
