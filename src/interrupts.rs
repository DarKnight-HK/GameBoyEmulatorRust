// https://gbdev.io/pandocs/Interrupt_Sources.html
#[derive(Debug, Clone, Copy)]
pub enum Interrupt {
    VBlank = 0,
    LcdStat = 1,
    Timer = 2,
    Serial = 3,
    Joypad = 4,
}

impl Interrupt {
    pub fn handle_address(&self) -> u16
    {
        match self {
            Interrupt::VBlank => 0x0040,
            Interrupt::LcdStat => 0x0048,
            Interrupt::Timer => 0x0050,
            Interrupt::Serial => 0x0058,
            Interrupt::Joypad => 0x0060,
        }
    }
}