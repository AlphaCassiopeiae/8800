// import recall RAM struct and functions
// might just move contents of recall.rs here since its small anyway
#![no_std]
#![no_main]

mod recall;
mod ram;

use common::*;
use recall::Recall;
use ram::RAM;
use tram::Tram;
use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::gpio;
use embassy_rp::gpio::{Input, Pull};
use embassy_rp::Peripherals;
use embassy_time::{Timer, Duration};
use gpio::{Level, Output};
use embassy_sync::channel::{Channel, Sender, Receiver};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use {defmt_rtt as _, panic_probe as _};

#[link_section = ".bi_entries"]
#[used]
pub static PICOTOOL_ENTRIES: [embassy_rp::binary_info::EntryAddr; 4] = [
    embassy_rp::binary_info::rp_program_name!(c"recall"),
    embassy_rp::binary_info::rp_program_description!(
        c"RAM simulator"
    ),
    embassy_rp::binary_info::rp_cargo_version!(),
    embassy_rp::binary_info::rp_program_build_attribute!(),
];

// channels for bidirectional communication between recall and tram instances
static TRAM_TO_RECALL: Channel<CriticalSectionRawMutex, Message, 4> = Channel::new();
static RECALL_TO_TRAM: Channel<CriticalSectionRawMutex, Message, 4> = Channel::new();

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    // instances of RAM and tram, channel assignments
    let mut recall: Recall = Recall::new(RECALL_TO_TRAM.sender(), TRAM_TO_RECALL.receiver());
    let mut tram: Tram = Tram::new(TRAM_TO_RECALL.sender(), RECALL_TO_TRAM.receiver());

    loop {
        // recall itself should be completely blind to machine cycles
        // just respond to requests as they come in
        let bus_state: BusState = tram.detect_memory_request();
        // if read or write request, forward request to recall through chaannel
        if bus_state.read_request {
            let message: Message = Message::MemoryReadRequest{addr: bus_state.addr};
            tram.send_msg(message);
        }
        else if bus_state.write_request {
            let message: Message = Message::MemoryWriteRequest{addr: bus_state.addr, data: bus_state.data};
            tram.send_msg(message);
        }

        // if recall recieves message from tram, take appropriate action
        // if let Some(message) = recall.recv_msg() {
        //     match message {
        //         Message::MemoryReadRequest{addr} => {
        //             // read and send response on recall tx channel for tram to recieve
        //             let response: Message = Message::MemoryReadResponse{data: (ram.read(addr))};
        //             recall.send_msg(response); 
        //         }
        //         BusRequest::MemoryWriteRequest{addr, data} => {
        //             // write data to mem at addr
        //             recall.ram.write(addr, data);
        //         } 
        //         _ => {} // recall should never recieve responses
        //     }
        // }

        // now using async await instead of Option, so no longer need match block
    }
}
