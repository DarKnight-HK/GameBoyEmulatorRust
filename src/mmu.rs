pub struct Mmu {
    memory: [u8; 0xFFFF], // 64KB RAM
}

impl Mmu {
    pub fn new() -> Self {
        Mmu {
            memory: [0; 0xFFFF],
        }
    }

    pub fn read(&self, addr: u16) -> u8 {
        self.memory[addr as usize]
    }

    pub fn write(&mut self, addr: u16, value: u8) {
        self.memory[addr as usize] = value;
    }
}