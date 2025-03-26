// import recall RAM struct and functions
// might just move contents of recall.rs here since its small anyway
mod recall;
use recall::RAM;

use tram::{TramSim, BusMessage};


fn main() -> ! {

    let mut ram: RAM = RAM::new();
    // TODO: still need a TramSim instance to send responses back
    let mut tram: TramSim = 0; // idk 
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
