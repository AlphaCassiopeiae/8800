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

  pub fn write(&mut self, addr: usize, data: u8) {
    self.memory[addr] = data;
  }  
}

// TODO: figure out what tram/recall need to send to eachother
// enum of some kind, but not a bus request directly

struct Recall {
  memory: RAM,
  pub send_channel: std::sync::mpsc::Sender<>,
  pub recv_channel: std::sync::mpsc::Reciever<>,
}

impl Recall {
  pub fn new() -> Self {
    memory: RAM::new(),
  }

  pub fn send(&self, message: BusMessage) {
    self.send_channel.send(message).unwrap();
  }

  pub fn recv(&self) -> Option<BusMessage> {
    self.recv_channel.recv().ok()
  }
}
