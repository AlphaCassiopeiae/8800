// import recall RAM struct and functions
// might just move contents of recall.rs here since its small anyway
mod recall;
use recall::RAM;
// TODO: figure out how to use tram stuff (BusRequest/Response) here


fn main() -> ! {

    let mut ram: RAM = RAM::new();

    loop {
        // use Option to detect if request for memory has come in
        // recall itself should be completely blind to machine cycles
        // just respond to requests as they come in
        if let Some(request) = tram.recieve() {
            match request {
                // do more here
                BusRequest::Read{addr} => {
                    // read and return data on bus
                }
                BusRequest::Write{addr, data} => {
                    // write data to mem at addr
                } 
                _ => {} // should never happen but who knows
            }
        }
    }
}
