// lib.rs
// defines types to be used by tram

// from is a string identifier for where the request came from ("cpu", "front panel", etc.)
pub enum BusRequest {
    Read  {from: String, addr: u16},
    Write {from: String, addr: u16, data: u8},
}

// to is a string identifier for where the response needs to go ("cpu", "front panel", etc.)
pub enum BusResponse {
    Read {to: String, data: u8},
}