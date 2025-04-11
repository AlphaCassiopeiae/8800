// defines the tram struct used to communicate with packages
#![no_std]
use common::*;

pub struct Tram {
    pub recv_channel: RX,
    pub send_channel: TX,
}

impl Tram {
    pub fn new(tx: TX, rx: RX) -> Self {
        Self {
            recv_channel: rx,
            send_channel: tx,
        }
    }

    /* functions for communication with packages */
    pub async fn send_msg(&self, message: Message) {
        self.send_channel.send(message).await;
    }

    pub async fn recv_msg(&self) -> Message {
        self.recv_channel.receive().await
    }

    /* functions for driving bus (GPIO, PIO, etc.) */
    // detect bus request using GPIO signals, translate into message to send to recall
    // TODO: add GPIO pins as argument to this function
    pub fn detect_memory_request(&self) -> BusState {
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
