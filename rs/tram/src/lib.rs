// lib.rs
// defines types to be used by tram

// All of this is simply for simulation purposes
// but the idea is the same for the final design, just using embassy/PIO instead of channel
// will need to create struct of GPIO signals as bus message
pub enum BusMessage {
    ReadRequest{from: String, addr: usize},
    WriteRequest{from: String, addr: usize, data: u8},
    ReadResponse{to: String, data: u8},
}

pub struct TramSim {
    pub recv_channel: std::sync::mpsc::Receiver<BusMessage>,
    pub send_channel: std::sync::mpsc::Sender<BusMessage>,
}

// want to be able to send and recieve any type of bus message (requests and responses)
impl TramSim {
    pub fn send(&self, message: BusMessage) {
        self.send_channel.send(message).unwrap();
    }

    pub fn recv(&self) -> Option<BusMessage> {
        self.recv_channel.recv().ok()
    }
}