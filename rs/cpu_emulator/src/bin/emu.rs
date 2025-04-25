// runner.rs - this will run on the CPU card
#![no_std]
#![no_main]

// use cortex_m::Peripherals;
use defmt::*;
// use core::concat;
// use core::stringify;
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
    let mut rst = Input::new(p.PIN_3, Pull::Up);
    let mut prdy = Input::new(p.PIN_2, Pull::Down);
    let mut hlta = Output::new(p.PIN_36, Level::Low);
    let mut panel = Output::new(p.PIN_1, Level::High); // low active
    // setup of components/channels
    let mut ram: RAM = RAM::new();
    ram.init();
    let mut cpu: CPU = CPU::new(ram);
    cpu.reset();

    loop {
        // just gonna fetch instructions until halt, then exit
        // i.e. get run mode working first

        if cpu.halted() {
            panel.set_low();
            hlta.set_high();
        }

        // rst.wait_for_falling_edge().await;
        if rst.get_level() == Level::Low {
            info!("Reset Occurred!");
            cpu.reset();
        }
        else if prdy.get_level() == Level::High {
            info!("cpu running...");
            panel.set_high();
            cpu.unhalt();

            let instr: u8 = cpu.fetch_instruction();
            cpu.execute_instruction(instr);
            cpu.show_state();

            cpu.halt();
            panel.set_low();
        }

        // not necessary for final implementation but nice for terminal debugging
        Timer::after_millis(50).await;
    }
}