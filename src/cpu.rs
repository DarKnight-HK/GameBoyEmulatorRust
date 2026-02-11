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
    f: u8,
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

//Helpers functions here
impl Cpu {
    fn xor_a(&mut self, value: u8) {
        self.a ^= value;
        self.f = 0;
        self.set_z(self.a == 0);
    }
    fn next_u16(&mut self) -> u16 {
        let low = self.bus.read_byte(self.pc) as u16;
        let high = self.bus.read_byte(self.pc + 1) as u16;
        self.pc = self.pc.wrapping_add(2);
        (high << 8 | low)
    }
    fn dec(&mut self, value: u8) -> u8 {
        let result = value.wrapping_sub(1);
        self.set_z(result == 0);
        self.set_n(true);
        self.set_h((value & 0x0F) == 0);

        result
    }

    fn jr(&mut self, condition: bool) -> u8 {
        let offset = self.bus.read_byte(self.pc) as i8;
        self.pc = self.pc.wrapping_add(1);
        if condition {
            self.pc = self.pc.wrapping_add(offset as u16);
            12
        }
        else {
            8
        }

    }
}

// Step function and instructions here
impl Cpu {
    pub fn step(&mut self) -> u8 {
        let opcode = self.bus.read_byte(self.pc);
        self.pc = self.pc.wrapping_add(1);
        match opcode {
            0x00 => 4, //NOP
            0xC3 => {
                let low = self.bus.read_byte(self.pc) as u16;
                let high = self.bus.read_byte(self.pc + 1) as u16;
                let target = (high << 8 | low);
                self.pc = target;
                16
            }
            0xA8 => {
                self.xor_a(self.b);
                4
            }
            0xA9 => {
                self.xor_a(self.c);
                4
            }
            0xAA => {
                self.xor_a(self.d);
                4
            }
            0xAB => {
                self.xor_a(self.e);
                4
            }
            0xAC => {
                self.xor_a(self.h);
                4
            }
            0xAD => {
                self.xor_a(self.l);
                4
            }

            0xAE => {
                let hl = self.get_hl();
                let val = self.bus.read_byte(hl);
                self.xor_a(val);
                8
            }

            0xAF => {
                self.xor_a(self.a);
                4
            }
            0x21 => {
                let val = self.next_u16();
                self.set_hl(val);
                12
            }

            // --- LD r, d8 Group (Load Immediate 8-bit) ---
            // Cycles: 8
            0x06 => {
                let val = self.bus.read_byte(self.pc);
                self.pc = self.pc.wrapping_add(1);
                self.b = val;
                8
            }
            0x0E => {
                let val = self.bus.read_byte(self.pc);
                self.pc = self.pc.wrapping_add(1);
                self.c = val;
                8
            }
            0x16 => {
                let val = self.bus.read_byte(self.pc);
                self.pc = self.pc.wrapping_add(1);
                self.d = val;
                8
            }
            0x1E => {
                let val = self.bus.read_byte(self.pc);
                self.pc = self.pc.wrapping_add(1);
                self.e = val;
                8
            }
            0x26 => {
                let val = self.bus.read_byte(self.pc);
                self.pc = self.pc.wrapping_add(1);
                self.h = val;
                8
            }
            0x2E => {
                let val = self.bus.read_byte(self.pc);
                self.pc = self.pc.wrapping_add(1);
                self.l = val;
                8
            }
            // Special Case: LD (HL), d8
            // Writes the immediate value to the memory address at HL
            // Cycles: 12
            0x36 => {
                let val = self.bus.read_byte(self.pc);
                self.pc = self.pc.wrapping_add(1);
                let hl = self.get_hl();
                self.bus.write_byte(hl, val);
                12
            }
            0x32 => {
                let mut hl = self.get_hl();
                self.bus.write_byte(hl, self.a);
                self.set_hl(hl.wrapping_sub(1));
                8
            }
            // --- DEC r8 Family (Decrement 8-bit) ---
            // Cycles: 4
            0x05 => {
                self.b = self.dec(self.b);
                4
            }
            0x0D => {
                self.c = self.dec(self.c);
                4
            }
            0x15 => {
                self.d = self.dec(self.d);
                4
            }
            0x1D => {
                self.e = self.dec(self.e);
                4
            }
            0x25 => {
                self.h = self.dec(self.h);
                4
            }
            0x2D => {
                self.l = self.dec(self.l);
                4
            }
            0x3D => {
                self.a = self.dec(self.a);
                4
            }

            // Special Case: DEC (HL)
            // Read from memory, decrement, write back.
            // Cycles: 12 (4 for opcode + 4 for read + 4 for write)
            0x35 => {
                let hl = self.get_hl();
                let val = self.bus.read_byte(hl);
                let result = self.dec(val);
                self.bus.write_byte(hl, result);
                12
            },
            // --- JR Family (Jump Relative) ---
            
            // 0x18: JR r8 (Unconditional Jump Relative)
            // Always jumps.
            0x18 => {
                self.jr(true); // Always true
                12 // Unconditional JR is always 12 cycles
            }

            // 0x20: JR NZ, r8 (Jump if Not Zero)
            0x20 => {
                let check = !self.get_z();
                self.jr(check)
            }

            // 0x28: JR Z, r8 (Jump if Zero)
            0x28 => {
                let check = self.get_z();
                self.jr(check)
            }

            // 0x30: JR NC, r8 (Jump if Not Carry)
            0x30 => {
                let check = !self.get_c();
                self.jr(check)
            }

            // 0x38: JR C, r8 (Jump if Carry)
            0x38 => {
                let check = self.get_c();
                self.jr(check)
            }
            _ => {
                println!("Unknown Opcode: {:#02X} at {:#04X}", opcode, self.pc - 1);
                0
            }
        }
    }
}
