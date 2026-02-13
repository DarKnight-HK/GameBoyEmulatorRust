// https://gbdev.io/pandocs/Timer_and_Divider_Registers.html
// https://github.com/Ashiepaws/GBEDG/blob/master/timers/index.md
// https://gbdev.io/pandocs/Timer_Obscure_Behaviour.html
pub struct Timer {
    div: u16,
    tima: u8,
    tma: u8,
    tac: u8,
}

impl Timer {
    pub fn new() -> Self {
        Timer {
            div: 0,
            tima: 0,
            tma: 0,
            tac: 0,
        }
    }
    // returns true if an interrupt needs to be requested
    pub fn tick(&mut self, cycles: u8) -> bool {
        let old_div = self.div;
        self.div = self.div.wrapping_add(cycles as u16);
        let clock_select = self.tac & 0x03;
        let timer_enabled = (self.tac & 0x04) != 0;
        // read timer's obscure behavior on pandocs

        let bit_pos: u8 = match clock_select {
            0 => 9,
            1 => 3,
            2 => 5,
            3 => 7,
            _ => unreachable!(),
        };
        let old_bit = (old_div >> bit_pos) & 1;
        let new_bit = (self.div >> bit_pos) & 1;

        if timer_enabled && old_bit == 1 && new_bit == 0 {
            return self.increment_tima();
        }

        false // No interrupt
    }

    fn increment_tima(&mut self) -> bool {
        let (result, overflow) = self.tima.overflowing_add(1);
        self.tima = result;

        if overflow {
            self.tima = self.tma;
            return true; // Request Interrupt!
        }
        false
    }
    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            0xFF04 => (self.div >> 8) as u8,
            0xFF05 => self.tima,
            0xFF06 => self.tma,
            0xFF07 => self.tac,
            _ => 0,
        }
    }

    pub fn write(&mut self, addr: u16, value: u8) {
        match addr {
            0xFF04 => self.div = 0,
            0xFF05 => self.tima = value,
            0xFF06 => self.tma = value,
            0xFF07 => self.tac = value,
            _ => {}
        }
    }
}
