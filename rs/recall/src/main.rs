// import recall RAM struct and functions
// might just move contents of recall.rs here since its small anyway
mod lib;
mod recall;

use lib::{Message, BusState};
use recall::Recall;
use std::sync::mpsc::channel;

// TODO: figure out how to get Tram (from tram.rs in other package) in here

fn main() -> ! {
    // channels for bidirectional communication between recall and tram instances
    let (recall_tram_tx, recall_tram_rx) = channel();
    let (tram_recall_tx, tram_recall_rx) = channel();

    // instances of RAM and tram, channel assignments
    let mut recall = Recall::new(tram_recall_rx, recall_tram_tx);
    let mut tram   = Tram::new(recall_tram_rx, tram_recall_tx);

    loop {
        // recall itself should be completely blind to machine cycles
        // just respond to requests as they come in
        let bus_state: BusState = tram.detect_memory_request();
        // if read or write request, forward request to recall through chaannel
        if bus_state.read_request {
            let message: Message::MemoryReadRequest{addr: bus_state.addr};
            tram.send(message);
        }
        else if bus_state.write_request {
            let message: Message::MemoryWriteRequest{addr: bus_state.addr, data: bus_state.data};
            tram.send(message);
        }

        // if recall recieves message from tram, take appropriate action
        if let Some(message) = recall.recv() {
            match message {
                Message::MemoryReadRequest{addr} => {
                    // read and send response on recall tx channel for tram to recieve
                    let response: Message = Message::MemoryReadResponse{data: (ram.read(addr))};
                    recall.send_msg(response); 
                }
                BusRequest::MemoryWriteRequest{addr, data} => {
                    // write data to mem at addr
                    recall.ram.write(addr, data);
                } 
                _ => {} // recall should never recieve responses
            }
        }
    }
}
