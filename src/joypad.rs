pub struct Joypad {
    pub left: bool,
    pub right: bool,
    pub up: bool,
    pub down: bool,

    pub a: bool,
    pub b: bool,
    pub select: bool,
    pub start: bool,

    pub select_dpad: bool,
    pub select_buttons: bool,
}

impl Joypad {
    pub fn new() -> Self {
        Joypad {
            left: false,
            right: false,
            up: false,
            down: false,
            a: false,
            b: false,
            select: false,
            start: false,

            // By default, the game hasn't selected anything yet
            select_dpad: false,
            select_buttons: false,
        }
    }

    pub fn read(&self) -> u8 {
        let mut state: u8 = 0xCF;

        if self.select_dpad {
            state &= 0b1110_1111;
            if self.right {
                state &= 0b1111_1110;
            } // Bit 0
            if self.left {
                state &= 0b1111_1101;
            } // Bit 1
            if self.up {
                state &= 0b1111_1011;
            } // Bit 2
            if self.down {
                state &= 0b1111_0111;
            } // Bit 3
        }

        if self.select_buttons {
            state &= 0b1101_1111; // Clear bit 5 to indicate Buttons are selected
            if self.a {
                state &= 0b1111_1110;
            } // Bit 0
            if self.b {
                state &= 0b1111_1101;
            } // Bit 1
            if self.select {
                state &= 0b1111_1011;
            } // Bit 2
            if self.start {
                state &= 0b1111_0111;
            } // Bit 3
        }

        state
    }

    pub fn write(&mut self, byte: u8) {
        self.select_buttons = (byte & 0x20) == 0;
        self.select_dpad = (byte & 0x10) == 0;
    }

    pub fn set_button(&mut self, bit: u8, pressed: bool) -> bool {
        let mut request_interrupt = false;
        let previously_pressed = match bit {
            0 => self.right,
            1 => self.left,
            2 => self.up,
            3 => self.down,
            4 => self.a,
            5 => self.b,
            6 => self.select,
            7 => self.start,
            _ => false,
        };

        match bit {
            0 => self.right = pressed,
            1 => self.left = pressed,
            2 => self.up = pressed,
            3 => self.down = pressed,
            4 => self.a = pressed,
            5 => self.b = pressed,
            6 => self.select = pressed,
            7 => self.start = pressed,
            _ => {}
        }

        if !previously_pressed && pressed {
            let is_action = bit >= 4;
            let is_direction = bit < 4;

            if (is_action && self.select_buttons) || (is_direction && self.select_dpad) {
                request_interrupt = true;
            }
        }

        request_interrupt
    }
}
