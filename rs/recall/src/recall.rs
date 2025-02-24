// memory is literally just going to be a stack allocated array
// it will have methods for reading and writing that the s-100 bus will call? or not? 
const MEMORY_SIZE: usize = 65536;

pub struct RAM {
  memory: [u8; MEMORY_SIZE] // support 16-bit addresses
}

impl RAM {

  pub fn new() -> Self {
    RAM {
      memory: [0; MEMORY_SIZE]
    }
  } 

  pub fn read(&mut self, addr: usize) -> u8 {
    self.memory[addr]
  }

  pub fn write(&mut self, addr: usize, byte: u8) {
    self.memory[addr] = byte;
  }  
}
