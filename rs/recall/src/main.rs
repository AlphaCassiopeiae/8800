// import recall RAM struct and functions
// might just move contents of recall.rs here since its small anyway
mod recall;
use recall::RAM;

use tram::{TramSim, BusMessage};

use std::sync::mpsc::channel;

// not useful for simulation, will be needed for actual implementation
fn main() -> ! {

    let (tx, rx) = channel();

    // instances of RAM and tram
    // main file outside of all of this will link channels together
    let mut ram: RAM = RAM::new();
    let mut tram: TramSim = TramSim{recv_channel:rx, send_channel:tx};

    loop {
        // use Option to detect if request for memory has come in
        // recall itself should be completely blind to machine cycles
        // just respond to requests as they come in
        if let Some(request) = tram.recv() {
            match request {
                // do more here
                BusMessage::ReadRequest{from, addr} => {
                    // read and return data on bus
                    let response: BusMessage = BusMessage::ReadResponse{to:(from), data:(ram.read(addr))};
                    // still need to send response on TramSim send_channel
                    
                }
                BusRequest::WriteRequest{from, addr, data} => {
                    // write data to mem at addr
                    ram.write(addr, data);
                } 
                _ => {} // should never happen but who knows
            }
        }
    }
}
