// main.rs - main of 8080 emulator
// will set up channels to communicate with its tram instance
#![no_std]
#![no_main]

mod cpu;
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

#[link_section = ".bi_entries"]
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

    loop {
        // check if GPIO 4's level has changed from low->high
        // if it has, toggle LED
        // result is an LED that toggles with every positive clock edge
        clk.wait_for_rising_edge().await;
        info!("positive clk edge!");
    }
}