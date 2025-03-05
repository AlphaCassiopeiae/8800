#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::gpio;
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
async fn main(_spawner: Spawner) -> ! {
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

    loop {
        // this is where we will be checking if memory control signals are asserted
        // respond accordingly (read or write)
        // A0-A15: address pins
        // D0-D7: data pins
        // nPWR: low-active, bus signal confirming D0-D7 is valid
        // MWRITE: identifies read operation
        // SMEMR: identifes read operation
        // PDBIN: control signal, tells memory to put read value on D0-D7
        // panic!() doesn't work with the current build, surely theres a way to fix
        // panic!("not done with micro stuff yet");
    }
}
