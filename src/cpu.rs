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
            a: 0x01,
            f: 0xB0, // Z=1, N=0, H=1, C=1
            b: 0x00,
            c: 0x13,
            d: 0x00,
            e: 0xD8,
            h: 0x01,
            l: 0x4D,
            sp: 0xFFFE,
            pc: 0x0100,
        }
    }
    pub fn get_z(&self) -> bool {
        self.f & Z_FLAG != 0
    }
    pub fn get_n(&self) -> bool {
        self.f & N_FLAG != 0
    }
    pub fn get_h(&self) -> bool {
        self.f & H_FLAG != 0
    }
    pub fn get_c(&self) -> bool {
        self.f & C_FLAG != 0
    }
    pub fn set_z(&mut self, value: bool) {
        if value {
            self.f = self.f | Z_FLAG;
        } else {
            self.f = self.f & !Z_FLAG;
        }
    }
    pub fn set_n(&mut self, value: bool) {
        if value {
            self.f = self.f | N_FLAG;
        } else {
            self.f = self.f & !N_FLAG;
        }
    }
    pub fn set_h(&mut self, value: bool) {
        if value {
            self.f = self.f | H_FLAG;
        } else {
            self.f = self.f & !H_FLAG;
        }
    }
    pub fn set_c(&mut self, value: bool) {
        if value {
            self.f = self.f | C_FLAG;
        } else {
            self.f = self.f & !C_FLAG;
        }
    }

    pub fn get_af(&self) -> u16 {
        (self.a as u16) << 8 | (self.f as u16)
    }
    pub fn set_af(&mut self, value: u16) {
        self.a = ((value & 0xFF00) >> 8) as u8;
        //last 4 bits of F-flag are always zero
        self.f = (value & 0x00F0) as u8;
    }
    pub fn get_bc(&self) -> u16 {
        (self.b as u16) << 8 | (self.c as u16)
    }
    pub fn set_bc(&mut self, value: u16) {
        self.b = ((value & 0xFF00) >> 8) as u8;
        self.c = (value & 0x00FF) as u8;
    }

    pub fn get_de(&self) -> u16 {
        (self.d as u16) << 8 | (self.e as u16)
    }
    pub fn set_de(&mut self, value: u16) {
        self.d = ((value & 0xFF00) >> 8) as u8;
        self.e = (value & 0x00FF) as u8;
    }

    pub fn get_hl(&self) -> u16 {
        (self.h as u16) << 8 | (self.l as u16)
    }
    pub fn set_hl(&mut self, value: u16) {
        self.h = ((value & 0xFF00) >> 8) as u8;
        self.l = (value & 0x00FF) as u8;
    }
}
