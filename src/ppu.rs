#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PpuMode {
    HBlank = 0,
    VBlank = 1,
    OamSearch = 2,
    PixelTransfer = 3,
}

pub struct Ppu {
    pub vram: [u8; 8192],
    pub oam: [u8; 160],
    // Format: 0x00RRGGBB
    pub buffer: Vec<u32>,

    pub lcdc: u8, // 0xFF40: LCD Control
    pub stat: u8, // 0xFF41: LCD Status
    pub scy: u8,  // 0xFF42: Scroll Y
    pub scx: u8,  // 0xFF43: Scroll X
    pub ly: u8,   // 0xFF44: LCD Y Coordinate (Current Line)
    pub lyc: u8,  // 0xFF45: LY Compare
    pub bgp: u8,  // 0xFF47: BG Palette
    pub obp0: u8, // 0xFF48: Object Palette 0
    pub obp1: u8, // 0xFF49: Object Palette 1
    pub wy: u8,   // 0xFF4A: Window Y
    pub wx: u8,   // 0xFF4B: Window X
    pub mode: PpuMode,
    pub cycle_accumulator: u32,
}

impl Ppu {
    pub fn new() -> Self {
        Ppu {
            vram: [0; 8192],
            oam: [0; 160],
            buffer: vec![0; 160 * 144],

            lcdc: 0x91,
            stat: 0,
            scy: 0,
            scx: 0,
            ly: 0,
            lyc: 0,
            bgp: 0xFC,
            obp0: 0xFF,
            obp1: 0xFF,
            wy: 0,
            wx: 0,

            mode: PpuMode::OamSearch,
            cycle_accumulator: 0,
        }
    }

    pub fn read(&self, address: u16) -> u8 {
        match address {
            0x8000..=0x9FFF => self.vram[(address - 0x8000) as usize],
            0xFE00..=0xFE9F => self.oam[(address - 0xFE00) as usize],

            0xFF40 => self.lcdc,
            0xFF41 => self.stat,
            0xFF42 => self.scy,
            0xFF43 => self.scx,
            0xFF44 => self.ly,
            0xFF45 => self.lyc,
            0xFF47 => self.bgp,
            0xFF48 => self.obp0,
            0xFF49 => self.obp1,
            0xFF4A => self.wy,
            0xFF4B => self.wx,
            _ => 0xFF,
        }
    }

    pub fn write(&mut self, address: u16, value: u8) {
        match address {
            0x8000..=0x9FFF => self.vram[(address - 0x8000) as usize] = value,
            0xFE00..=0xFE9F => self.oam[(address - 0xFE00) as usize] = value,
            0xFF40 => self.lcdc = value,
            0xFF41 => self.stat = (self.stat & 0xFC) | (value & 0xF8),
            0xFF42 => self.scy = value,
            0xFF43 => self.scx = value,
            0xFF44 => {}
            0xFF45 => self.lyc = value,
            0xFF47 => self.bgp = value,
            0xFF48 => self.obp0 = value,
            0xFF49 => self.obp1 = value,
            0xFF4A => self.wy = value,
            0xFF4B => self.wx = value,
            _ => {}
        }
    }

    // Returns (VBlank Interrupt, Stat Interrupt)
    pub fn tick(&mut self, cycles: u8) -> (bool, bool) {
        let mut vblank_irq = false;
        let mut stat_irq = false;

        self.cycle_accumulator += cycles as u32;

        if self.ly >= 144 {
            self.mode = PpuMode::VBlank;
            if self.cycle_accumulator >= 456 {
                self.cycle_accumulator -= 456;
                self.ly += 1;

                if self.ly >= 154 {
                    self.ly = 0;
                    self.mode = PpuMode::OamSearch;
                }
            }
            return (
                self.ly == 144 && self.cycle_accumulator < cycles as u32,
                false,
            );
        }

        // NORMAL LINE Handling (Lines 0-143)
        if self.cycle_accumulator < 80 {
            // Mode 2: OAM Scan
            self.mode = PpuMode::OamSearch;
        } else if self.cycle_accumulator < 252 {
            // Mode 3: Pixel Transfer
            self.mode = PpuMode::PixelTransfer;
        } else if self.cycle_accumulator < 456 {
            // Mode 0: H-Blank
            if self.mode != PpuMode::HBlank {
                self.mode = PpuMode::HBlank;
                self.draw_scanline();
            }
        } else {
            self.cycle_accumulator -= 456;
            self.ly += 1;

            if self.ly == 144 {
                self.mode = PpuMode::VBlank;
                vblank_irq = true;
            } else {
                self.mode = PpuMode::OamSearch;
            }
        }

        (vblank_irq, stat_irq)
    }
    pub fn draw_scanline(&mut self) {
        let y = self.ly as usize;
        for x in 0..160 {
            self.buffer[y * 160 + x] = 0x00FFFFFF; // White
        }
    }
    pub fn is_lcd_enabled(&self) -> bool {
        (self.lcdc & 0x80) != 0
    }

    pub fn window_tile_map_area(&self) -> u16 {
        if (self.lcdc & 0x40) != 0 {
            0x9C00
        } else {
            0x9800
        }
    }

    pub fn is_window_enabled(&self) -> bool {
        (self.lcdc & 0x20) != 0
    }

    pub fn tile_data_area(&self) -> u16 {
        if (self.lcdc & 0x10) != 0 {
            0x8000
        } else {
            0x8800
        }
    }

    pub fn bg_tile_map_area(&self) -> u16 {
        if (self.lcdc & 0x08) != 0 {
            0x9C00
        } else {
            0x9800
        }
    }

    pub fn obj_size(&self) -> bool {
        (self.lcdc & 0x04) != 0
    }

    pub fn obj_enabled(&self) -> bool {
        (self.lcdc & 0x02) != 0
    }

    pub fn bg_window_enabled(&self) -> bool {
        (self.lcdc & 0x01) != 0
    }
}
