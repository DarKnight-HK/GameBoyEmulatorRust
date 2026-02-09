use crate::mmu::Mmu;

pub struct Bus {
    mmu: Mmu,
}

impl Bus {
    pub fn new() -> Self {
        Bus {
            mmu: Mmu::new(),
        }
    }

    // The CPU will call this to read/write
    pub fn read_byte(&self, addr: u16) -> u8 {
        self.mmu.read(addr)
    }

    pub fn write_byte(&mut self, addr: u16, value: u8) {
        self.mmu.write(addr, value);
    }
}