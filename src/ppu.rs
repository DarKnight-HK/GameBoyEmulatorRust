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

    pub fn tick(&mut self, cycles: u8) -> (bool, bool) {
        self.cycle_accumulator += cycles as u32;
        if self.cycle_accumulator >= 456 {
            self.cycle_accumulator -= 456;
            self.ly = (self.ly + 1) % 154;
            if self.ly == 144 {
                return (true, false);
            }
        }
        (false, false)
    }
}
