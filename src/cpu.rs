use crate::bus::Bus;
use crate::interrupts::Interrupt;
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
    pub ime: bool,
    pub is_sleeping: bool,
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
            ime: false,
            is_sleeping: false,
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

    pub fn check_interrupts(&mut self) {
        let int_flag = self.bus.int_flag;
        let ie_reg = self.bus.ie_reg;
        for interrupt in Interrupt::iterate() {
            let mask = interrupt.mask();
            if (int_flag & mask) != 0 && (ie_reg & mask) != 0 {
                self.is_sleeping = false;
                if self.ime {
                    self.handle_interrupt(interrupt);
                }
                return;
            }
        }
    }
    pub fn handle_interrupt(&mut self, interrupt: Interrupt) {
        self.ime = false;
        self.bus.int_flag &= !interrupt.mask();
        self.push_stack(self.pc);
        self.pc = interrupt.handler_address();
    }
}

//Helpers functions here
impl Cpu {
    fn xor_a(&mut self, value: u8) {
        self.a ^= value;
        self.f = 0;
        self.set_z(self.a == 0);
    }
    fn next_u8(&mut self) -> u8 {
        let val = self.bus.read_byte(self.pc);
        self.pc = self.pc.wrapping_add(1);
        val
    }
    fn next_u16(&mut self) -> u16 {
        let low = self.bus.read_byte(self.pc) as u16;
        let high = self.bus.read_byte(self.pc + 1) as u16;
        self.pc = self.pc.wrapping_add(2);
        high << 8 | low
    }
    fn dec(&mut self, value: u8) -> u8 {
        let result = value.wrapping_sub(1);
        self.set_z(result == 0);
        self.set_n(true);
        self.set_h((value & 0x0F) == 0);

        result
    }
    fn inc(&mut self, value: u8) -> u8 {
        let result = value.wrapping_add(1);
        self.set_z(result == 0);
        self.set_n(false);
        self.set_h((value & 0x0F) == 0x0F);
        result
    }

    fn jr(&mut self, condition: bool) -> u8 {
        let offset = self.bus.read_byte(self.pc) as i8;
        self.pc = self.pc.wrapping_add(1);
        if condition {
            self.pc = self.pc.wrapping_add(offset as u16);
            12
        } else {
            8
        }
    }

    fn cp(&mut self, n: u8) {
        let (result, carry) = self.a.overflowing_sub(n);
        self.set_z(result == 0);
        self.set_n(true);
        self.set_h((self.a & 0x0F) < (n & 0x0F));
        self.set_c(carry);
    }
    fn or(&mut self, value: u8) {
        self.a |= value;

        self.set_z(self.a == 0);
        self.set_n(false);
        self.set_h(false);
        self.set_c(false);
    }
    fn push_stack(&mut self, value: u16) {
        self.sp = self.sp.wrapping_sub(1);
        self.bus.write_byte(self.sp, ((value & 0xFF00) >> 8) as u8);
        self.sp = self.sp.wrapping_sub(1);
        self.bus.write_byte(self.sp, ((value & 0x00FF) >> 8) as u8);
    }

    fn pop_stack(&mut self) -> u16 {
        let low = self.bus.read_byte(self.sp) as u16;
        self.sp = self.sp.wrapping_add(1);
        let high = self.bus.read_byte(self.sp) as u16;
        self.sp = self.sp.wrapping_add(1);
        (high << 8) | low
    }
    fn rst(&mut self, address: u16) {
        self.push_stack(self.pc);
        self.pc = address;
    }
}

//Helpers for CB instructions
impl Cpu {
    fn get_cb_reg(&mut self, index: u8) -> u8 {
        match index {
            0 => self.b,
            1 => self.c,
            2 => self.d,
            3 => self.e,
            4 => self.h,
            5 => self.l,
            6 => self.bus.read_byte(self.get_hl()),
            7 => self.a,
            _ => unreachable!(),
        }
    }

    fn set_cb_reg(&mut self, index: u8, val: u8) {
        match index {
            0 => self.b = val,
            1 => self.c = val,
            2 => self.d = val,
            3 => self.e = val,
            4 => self.h = val,
            5 => self.l = val,
            6 => self.bus.write_byte(self.get_hl(), val),
            7 => self.a = val,
            _ => unreachable!(),
        }
    }

    fn rlc(&mut self, value: u8) -> u8 {
        let carry = (value & 0x80) != 0;
        let result = (value << 1) | (if carry { 1 } else { 0 });
        self.set_z(result == 0);
        self.set_n(false);
        self.set_h(false);
        self.set_c(carry);
        result
    }
    fn rrc(&mut self, value: u8) -> u8 {
        let carry = (value & 0x01) != 0;
        let result = (value >> 1) | (if carry { 0x80 } else { 0 });
        self.set_z(result == 0);
        self.set_n(false);
        self.set_h(false);
        self.set_c(carry);
        result
    }

    fn rl(&mut self, value: u8) -> u8 {
        let new_carry = (value & 0x80) != 0;
        let result = (value << 1) | (if self.get_c() { 1 } else { 0 });
        self.set_z(result == 0);
        self.set_n(false);
        self.set_h(false);
        self.set_c(new_carry);
        result
    }
    fn rr(&mut self, value: u8) -> u8 {
        let new_carry = (value & 0x01) != 0;
        let result = (value >> 1) | (if self.get_c() { 0x80 } else { 0 });
        self.set_z(result == 0);
        self.set_n(false);
        self.set_h(false);
        self.set_c(new_carry);
        result
    }
    fn sla(&mut self, value: u8) -> u8 {
        let carry = (value & 0x80) != 0;
        let result = value << 1;
        self.set_z(result == 0);
        self.set_n(false);
        self.set_h(false);
        self.set_c(carry);
        result
    }
    fn sra(&mut self, value: u8) -> u8 {
        let carry = (value & 0x01) != 0;
        let result = (value >> 1) | (value & 0x80);
        self.set_z(result == 0);
        self.set_n(false);
        self.set_h(false);
        self.set_c(carry);
        result
    }
    fn swap(&mut self, value: u8) -> u8 {
        let result = ((value & 0x0F) << 4) | ((value & 0xF0) >> 4);
        self.set_z(result == 0);
        self.set_n(false);
        self.set_h(false);
        self.set_c(false);
        result
    }
    fn srl(&mut self, value: u8) -> u8 {
        let carry = (value & 0x01) != 0;
        let result = value >> 1;
        self.set_z(result == 0);
        self.set_n(false);
        self.set_h(false);
        self.set_c(carry);
        result
    }
}

// Step function and instructions here
// https://gbdev.io/gb-opcodes/optables/
// https://rgbds.gbdev.io/docs/v1.0.1/gbz80.7
impl Cpu {
    pub fn step(&mut self) -> u8 {
        if self.is_sleeping {
            return 4;
        }
        let opcode = self.bus.read_byte(self.pc);
        self.pc = self.pc.wrapping_add(1);
        match opcode {
            0x00 => 4, //NOP
            0xC3 => {
                let low = self.bus.read_byte(self.pc) as u16;
                let high = self.bus.read_byte(self.pc + 1) as u16;
                let target = high << 8 | low;
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
            0x31 => {
                self.sp = self.next_u16();
                12
            }

            0x32 => {
                let hl = self.get_hl();
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
            0x35 => {
                let hl = self.get_hl();
                let val = self.bus.read_byte(hl);
                let result = self.dec(val);
                self.bus.write_byte(hl, result);
                12
            }
            // --- JR Family (Jump Relative) ---
            // 0x18: JR r8 (Unconditional Jump Relative)
            // Always jumps.
            0x18 => {
                self.jr(true);
                12
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

            // --- LD r, d8 Group (Load Immediate 8-bit) ---
            0x06 => {
                self.b = self.next_u8();
                8
            }
            0x0E => {
                self.c = self.next_u8();
                8
            }
            0x16 => {
                self.d = self.next_u8();
                8
            }
            0x1E => {
                self.e = self.next_u8();
                8
            }
            0x26 => {
                self.h = self.next_u8();
                8
            }
            0x2E => {
                self.l = self.next_u8();
                8
            }
            0x3E => {
                self.a = self.next_u8();
                8
            }

            0x36 => {
                let val = self.next_u8();
                let hl = self.get_hl();
                self.bus.write_byte(hl, val);
                12
            }
            // 0xF3: DI (Disable Interrupts)
            0xF3 => {
                self.ime = false;
                4
            }

            // 0xFB: EI (Enable Interrupts)
            0xFB => {
                self.ime = true;
                4
            }
            0xE0 => {
                let offset = self.next_u8() as u16;
                let address = 0xFF00 | offset;
                self.bus.write_byte(address, self.a);
                12
            }
            // 0xE2: LD (C), A
            // Write A to (0xFF00 + C)
            0xE2 => {
                let address = 0xFF00 | (self.c as u16);
                self.bus.write_byte(address, self.a);
                8
            }

            // 0xF2: LD A, (C)
            // Read from (0xFF00 + C) into A
            0xF2 => {
                let address = 0xFF00 | (self.c as u16);
                self.a = self.bus.read_byte(address);
                8
            }
            0xF0 => {
                let address = 0xFF00 | self.next_u8() as u16;
                self.a = self.bus.read_byte(address);
                12
            }
            // --- CP r8 Family (Compare A with r8) ---
            0xB8 => {
                self.cp(self.b);
                4
            }
            0xB9 => {
                self.cp(self.c);
                4
            }
            0xBA => {
                self.cp(self.d);
                4
            }
            0xBB => {
                self.cp(self.e);
                4
            }
            0xBC => {
                self.cp(self.h);
                4
            }
            0xBD => {
                self.cp(self.l);
                4
            }
            0xBF => {
                self.cp(self.a);
                4
            }

            // Special Case: CP (HL)
            // Compare A with value in memory at HL
            0xBE => {
                let hl = self.get_hl();
                let val = self.bus.read_byte(hl);
                self.cp(val);
                8
            }

            // Special Case: CP d8 (Immediate)
            // Compare A with the next byte in code
            0xFE => {
                let val = self.next_u8();
                self.cp(val);
                8
            }
            0xEA => {
                let address = self.next_u16();
                self.bus.write_byte(address, self.a);
                16
            }
            0xFA => {
                let address = self.next_u16();
                self.a = self.bus.read_byte(address);
                16
            }
            0x2A => {
                let hl = self.get_hl();
                self.a = self.bus.read_byte(hl);
                self.set_hl(hl.wrapping_add(1));
                8
            }
            0xCD => {
                let target = self.next_u16();
                self.push_stack(self.pc);
                self.pc = target;
                24
            }

            0x01 => {
                let val = self.next_u16();
                self.set_bc(val);
                12
            }

            0x11 => {
                let val = self.next_u16();
                self.set_de(val);
                12
            }
            // --- DEC rr Family (Decrement 16-bit) ---
            // Flags: None affected
            0x0B => {
                let val = self.get_bc().wrapping_sub(1);
                self.set_bc(val);
                8
            }
            0x1B => {
                let val = self.get_de().wrapping_sub(1);
                self.set_de(val);
                8
            }
            0x2B => {
                let val = self.get_hl().wrapping_sub(1);
                self.set_hl(val);
                8
            }
            0x3B => {
                self.sp = self.sp.wrapping_sub(1);
                8
            }
            // --- LD A, r8 Group (Load into A) ---
            0x78 => {
                self.a = self.b;
                4
            }

            0x79 => {
                self.a = self.c;
                4
            }

            0x7A => {
                self.a = self.d;
                4
            }

            0x7B => {
                self.a = self.e;
                4
            }

            0x7C => {
                self.a = self.h;
                4
            }
            0x7D => {
                self.a = self.l;
                4
            }
            0x7E => {
                let hl = self.get_hl();
                self.a = self.bus.read_byte(hl);
                8
            }
            0x7F => 4,
            // --- OR r8 Family ---
            0xB0 => {
                self.or(self.b);
                4
            }
            0xB1 => {
                self.or(self.c);
                4
            }
            0xB2 => {
                self.or(self.d);
                4
            }
            0xB3 => {
                self.or(self.e);
                4
            }
            0xB4 => {
                self.or(self.h);
                4
            }
            0xB5 => {
                self.or(self.l);
                4
            }
            0xB7 => {
                self.or(self.a);
                4
            }
            0xB6 => {
                let hl = self.get_hl();
                let val = self.bus.read_byte(hl);
                self.or(val);
                8
            }
            0xF6 => {
                let val = self.next_u8();
                self.or(val);
                8
            }
            // --- RET Family (Return from Subroutine) ---
            0xC9 => {
                self.pc = self.pop_stack();
                16
            }

            0xC0 => {
                if !self.get_z() {
                    self.pc = self.pop_stack();
                    20
                } else {
                    8
                }
            }

            0xC8 => {
                if self.get_z() {
                    self.pc = self.pop_stack();
                    20
                } else {
                    8
                }
            }

            0xD0 => {
                if !self.get_c() {
                    self.pc = self.pop_stack();
                    20
                } else {
                    8
                }
            }

            0xD8 => {
                if self.get_c() {
                    self.pc = self.pop_stack();
                    20
                } else {
                    8
                }
            }

            0xD9 => {
                self.pc = self.pop_stack();
                self.ime = true;
                16
            }
            // --- PUSH rr Family ---
            0xC5 => {
                let val = self.get_bc();
                self.push_stack(val);
                16
            }
            0xD5 => {
                let val = self.get_de();
                self.push_stack(val);
                16
            }
            0xE5 => {
                let val = self.get_hl();
                self.push_stack(val);
                16
            }
            0xF5 => {
                let val = self.get_af();
                self.push_stack(val);
                16
            }
            // --- POP rr Family ---
            0xC1 => {
                let val = self.pop_stack();
                self.set_bc(val);
                12
            }
            0xD1 => {
                let val = self.pop_stack();
                self.set_de(val);
                12
            }
            0xE1 => {
                let val = self.pop_stack();
                self.set_hl(val);
                12
            }
            0xF1 => {
                let val = self.pop_stack();
                self.set_af(val);
                12
            }

            // --- INC r8 Family ---
            0x04 => {
                self.b = self.inc(self.b);
                4
            }
            0x0C => {
                self.c = self.inc(self.c);
                4
            }
            0x14 => {
                self.d = self.inc(self.d);
                4
            }
            0x1C => {
                self.e = self.inc(self.e);
                4
            }
            0x24 => {
                self.h = self.inc(self.h);
                4
            }
            0x2C => {
                self.l = self.inc(self.l);
                4
            }
            0x3C => {
                self.a = self.inc(self.a);
                4
            }
            0x34 => {
                let hl = self.get_hl();
                let val = self.bus.read_byte(hl);
                let result = self.inc(val);
                self.bus.write_byte(hl, result);
                12
            }

            // --- LD r8, r8 Family ---

            // Destination B (0x40 - 0x47)
            0x40 => 4,
            0x41 => {
                self.b = self.c;
                4
            }
            0x42 => {
                self.b = self.d;
                4
            }
            0x43 => {
                self.b = self.e;
                4
            }
            0x44 => {
                self.b = self.h;
                4
            }
            0x45 => {
                self.b = self.l;
                4
            }
            0x46 => {
                self.b = self.bus.read_byte(self.get_hl());
                8
            }
            0x47 => {
                self.b = self.a;
                4
            }

            // Destination C (0x48 - 0x4F)
            0x48 => {
                self.c = self.b;
                4
            }
            0x49 => 4,
            0x4A => {
                self.c = self.d;
                4
            }
            0x4B => {
                self.c = self.e;
                4
            }
            0x4C => {
                self.c = self.h;
                4
            }
            0x4D => {
                self.c = self.l;
                4
            }
            0x4E => {
                self.c = self.bus.read_byte(self.get_hl());
                8
            }
            0x4F => {
                self.c = self.a;
                4
            }

            // Destination D (0x50 - 0x57)
            0x50 => {
                self.d = self.b;
                4
            }
            0x51 => {
                self.d = self.c;
                4
            }
            0x52 => 4,
            0x53 => {
                self.d = self.e;
                4
            }
            0x54 => {
                self.d = self.h;
                4
            }
            0x55 => {
                self.d = self.l;
                4
            }
            0x56 => {
                self.d = self.bus.read_byte(self.get_hl());
                8
            }
            0x57 => {
                self.d = self.a;
                4
            }

            // Destination E (0x58 - 0x5F)
            0x58 => {
                self.e = self.b;
                4
            }
            0x59 => {
                self.e = self.c;
                4
            }
            0x5A => {
                self.e = self.d;
                4
            }
            0x5B => 4,
            0x5C => {
                self.e = self.h;
                4
            }
            0x5D => {
                self.e = self.l;
                4
            }
            0x5E => {
                self.e = self.bus.read_byte(self.get_hl());
                8
            }
            0x5F => {
                self.e = self.a;
                4
            }

            // Destination H (0x60 - 0x67)
            0x60 => {
                self.h = self.b;
                4
            }
            0x61 => {
                self.h = self.c;
                4
            }
            0x62 => {
                self.h = self.d;
                4
            }
            0x63 => {
                self.h = self.e;
                4
            }
            0x64 => 4,
            0x65 => {
                self.h = self.l;
                4
            }
            0x66 => {
                self.h = self.bus.read_byte(self.get_hl());
                8
            }
            0x67 => {
                self.h = self.a;
                4
            }

            // Destination L (0x68 - 0x6F)
            0x68 => {
                self.l = self.b;
                4
            }
            0x69 => 4,
            0x6A => {
                self.l = self.d;
                4
            }
            0x6B => {
                self.l = self.e;
                4
            }
            0x6C => {
                self.l = self.h;
                4
            }
            0x6D => 4,
            0x6E => {
                self.l = self.bus.read_byte(self.get_hl());
                8
            }
            0x6F => {
                self.l = self.a;
                4
            }

            // Destination (HL) (0x70 - 0x77)
            0x70 => {
                self.bus.write_byte(self.get_hl(), self.b);
                8
            }
            0x71 => {
                self.bus.write_byte(self.get_hl(), self.c);
                8
            }
            0x72 => {
                self.bus.write_byte(self.get_hl(), self.d);
                8
            }
            0x73 => {
                self.bus.write_byte(self.get_hl(), self.e);
                8
            }
            0x74 => {
                self.bus.write_byte(self.get_hl(), self.h);
                8
            }
            0x75 => {
                self.bus.write_byte(self.get_hl(), self.l);
                8
            }

            0x77 => {
                self.bus.write_byte(self.get_hl(), self.a);
                8
            }

            // --- RST (Restart) Family ---
            0xC7 => {
                self.rst(0x0000);
                16
            }
            0xCF => {
                self.rst(0x0008);
                16
            }
            0xD7 => {
                self.rst(0x0010);
                16
            }
            0xDF => {
                self.rst(0x0018);
                16
            }
            0xE7 => {
                self.rst(0x0020);
                16
            }
            0xEF => {
                self.rst(0x0028);
                16
            }
            0xF7 => {
                self.rst(0x0030);
                16
            }
            0xFF => {
                self.rst(0x0038);
                16
            }
            0x76 => {
                self.is_sleeping = true;
                4
            }
            0xCB => self.step_cb(),
            _ => {
                println!("Unknown Opcode: {:#02X} at {:#04X}", opcode, self.pc - 1);
                0
            }
        }
    }
}

// CB instructions

impl Cpu {
    pub fn step_cb(&mut self) -> u8 {
        let opcode = self.next_u8();
        let reg_idx = opcode & 0x07;
        let bit_idx = (opcode >> 3) & 0x07;
        let group = (opcode >> 6) & 0x03;
        let cycles = if reg_idx == 6 {
            if group == 1 { 12 } else { 16 }
        } else {
            8
        };

        match group {
            // Group 0: Rotates and Shifts
            0 => {
                let val = self.get_cb_reg(reg_idx);
                let result = match bit_idx {
                    0 => self.rlc(val),  // RLC
                    1 => self.rrc(val),  // RRC
                    2 => self.rl(val),   // RL
                    3 => self.rr(val),   // RR
                    4 => self.sla(val),  // SLA
                    5 => self.sra(val),  // SRA
                    6 => self.swap(val), // SWAP
                    7 => self.srl(val),  // SRL
                    _ => unreachable!(),
                };
                self.set_cb_reg(reg_idx, result);
            }

            // Group 1: BIT (Test Bit)
            1 => {
                let val = self.get_cb_reg(reg_idx);
                let is_zero = (val & (1 << bit_idx)) == 0;
                self.set_z(is_zero);
                self.set_n(false);
                self.set_h(true); // BIT always sets H=1
            }

            // Group 2: RES (Reset Bit)
            2 => {
                let val = self.get_cb_reg(reg_idx);
                let result = val & !(1 << bit_idx);
                self.set_cb_reg(reg_idx, result);
            }

            // Group 3: SET (Set Bit)
            3 => {
                let val = self.get_cb_reg(reg_idx);
                let result = val | (1 << bit_idx);
                self.set_cb_reg(reg_idx, result);
            }

            _ => unreachable!(),
        }
        cycles
    }
}
