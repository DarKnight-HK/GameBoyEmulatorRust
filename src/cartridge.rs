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
            0x52 => (1_152 * 1024),
            0x53 => (1_280 * 1024),
            0x54 => (1_536 * 1024),
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
}
