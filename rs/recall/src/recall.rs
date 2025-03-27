// this code defines the recall struct that will communicate with tram
mod ram;
mod lib;

use ram::RAM;
use lib::{Message, TX, RX};

use std::sync::mpsc::channel;

// recall never directly uses GPIO
pub struct Recall {
    // needs a RAM instance and send/recieve channels
    pub ram: RAM,
    pub recv_channel: RX,
    pub send_channel: TX,
}

impl Recall {
    
    fn new(rx: RX, tx: TX) -> Self {
        // define channels and connect in main.rs
        Recall { 
            ram: RAM::new(),
            recv_channel: rx,
            send_channel: tx,
        }
    }

    pub fn send_msg(&self, message: Message) {
        self.send_channel.send(message).unwrap();
    }

    pub fn recv_msg(&self) -> Option<Message> {
        self.recv_channel.recv().ok()
    }

    // RAM reads and writes called like recall.ram.read(), recall.ram.write()
}