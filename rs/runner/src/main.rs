// main.rs - main of 8080 emulator
// will set up channels to communicate with its tram instance
#![no_std]
#![no_main]

mod cpu;
mod instructions;
mod runner;

use common::Message;
use defmt::*;
use {defmt_rtt as _, panic_probe as _};
use embassy_executor::Spawner;
use embassy_rp::gpio::{Input, Output, Pull, Level};
use embassy_rp::Peripherals;
use embassy_time::{Timer, Duration};
use embassy_sync::channel::Channel;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;

use common::*;
use tram::*;
use runner::*;

#[unsafe(link_section = ".bi_entries")]
#[used]
pub static PICOTOOL_ENTRIES: [embassy_rp::binary_info::EntryAddr; 4] = [
    embassy_rp::binary_info::rp_program_name!(c"runner"),
    embassy_rp::binary_info::rp_program_description!(
        c"8080 emulator that listens to external clock"
    ),
    embassy_rp::binary_info::rp_cargo_version!(),
    embassy_rp::binary_info::rp_program_build_attribute!(),
];

static RUNNER_TO_TRAM: Channel<CriticalSectionRawMutex, Message, 4> = Channel::new();
static TRAM_TO_RUNNER: Channel<CriticalSectionRawMutex, Message, 4> = Channel::new();

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    // instances of runner and tram, channels connected
    let mut runner: Runner = Runner::new(RUNNER_TO_TRAM.sender(), TRAM_TO_RUNNER.receiver());
    let mut tram: Tram = Tram::new(TRAM_TO_RUNNER.sender(), RUNNER_TO_TRAM.receiver());

    // initialize GPIO stuff
    let p: Peripherals = embassy_rp::init(Default::default());

    let mut clk: Input<'_> = Input::new(p.PIN_4, Pull::Down);

    // let mut instr_present: bool = false; // keeps track if we have an instruction to run
    // let mut current_instr: u8 = 0;
    // let mut waiting_for_tram: bool = false;
    // let mut waiting_for_mem: bool = false;

    loop {
        // check if tram has anything to do first ?
        // if waiting_for_tram {
        //     let recieved_msg: Message = tram.recv_msg().await;
        //     // tram should have whatever runner sent, now do something on GPIO
        //     match recieved_msg {
        //         Message::MemoryReadRequest { addr } => {
        //             // details TBD
        //         }
        //         Message::MemoryWriteRequest { addr, data } => {
        //             // details TBD
        //         }
        //         _ => {}
        //     }
        //     waiting_for_tram = false;
        //     waiting_for_mem = true;
        // }

        // if waiting_for_mem {
        //     // wait for tram to recieve memory response
        //     // probably gonna want some kind of await response function
        //     // but call that here and await, then forward response to runner to decode
        //     // after these waiting conditions are checked, do normal CPU operation

        //     // turn of waiting for mem flag at end
        //     waiting_for_mem = false;
        //     // if got instruction, set instr_present = true;
        // }
        // // check if GPIO 4's level has changed from low->high
        // clk.wait_for_rising_edge().await;
        // info!("positive clk edge!");

        // if instr_present {
        //     // continue instruction execution
        //     // runner.cpu.execute_instruction(instruction);

        //     // if cycle num > num_cycles, clear instruction
        //     // so next cycle a new instruction is fetched
        // } else {
        //     // fetch instruction from memory
        //     // generate tram request to be sent over bus
        //     // will use pc for instruction addr
        //     let message: Message = Message::MemoryReadRequest{ addr: runner.cpu.read_pc() as usize };
        //     runner.send_msg(message).await;
        //     waiting_for_tram = true;
        // }
    }
}