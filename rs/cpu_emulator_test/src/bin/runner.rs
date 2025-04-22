// runner.rs - this will run on the CPU card
#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_time::Timer;
use embassy_rp::gpio;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use {defmt_rtt as _, panic_probe as _};

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

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    // setup of components/channels
    let mut ram: RAM = RAM::new();
    ram.init();
    let mut cpu: CPU = CPU::new(ram);
    cpu.reset();

    loop {
        // just gonna fetch instructions until halt, then exit
        // no GPIO yet
        let instr: u8 = cpu.fetch_instruction();

        cpu.execute_instruction(instr);

        if cpu.halted() {
            info!("HLT detected, exiting...\r");
            break;
        }
        Timer::after_millis(250).await;
    }
}