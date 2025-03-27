// define types to be used for communication between recall and tram
use std::sync::mpsc::channel;

pub enum Message {
    MemoryReadRequest{addr: usize},
    MemoryWriteRequest{addr: usize, data: u8},
    MemoryReadResponse{data: u8},
}

type TX = std::sync::mpsc::Sender<Message>
type RX = std::sync::mpsc::Reciever<Message>

// for wrapping GPIO pins into something meaningful
pub struct BusState {
    pub addr: usize,
    pub data: u8,
    pub read_request: bool,
    pub write_request: bool,
    pub read_response: bool,
}