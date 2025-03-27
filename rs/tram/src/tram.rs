// defines the tram struct used to communicate with packages
mod lib;
use lib::{Message, TX, RX, BusState};
use std::sync::mpsc::channel;

pub struct Tram {
    pub recv_channel: RX,
    pub send_channel: TX,
}

impl Tram {
    fn new(rx: RX, tx: TX) -> Self {
        recv_channel: rx,
        send_channel: tx,
    }

    /* functions for communication with packages */
    pub fn send_msg(&self, message: Message) {
        self.send_channel.send(message).unwrap();
    }

    pub fn recv_msg(&self) -> Option<Message> {
        self.recv_channel.recv().ok()
    }

    /* functions for driving bus (GPIO, PIO, etc.) */
    // detect bus request using GPIO signals, translate into message to send to recall
    // TODO: add GPIO pins as argument to this function
    pub fn detect_memory_request() -> BusState {
        // TODO: update this with GPIO assignments/logic to set bool flags
        // check if active memory request on GPIO pins
        let addr: usize = 0;
        let data: u8 = 0;
        let read_request: bool = false;
        let write_request: bool = false;
        let read_response: bool = false;

        // return corresponding bus state
        BusState {addr, data, read_request, write_request, read_response}
    }
}
