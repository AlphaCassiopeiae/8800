// runner.rs - defines runner struct that will interact with tram
use common::*;
use crate::cpu::*;

pub struct Runner {
    pub cpu: CPU,
    pub recv_channel: RX,
    pub send_channel: TX,
}

impl Runner {
    pub fn new(tx: TX, rx: RX) -> Self {
        Self {
            cpu: CPU::new(),
            recv_channel: rx,
            send_channel: tx,
        }
    }
}