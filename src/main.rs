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
    println!("{:#?}", cart.as_ref().unwrap().header);
    match cart.unwrap().verify_checksum() {
        true => {  
            println!("Verifcation Successful!");
        }
        false => {
            println!("Failed to verify checksum");
        }
    }
}
