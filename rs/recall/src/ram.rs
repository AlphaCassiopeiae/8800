// simple rust code to support basic read/write functionality of RAM
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

  pub fn write(&mut self, addr: usize, data: u8) {
    self.memory[addr] = data;
  }  
}