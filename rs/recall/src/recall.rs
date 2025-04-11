// recall.rs - creates Recall struct to house RAM and communicate with tram
use crate::ram::*;
use common::*;

// recall never directly uses GPIO
pub struct Recall {
    // needs a RAM instance and send/recieve channels
    pub ram: RAM,
    pub recv_channel: RX,
    pub send_channel: TX,
}

impl Recall {
    
    pub fn new(tx: TX, rx: RX) -> Self {
        // define channels and connect in main.rs
        Recall { 
            ram: RAM::new(),
            recv_channel: rx,
            send_channel: tx,
        }
    }

    pub async fn send_msg(&self, message: Message) {
        self.send_channel.send(message).await;
    }

    pub async fn recv_msg(&self) -> Message {
        self.recv_channel.receive().await
    }

    // RAM reads and writes called like recall.ram.read(), recall.ram.write()
}