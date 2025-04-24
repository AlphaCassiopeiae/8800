// runner.rs - this will run on the CPU card
#![no_std]
#![no_main]

// use cortex_m::Peripherals;
use defmt::*;
use embassy_executor::Spawner;
use embassy_time::Timer;
use embassy_rp::gpio::{Input, Output, Level, Pull};
use embassy_rp::Peripherals;
use {defmt_rtt as _, panic_probe as _};

// channels for communication with Tram
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;

use cpu_emulator::cpu::CPU;
use cpu_emulator::ram::RAM;

#[unsafe(link_section = ".bi_entries")]
#[used]
pub static PICOTOOL_ENTRIES: [embassy_rp::binary_info::EntryAddr; 4] = [
    embassy_rp::binary_info::rp_program_name!(c"runner"),
    embassy_rp::binary_info::rp_program_description!(c"Intel 8080 Emulator"),
    embassy_rp::binary_info::rp_cargo_version!(),
    embassy_rp::binary_info::rp_program_build_attribute!(),
];

/* NOTES */
// GPIO 16-31 are A0-A15
// GPIO 8-15 are D0-D7
// CLK is GPIO 0, active high
// M1 on GPIO 33 (for instruction boundary detection)

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    // GPIO initialization
    let p: Peripherals = embassy_rp::init(Default::default());
    let clk = Input::new(p.PIN_0, Pull::Down);
    // setup of components/channels
    let mut ram: RAM = RAM::new();
    ram.init();
    let mut cpu: CPU = CPU::new(ram);
    cpu.reset();

    loop {
        // just gonna fetch instructions until halt, then exit
        // i.e. get run mode working first

        // fetch instruction will also need to make a tram request
        // instead of reading local RAM instance
        let instr: u8 = cpu.fetch_instruction();

        // execute_instruction will need to be modified to make tram requests 
        // instead of reading/writing local RAM instance
        cpu.execute_instruction(instr);
        cpu.show_state();

        // will just want to pause, not break from loop
        // so this will need to be changed
        if cpu.halted() {
            info!("HLT detected, exiting...\r");
            break;
        }

        // not necessary for final implementation but nice for terminal debugging
        Timer::after_millis(250).await;
    }
}