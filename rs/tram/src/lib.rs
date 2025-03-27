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
    // clock
    pub clk: Anypin,
    // address and data lines
    pub addr: [AnyPin; 16],
    pub data: [AnyPin; 8],
    // control signals
    pub pinte: AnyPin,
    pub prot: AnyPin,
    pub smemr: AnyPin,
    pub sinp: AnyPin,
    pub sm1: AnyPin,
    pub sout: AnyPin,
    pub shlta: AnyPin,
    pub sstack: AnyPin,
    pub swo: AnyPin,
    pub sinta: AnyPin,
    pub xrdy: AnyPin,
    pub pwait: AnyPin,
    pub prdy: AnyPin,
    pub n_pint: AnyPin,
    pub n_preset: AnyPin,
    pub n_phold: AnyPin,
    pub pdbin: AnyPin,
    pub mwrt: AnyPin,
    pub panel: AnyPin,
}