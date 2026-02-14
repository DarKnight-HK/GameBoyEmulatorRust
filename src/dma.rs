pub struct Dma {
    pub active: bool,
    pub byte: u8,
    pub start_delay: u8,
}

impl Dma {
    pub fn new() -> Self {
        Dma {
            active: false,
            byte: 0,
            start_delay: 0,
        }
    }

    pub fn start(&mut self, value: u8) {
        self.active = true;
        self.byte = value;
        self.start_delay = 2; 
    }

    pub fn is_transferring(&self) -> bool {
        self.active
    }
}