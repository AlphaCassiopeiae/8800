// lib.rs
// defines types to be used by tram
use std::sync::mpsc::channel;

use embassy_rp::gpio::{AnyPin, Pull, Level, Pin};

// add more to these if components need more info
// tram itself should be able to figure out what bus signals to assert
// => shouldn't need to be in these messages
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

// struct of GPIO pins
// defines names for easier use, actual assignments will be done in packages
// this will only ever be used by tram instances
// ex: clk is input for runner and recall, but output for front panel
pub struct Bus {
    // control                  
    pub clk: Anypin,        // Pin 4
    pub n_preset: AnyPin,   // Pin 5
    // UI
    pub panel: AnyPin,      // Pin 6
    pub prot: AnyPin,       // Pin 7
    pub sm1: AnyPin,        // Pin 8
    // status
    pub shlta: AnyPin,      // Pin 9
    pub sstack: AnyPin,     // Pin 10
    // DMA
    pub n_phold: AnyPin,    // Pin 11
    // R/W
    pub pdbin: AnyPin,      // Pin 12
    pub swo: AnyPin,        // Pin 13
    pub pwait: AnyPin,      // Pin 14
    // I/O
    pub prdy: AnyPin,       // Pin 15
    pub sout: AnyPin,       // Pin 16
    pub sinp: AnyPin,       // Pin 17
    // memory
    pub xrdy: AnyPin,       // Pin 18
    pub mwrt: AnyPin,       // Pin 19
    pub smemr: AnyPin,      // Pin 20
    // interrupt
    pub n_pint: AnyPin,     // Pin 21
    pub sinta: AnyPin,      // Pin 22
    pub pinte: AnyPin,      // Pin 23
    // A/D lines
    pub addr: [AnyPin; 16], // Pins 24-39
    pub data: [AnyPin; 8],  // Pins 40-47
}