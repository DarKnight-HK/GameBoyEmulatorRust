// More Info can be found at https://gbdev.io/pandocs/The_Cartridge_Header.html

use std::fs;

#[derive(Debug, Clone, Copy)]
pub enum MbcType {
    RomOnly,
    MBC1,
    MBC2,
    MBC3,
    Unknown(u8),
}

#[derive(Debug)]
pub struct CartridgeHeader {
    pub title: String,
    pub cart_type: MbcType,
    pub ram_size: usize,
    pub rom_size: usize,
    pub check_sum: u8,
}

impl CartridgeHeader {
    pub fn parse(contents: &[u8]) -> Result<Self, String> {
        if contents.len() < 0x150 {
            return Err("ROM is too small".to_string());
        }

        let title_bytes = &contents[0x134..0x143];
        let title = String::from_utf8_lossy(title_bytes)
            .trim_matches('\0')
            .to_string();

        let cart_type_bytes = contents[0x147];
        let cart_type = match cart_type_bytes {
            0x00 => MbcType::RomOnly,
            0x01..=0x03 => MbcType::MBC1,
            0x05..=0x06 => MbcType::MBC2,
            0x0F..=0x13 => MbcType::MBC3,
            _ => MbcType::Unknown(cart_type_bytes),
        };

        let rom_size = match contents[0x148] {
            0x00..=0x08 => 32 * 1024 * (1 << contents[0x148]),
            0x52 => 1_152 * 1024,
            0x53 => 1_280 * 1024,
            0x54 => 1_536 * 1024,
            _ => 32 * 1024,
        };

        let ram_size = match contents[0x149] {
            0x00 => 0,
            0x01 => 2 * 1024,
            0x02 => 8 * 1024,
            0x03 => 32 * 1024,
            0x04 => 128 * 1024,
            0x05 => 64 * 1024,
            _ => 0,
        };

        let checksum = contents[0x14D];

        Ok(Self {
            title,
            cart_type,
            ram_size,
            rom_size,
            check_sum: checksum,
        })
    }
}

#[derive(Debug)]
pub struct Cartridge {
    pub header: CartridgeHeader,
    pub rom_data: Vec<u8>,
    pub ram_data: Vec<u8>,

    // MBC1 State
    ram_enabled: bool,
    rom_bank: u8,
    ram_bank: u8,
    banking_mode: u8, // 0 = ROM Banking Mode, 1 = RAM Banking Mode
}
impl Cartridge {
    pub fn new(file_path: &str) -> Result<Self, String> {
        let rom_data = fs::read(file_path).map_err(|e| format!("Failed to read file: {}", e))?;
        let header = CartridgeHeader::parse(&rom_data)?;
        let ram_size = header.ram_size;

        Ok(Cartridge {
            header,
            rom_data,
            ram_data: vec![0; ram_size],
            // Initialize MBC1 State
            ram_enabled: false,
            rom_bank: 1, // Defaults to 1, not 0
            ram_bank: 0,
            banking_mode: 0,
        })
    }

    pub fn verify_checksum(&self) -> bool {
        let mut checksum: u8 = 0;
        for address in 0x134..=0x14C {
            checksum = checksum
                .wrapping_sub(self.rom_data[address])
                .wrapping_sub(1);
        }
        checksum == self.header.check_sum
    }

    pub fn read(&self, address: u16) -> u8 {
        match self.header.cart_type {
            MbcType::RomOnly => self.read_rom_only(address),
            MbcType::MBC1 => self.read_mbc1(address),
            _ => 0xFF, // Placeholder for MBC2/3
        }
    }

    pub fn write(&mut self, address: u16, byte: u8) {
        match self.header.cart_type {
            MbcType::RomOnly => self.write_rom_only(address, byte),
            MbcType::MBC1 => self.write_mbc1(address, byte),
            _ => {}
        }
    }

    // --- ROM ONLY Logic ---
    fn read_rom_only(&self, address: u16) -> u8 {
        match address {
            0x0000..=0x7FFF => {
                if (address as usize) < self.rom_data.len() {
                    self.rom_data[address as usize]
                } else {
                    0xFF
                }
            }
            0xA000..=0xBFFF => self.read_ram_common(address, 0),
            _ => 0xFF,
        }
    }

    fn write_rom_only(&mut self, address: u16, byte: u8) {
        if let 0xA000..=0xBFFF = address {
            self.write_ram_common(address, byte, 0);
        }
    }

    // --- MBC1 Logic ---
    fn read_mbc1(&self, address: u16) -> u8 {
        match address {
            // ROM Bank 00 (0000-3FFF)
            0x0000..=0x3FFF => {
                let bank = if self.banking_mode == 1 {
                    // Advanced: In RAM banking mode, this can be switched (Multi-cart behavior)
                    (self.ram_bank << 5) as usize
                } else {
                    0
                };
                self.read_rom_banked(address, bank)
            }

            // ROM Bank 01-7F (4000-7FFF)
            0x4000..=0x7FFF => {
                // Combine the 2-bit RAM bank (high) and 5-bit ROM bank (low)
                let bank = ((self.ram_bank << 5) | self.rom_bank) as usize;
                self.read_rom_banked(address - 0x4000, bank)
            }

            // External RAM (A000-BFFF)
            0xA000..=0xBFFF => {
                if !self.ram_enabled {
                    return 0xFF;
                }

                let bank = if self.banking_mode == 1 {
                    self.ram_bank as usize
                } else {
                    0
                };
                self.read_ram_common(address, bank)
            }
            _ => 0xFF,
        }
    }

    fn write_mbc1(&mut self, address: u16, byte: u8) {
        match address {
            // RAM Enable (0000-1FFF)
            0x0000..=0x1FFF => {
                // 0x0A enables RAM, anything else disables it
                self.ram_enabled = (byte & 0x0F) == 0x0A;
            }

            // ROM Bank Number (2000-3FFF)
            0x2000..=0x3FFF => {
                let mut bank = byte & 0x1F; // Lower 5 bits
                if bank == 0 {
                    bank = 1;
                } // "0 is 1" quirk
                self.rom_bank = bank;
            }

            0x4000..=0x5FFF => {
                self.ram_bank = byte & 0x03; // Lower 2 bits
            }

            0x6000..=0x7FFF => {
                self.banking_mode = byte & 0x01;
            }

            0xA000..=0xBFFF => {
                if !self.ram_enabled {
                    return;
                }

                let bank = if self.banking_mode == 1 {
                    self.ram_bank as usize
                } else {
                    0
                };
                self.write_ram_common(address, byte, bank);
            }
            _ => {}
        }
    }

    fn read_rom_banked(&self, offset: u16, bank: usize) -> u8 {
        let rom_address = (bank * 0x4000) + offset as usize;
        let final_address = rom_address & (self.rom_data.len() - 1);
        self.rom_data[final_address]
    }

    fn read_ram_common(&self, address: u16, bank: usize) -> u8 {
        if self.ram_data.is_empty() {
            return 0xFF;
        }

        let ram_address = (bank * 0x2000) + (address - 0xA000) as usize;
        let final_address = ram_address & (self.ram_data.len() - 1);

        if final_address < self.ram_data.len() {
            self.ram_data[final_address]
        } else {
            0xFF
        }
    }

    fn write_ram_common(&mut self, address: u16, byte: u8, bank: usize) {
        if self.ram_data.is_empty() {
            return;
        }

        let ram_address = (bank * 0x2000) + (address - 0xA000) as usize;
        let final_address = ram_address & (self.ram_data.len() - 1);

        if final_address < self.ram_data.len() {
            self.ram_data[final_address] = byte;
        }
    }
}
