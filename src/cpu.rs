use crate::bus::Bus;
const Z_FLAG: u8 = 0b1000_0000;
const N_FLAG: u8 = 0b0100_0000;
const H_FLAG: u8 = 0b0010_0000;
const C_FLAG: u8 = 0b0001_0000;

pub struct Cpu {
    pub bus: Bus,
    // https://gbdev.io/pandocs/CPU_Registers_and_Flags.html
    pub a: u8,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub h: u8,
    pub l: u8,
    pub sp: u16,
    pub pc: u16,
    pub f: u8,
}

impl Cpu {
    pub fn new(bus: Bus) -> Self {
        Cpu {
            bus,
            a: 0,
            b: 0,
            c: 0,
            d: 0,
            e: 0,
            h: 0,
            l: 0,
            sp: 0xFFFE,
            pc: 0x0100,
            f: 0,
        }
    }
    pub fn get_z_flag(&self) -> bool {
        self.f & Z_FLAG != 0
    }
    pub fn get_n_flag(&self) -> bool {
        self.f & N_FLAG != 0
    }
    pub fn get_h_flag(&self) -> bool {
        self.f & H_FLAG != 0
    }
    pub fn get_c_flag(&self) -> bool {
        self.f & C_FLAG != 0
    }
    pub fn get_bc(&self) -> u16 {
        (self.b as u16) << 8 | (self.c as u16)
    }
    pub fn set_bc(&mut self, value: u16) {
        self.b = ((value & 0xFF00) >> 8) as u8;
        self.c = (value & 0x00FF) as u8;
    }
}
