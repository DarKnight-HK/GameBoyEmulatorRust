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
    pub stat_line: bool,
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
            stat_line: false,
        }
    }

    pub fn read(&self, address: u16) -> u8 {
        match address {
            0x8000..=0x9FFF => self.vram[(address - 0x8000) as usize],
            0xFE00..=0xFE9F => self.oam[(address - 0xFE00) as usize],

            0xFF40 => self.lcdc,
            0xFF41 => {
                let mut stat = self.stat & 0xF8;
                if self.ly == self.lyc {
                    stat |= 0x04;
                }
                stat |= self.mode as u8;
                stat | 0x80
            }
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
            0xFF40 => {
                self.lcdc = value;
                if (value & 0x80) == 0 {
                    self.ly = 0;
                    self.mode = PpuMode::HBlank;
                    self.cycle_accumulator = 0;
                }
            }
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

        if !self.is_lcd_enabled() {
            return (false, false);
        }

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
        } else {
            if self.cycle_accumulator < 80 {
                self.mode = PpuMode::OamSearch;
            } else if self.cycle_accumulator < 252 {
                self.mode = PpuMode::PixelTransfer;
            } else if self.cycle_accumulator < 456 {
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
        }

        let stat_signal = (self.ly == self.lyc && (self.stat & 0x40) != 0)
            || (self.mode == PpuMode::OamSearch && (self.stat & 0x20) != 0)
            || (self.mode == PpuMode::VBlank && (self.stat & 0x10) != 0)
            || (self.mode == PpuMode::HBlank && (self.stat & 0x08) != 0);

        if stat_signal && !self.stat_line {
            stat_irq = true;
        }
        self.stat_line = stat_signal;

        (vblank_irq, stat_irq)
    }
    //https://gbdev.io/pandocs/pixel_fifo.html
    pub fn draw_scanline(&mut self) {
        if !self.is_lcd_enabled() {
            let offset = self.ly as usize * 160;
            for x in 0..160 {
                self.buffer[offset + x] = 0xFFFFFFFF;
            }
            return;
        }
        let bg_y = self.scy.wrapping_add(self.ly);
        let base_map_area = self.bg_tile_map_area();
        let tile_row = (bg_y / 8) as u16;
        let internal_y = (bg_y % 8) as u16;
        let canvas_y = self.ly as usize;
        for x in 0..160 {
            let bg_x = self.scx.wrapping_add(x);
            let tile_col = (bg_x / 8) as u16;
            let internal_x = 7 - (bg_x % 8);
            let map_address = base_map_area + (tile_row * 32) + tile_col;
            let tile_index = self.read(map_address);
            let tile_data_address = match self.tile_data_area() {
                0x8000 => 0x8000 + (tile_index as u16 * 16),
                0x8800 => 0x9000 + (tile_index as i8 as i16 * 16) as u16,
                _ => {
                    unreachable!()
                }
            };
            let address = tile_data_address + (internal_y * 2);
            let byte1 = self.read(address);
            let byte2 = self.read(address + 1);
            let bit_low = (byte1 >> internal_x) & 1;
            let bit_high = (byte2 >> internal_x) & 1;
            let color_id = (bit_high << 1) | bit_low;
            let palette_color = self.get_color(color_id, self.bgp);
            self.buffer[canvas_y * 160 + x as usize] = palette_color;
        }
        self.draw_sprites();
    }

    fn get_color(&self, color_id: u8, palette: u8) -> u32 {
        let shade = (palette >> (color_id * 2)) & 0x03;
        match shade {
            0 => 0xFFE0F8D0,
            1 => 0xFF88C070,
            2 => 0xFF346856,
            3 => 0xFF081820,
            _ => 0xFFE0F8D0,
        }
    }
    fn draw_sprites(&mut self) {
        if !self.obj_enabled() {
            return;
        }
        let sprite_height = if self.obj_size() { 16 } else { 8 };
        let line = self.ly as i32;
        for i in 0..40 {
            let offset = i * 4;
            let sprite_y = self.oam[offset] as i32 - 16;
            let sprite_x = self.oam[offset + 1] as i32 - 8;
            let mut tile_index = self.oam[offset + 2];
            let flags = self.oam[offset + 3];
            // Bit mask to extract each flag
            let priority_below_bg = (flags & 0x80) != 0;
            let y_flip = (flags & 0x40) != 0;
            let x_flip = (flags & 0x20) != 0;
            let pallete = (flags & 0x10) != 0;
            if line >= sprite_y && line < (sprite_y + sprite_height) {
                let mut row_to_draw = line - sprite_y;
                if y_flip {
                    row_to_draw = sprite_height - 1 - row_to_draw;
                }
                if sprite_height == 16 {
                    tile_index &= 0xFE;
                    if row_to_draw >= 8 {
                        tile_index += 1;
                        row_to_draw -= 8;
                    }
                }
                let tile_address = 0x8000 + (tile_index as u16 * 16);
                let row_address = tile_address + (row_to_draw as u16 * 2);
                let byte1 = self.read(row_address);
                let byte2 = self.read(row_address + 1);
                for x in 0..8 {
                    let pixel_x = sprite_x + x;
                    if pixel_x >= 0 && pixel_x < 160 {
                        let bit_index = if x_flip { x } else { 7 - x };
                        let bit_low = (byte1 >> bit_index) & 1;
                        let bit_high = (byte2 >> bit_index) & 1;
                        let color_id = (bit_high << 1) | bit_low;
                        if color_id == 0 {
                            continue;
                        }
                        let buffer_idx = (self.ly as usize * 160) + pixel_x as usize;
                        let current_bg_pixel = self.buffer[buffer_idx];
                        // if BG priority is set and BG is not white, sprite is hidden by the non-white BG pixel
                        if priority_below_bg && current_bg_pixel != 0xFFFFFFFF {
                            continue;
                        }
                        let palette = if pallete { self.obp1 } else { self.obp0 };
                        let color = self.get_color(color_id, palette);
                        self.buffer[buffer_idx] = color;
                    }
                }
            }
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
