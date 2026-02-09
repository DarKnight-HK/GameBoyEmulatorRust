use crate::bus::Bus;

pub struct Cpu {
    pc: u16,
   
}

impl Cpu {
    pub fn new() -> Self {
        Cpu { pc: 0 }
    }

   // Fetch -> Decode -> Execute
    pub fn step(&mut self, bus: &mut Bus) {
       
        let opcode = bus.read_byte(self.pc);
        
        println!("PC: {:#04X} | Opcode: {:#02X}", self.pc, opcode);
        
        self.pc = self.pc.wrapping_add(1);
        
        // TODO: Decode the opcode here
    }
}