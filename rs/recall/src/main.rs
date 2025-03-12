#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::gpio;
use embassy_rp::gpio::{Input, Pull};
use embassy_rp::Peripherals;
use embassy_time::Timer;
use gpio::{Level, Output};
use {defmt_rtt as _, panic_probe as _};

mod recall;
use recall::RAM;

#[link_section = ".bi_entries"]
#[used]
pub static PICOTOOL_ENTRIES: [embassy_rp::binary_info::EntryAddr; 4] = [
    embassy_rp::binary_info::rp_program_name!(c"recall"),
    embassy_rp::binary_info::rp_program_description!(
        c"This runs a memory emulator on the rp2350"
    ),
    embassy_rp::binary_info::rp_cargo_version!(),
    embassy_rp::binary_info::rp_program_build_attribute!(),
];

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut memory: RAM = RAM::new();

    // testing basic read/write
    let byte: u8 = 100;
    let addr: usize = 10;
    memory.write(addr, byte);

    let mut readval: u8 = memory.read(addr);
    println!("The value {} is at address {}", readval, addr);

    // reading random address should return 0
    let unset_addr: usize = 15;
    readval =  memory.read(unset_addr);
    println!("The value {} is at address {}", readval, unset_addr);

    // ALL THIS IS PROBABLY USELESS IN RECALL, COULD BE USEFUL IN TRAM
    // also could be good for testing reads/writes without tram
    // A0-A15: address pins (GPIO 39-24)
    // D0-D7: data pins (GPIO 47-40)
    // MWRT: low-active, identifies write operation (GPIO 19)
    // SMEMR: low-active, identifes read operation (GPIO 20)
    // PDBIN: high active, tells memory to put read value on D0-D7 (GPIO 12)  
    // SWO: low-active, indicates write operation (GPIO 13)

    // TODO: investigate difference between MWRT and SWO
    let p: Peripherals = embassy_rp::init(Default::default());

    // high active
    let mut xrdy: Output<'_> = Output::new(p.PIN_18, Level::Low);
    // high active
    let mut pdbin: Input<'_>= Input::new(p.PIN_12, Pull::Down);
    // low active
    let mut swo: Input<'_>= Input::new(p.PIN_13, Pull::Up);
    // low active
    let mut mwrt: Input<'_> = Input::new(p.PIN_19, Pull::Up);
    // low active
    let mut smemr: Input<'_> = Input::new(p.PIN_20, Pull::Up);

    // address lines A15-A0
    let mut addr_lines: [Input<'_>; 16] = [
        Input::new(p.PIN_39, Pull::Down), // A0
        Input::new(p.PIN_38, Pull::Down), // A1
        Input::new(p.PIN_37, Pull::Down), // ...
        Input::new(p.PIN_36, Pull::Down),
        Input::new(p.PIN_35, Pull::Down),
        Input::new(p.PIN_34, Pull::Down),
        Input::new(p.PIN_33, Pull::Down),
        Input::new(p.PIN_32, Pull::Down),
        Input::new(p.PIN_31, Pull::Down),
        Input::new(p.PIN_30, Pull::Down),
        Input::new(p.PIN_29, Pull::Down),
        Input::new(p.PIN_28, Pull::Down),
        Input::new(p.PIN_27, Pull::Down),
        Input::new(p.PIN_26, Pull::Down), // ...
        Input::new(p.PIN_25, Pull::Down), // A14
        Input::new(p.PIN_24, Pull::Down), // A15
    ]; 

    // TODO: figure out how to reassign D7-D0 pins (40-47) dynamically
    //          - will depend on read or write
    //          - might also be a tram thing idk
    //          -- callback functions, args data, addr, etc

    // loop {
        // this is where we will be checking if memory control signals are asserted
        // respond accordingly (read or write)

        // if write operation, D lines are inputs
        // let mut d7 = Input::new(p.PIN_40, Pull::Up)
        // ...

        // drop pins after every operation (?)
        // if read operation, D lines are inputs
        // let mut d7 = Output::new(p.PIN_40, Level::Low)
        // ...
        // set high or low based on value in memory
        // if readval % 2 == 0
        // D0 low
        // else 
        // D0 high
        // this seems ass

        // new strat gonna use an arr of pins, use lshift and mod to assign a u8 bit by bit
        // maybe bitwise &
    // }
}
