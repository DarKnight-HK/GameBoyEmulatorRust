pub struct Joypad {
    pub joyp: u8,    // The register value at 0xFF00
    pub buttons: u8, // The internal state of all 8 buttons (1=Pressed)
}

impl Joypad {
    pub fn new() -> Self {
        Joypad {
            joyp: 0xCF,
            buttons: 0x00,
        }
    }

    pub fn read(&self) -> u8 {
        let mut result = self.joyp | 0xCF;

        if (self.joyp & 0x20) == 0 {
            if (self.buttons & 0x80) != 0 {
                result &= !(1 << 3);
            } // Start
            if (self.buttons & 0x40) != 0 {
                result &= !(1 << 2);
            } // Select
            if (self.buttons & 0x20) != 0 {
                result &= !(1 << 1);
            } // B
            if (self.buttons & 0x10) != 0 {
                result &= !(1 << 0);
            } // A
        }

        if (self.joyp & 0x10) == 0 {
            // Check Down (Bit 3), Up (Bit 2), Left (Bit 1), Right (Bit 0)
            if (self.buttons & 0x08) != 0 {
                result &= !(1 << 3);
            } // Down
            if (self.buttons & 0x04) != 0 {
                result &= !(1 << 2);
            } // Up
            if (self.buttons & 0x02) != 0 {
                result &= !(1 << 1);
            } // Left
            if (self.buttons & 0x01) != 0 {
                result &= !(1 << 0);
            } // Right
        }

        result
    }

    pub fn write(&mut self, byte: u8) {
        self.joyp = (self.joyp & 0xCF) | (byte & 0x30);
    }
    pub fn set_button(&mut self, bit: u8, pressed: bool) -> bool {
        let was_pressed = (self.buttons & (1 << bit)) != 0;
        if pressed {
            self.buttons |= 1 << bit;
        } else {
            self.buttons &= !(1 << bit);
        }
        let is_now_pressed = (self.buttons & (1 << bit)) != 0;
        if !was_pressed && is_now_pressed {
            let is_action = bit >= 4;
            let is_direction = bit < 4;

            let select_action = (self.joyp & 0x20) == 0;
            let select_direction = (self.joyp & 0x10) == 0;

            if (is_action && select_action) || (is_direction && select_direction) {
                return true;
            }
        }
        false
    }
}
