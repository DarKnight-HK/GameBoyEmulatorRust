use crate::cartridge::Cartridge;
use crate::dma::Dma;
use crate::interrupts::Interrupt;
use crate::ppu::Ppu;
use crate::timer::Timer;

pub struct Bus {
    pub ppu: Ppu,
    pub timer: Timer,
    pub dma: Dma,
    cartridge: Cartridge,
    pub ie_reg: u8,
    pub int_flag: u8,
    hram: [u8; 127],
}

impl Bus {
    pub fn new(cartridge: Cartridge) -> Self {
        Bus {
            ppu: Ppu::new(),
            timer: Timer::new(),
            dma: Dma::new(),
            cartridge,
            ie_reg: 0,
            int_flag: 0,
            hram: [0; 127],
        }
    }

    pub fn tick(&mut self, cycles: u8) {
        if self.timer.tick(cycles) {
            self.request_interrupt(Interrupt::Timer);
        }

        let (vblank, stat) = self.ppu.tick(cycles);
        if vblank {
            self.request_interrupt(Interrupt::VBlank);
        }
        if stat {
            self.request_interrupt(Interrupt::LcdStat);
        }
    }

    pub fn request_interrupt(&mut self, interrupt: Interrupt) {
        self.int_flag |= interrupt.mask();
    }

    pub fn read_byte(&mut self, address: u16) -> u8 {
        match address {
            0x0000..=0x7FFF => self.cartridge.read(address),

            0x8000..=0x9FFF => self.ppu.read(address),

            0xA000..=0xBFFF => self.cartridge.read(address),
            0xC000..=0xDFFF => 0,
            0xE000..=0xFDFF => 0,

            0xFE00..=0xFE9F => self.ppu.read(address),

            0xFF00..=0xFF7F => match address {
                0xFF0F => self.int_flag,
                0xFF04..=0xFF07 => self.timer.read(address),

                0xFF40..=0xFF45 | 0xFF47..=0xFF4B => self.ppu.read(address),

                0xFF46 => self.dma.byte,
                _ => 0xFF,
            },
            0xFF80..=0xFFFE => self.hram[(address - 0xFF80) as usize],
            0xFFFF => self.ie_reg,
            _ => 0xFF,
        }
    }

    pub fn write_byte(&mut self, address: u16, byte: u8) {
        match address {
            0x0000..=0x7FFF => {}

            0x8000..=0x9FFF => self.ppu.write(address, byte),

            0xA000..=0xBFFF => self.cartridge.write(address, byte),
            0xC000..=0xDFFF => {} // WRAM Logic

            0xFE00..=0xFE9F => self.ppu.write(address, byte),

            0xFF00..=0xFF7F => match address {
                0xFF0F => self.int_flag = byte,
                0xFF04..=0xFF07 => self.timer.write(address, byte),

                0xFF46 => {
                    self.dma.start(byte);
                    self.dma_transfer(byte);
                }

                0xFF40..=0xFF45 | 0xFF47..=0xFF4B => self.ppu.write(address, byte),
                _ => {}
            },
            0xFF80..=0xFFFE => self.hram[(address - 0xFF80) as usize] = byte,
            0xFFFF => self.ie_reg = byte,
            _ => {}
        }
    }

    fn dma_transfer(&mut self, value: u8) {
        let base = (value as u16) << 8;
        for i in 0..160 {
            let byte = self.read_byte(base + i);
            self.ppu.oam[i as usize] = byte;
        }
    }
}
