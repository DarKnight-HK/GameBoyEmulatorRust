use crate::cartridge::Cartridge;

pub struct Bus {
    cartridge: Cartridge,
    pub ie_reg: u8,   // 0xFFFF
    pub int_flag: u8, // 0xFF0F
    pub oam: [u8; 160],
    pub vram: [u8; 8192],
    wram: [u8; 8192],
    hram: [u8; 127],
}

impl Bus {
    pub fn new(cartridge: Cartridge) -> Self {
        Bus {
            ie_reg: 0,
            int_flag: 0,
            cartridge,
            vram: [0; 8192],
            wram: [0; 8192],
            hram: [0; 127],
            oam: [0; 160],
        }
    }

    pub fn read_byte(&self, address: u16) -> u8 {
        match address {
            0xFF0F => self.int_flag,
            0xFFFF => self.ie_reg,
            0x0000..=0x7FFF => self.cartridge.read(address),

            0x8000..=0x9FFF => self.vram[(address - 0x8000) as usize],

            0xA000..=0xBFFF => self.cartridge.read(address),

            0xC000..=0xDFFF => self.wram[(address - 0xC000) as usize],

            0xE000..=0xFDFF => self.wram[(address - 0xE000) as usize],

            0xFE00..=0xFE9F => self.oam[(address - 0xFE00) as usize],

            0xFEA0..=0xFEFF => 0x00,

            0xFF00..=0xFF7F => 0x00,

            0xFF80..=0xFFFE => self.hram[(address - 0xFF80) as usize],

            0xFFFF => 0x00,
        }
    }

    pub fn write_byte(&mut self, address: u16, byte: u8) {
        match address {
            0x0000..=0x7FFF => { /* Handle MBC Later */ }
            0x8000..=0x9FFF => self.vram[(address - 0x8000) as usize] = byte,
            0xA000..=0xBFFF => self.cartridge.write(address, byte),
            0xFE00..=0xFE9F => self.oam[(address - 0xFE00) as usize] = byte,
            0xFF00..=0xFF7F => {
                match address {
                    0xFF0F => self.int_flag = byte,
                    0xFF44 => {} // LY is Read-Only. Ignore writes.
                    _ => {
                        // For now, ignore other IO writes (Serial, Timer, Audio)
                    }
                }
            }
            0xC000..=0xDFFF => self.wram[(address - 0xC000) as usize] = byte,
            0xFFFF => {
                self.ie_reg = byte;
            }

            _ => {}
        }
    }
}
