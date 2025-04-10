// Embassy stuff
#![no_std]
#![no_main]

use defmt::*;
use common::*;

use embassy_executor::Spawner;
use embassy_rp::gpio;
use embassy_rp::Peripherals;
use embassy_time::{Timer, Duration};
use gpio::{Level, Output};
use {defmt_rtt as _, panic_probe as _};

#[unsafe(link_section = ".bi_entries")]
#[used]
pub static PICOTOOL_ENTRIES: [embassy_rp::binary_info::EntryAddr; 4] = [
    embassy_rp::binary_info::rp_program_name!(c"tram"),
    embassy_rp::binary_info::rp_program_description!(
        c"bus control with PIO routines, interfaces recall/runner/etc."
    ),
    embassy_rp::binary_info::rp_cargo_version!(),
    embassy_rp::binary_info::rp_program_build_attribute!(),
];

// TO BE USED BY CYCLE

// state machine to handle CPU reading from memory
// returns data
// need to go GPIO stuff in these functions or call PIO routines
async fn cpu_mem_read(addr: u16) -> u8 {
    // EXAMPLE USAGE: let val: u8 = cpu_mem_read(addr).await;

    // CHECK IN RUN MODE

    // T1
    // place addr on bus
    // set smemr to 1

    // wait clk 1

    // set pdbin low

    // wait for xrdy (should go low on T2)

    // wait one clk cycle (Tw)

    // if / wait xrdy to go high

    // data available
    // deassert smemr and pdbin
    0
}

// state machine to handle CPU writing to memory
async fn cpu_mem_write(addr: u16, data: u8) {
    // EXAMPLE USAGE: cpu_mem_write(addr, data).await;
}

// i understand it now
// request should trigger stuff in recall and return data to CPU
pub async fn cpu_mem_read_req(addr: u16) -> u8 {
    // create BusRequest{}
    // send()
    // recall will recognize request at some point
    // will call read()/write()
    // will tell it's tram instance to send response (CPU emulator is awaiting)
    // CPU emulator gets the data (u8)

    // CPU emulator
    // use tram::{...}
    // call this fn when needed and await response
    // returns u8 which is what CPU needs

    // will need to be a combination of bus signals eventually
    // probably call a PIO routine

    // for testing purposes rn jsut gonna use the BusRequest enum
    // let request: BusRequest = BusRequest::Read{from:("cpu"), addr:(addr)};
    // send request on channel
    0
}

async fn cpu_mem_write_req() {
    // ?
    // this should call recall's write() function at some point in the machine cycles
    // no data returned, maybe need to return some flags to the CPU indicating write finished?
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {

    let p: Peripherals = embassy_rp::init(Default::default());

    let mut led: Output<'_> = Output::new(p.PIN_25, Level::Low);

    loop {
        info!("led on!");
        led.set_high();
        Timer::after(Duration::from_millis(500)).await;

        info!("led off!");
        led.set_low();
        Timer::after(Duration::from_millis(500)).await;
    }
} // just trying to blink an LED for now

// defines the tram struct used to communicate with packages


pub struct Tram {
    pub recv_channel: RX,
    pub send_channel: TX,
}

impl Tram {
    pub fn new(tx: TX, rx: RX) -> Self {
        Self {
            recv_channel: rx,
            send_channel: tx,
        }
    }

    /* functions for communication with packages */
    pub async fn send_msg(&self, message: Message) {
        self.send_channel.send(message).await;
    }

    pub async fn recv_msg(&self) -> Message {
        self.recv_channel.receive().await
    }

    /* functions for driving bus (GPIO, PIO, etc.) */
    // detect bus request using GPIO signals, translate into message to send to recall
    // TODO: add GPIO pins as argument to this function
    pub fn detect_memory_request(&self) -> BusState {
        // TODO: update this with GPIO assignments/logic to set bool flags
        // check if active memory request on GPIO pins
        let addr: usize = 0;
        let data: u8 = 0;
        let read_request: bool = false;
        let write_request: bool = false;
        let read_response: bool = false;

        // return corresponding bus state
        BusState {addr, data, read_request, write_request, read_response}
    }
}
