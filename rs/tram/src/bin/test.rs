// #![no_std]
// #![no_main]

// use embassy_executor::Spawner;
// use embassy_rp::gpio::{Output, Level, Pull};
// use embassy_rp::pio::{Config as PioConfig, Pio};
// use embassy_rp::uart;
// use defmt::info;
// use {defmt_rtt as _, panic_probe as _};

// #[embassy_executor::main]
// async fn main(_spawner: Spawner) {
//     let p = embassy_rp::init(Default::default());

//     // Setup UART0 on pins 46 (TX) and 47 (RX) for logging.
//     let mut uart = uart::Uart::new_blocking(p.UART0, p.PIN_46, p.PIN_47, uart::Config::default());

//     // Define a PIO program that waits for the WR pulse, reads the 8-bit data bus, and pushes it to the FIFO.
//     // The program uses WAIT instructions on a designated pin (WR).
//     let prg = embassy_rp::pio::program::pio_asm!(
//         "
//         .wrap_target
//             wait 0 pin, 0   ; wait until WR is low (active)
//             in pins, 8      ; sample 8-bit data from the data bus
//             push            ; push the data to the RX FIFO
//             wait 1 pin, 0   ; wait until WR goes high (inactive)
//             jmp 0           ; repeat
//         .wrap
//         "
//     );

//     let pio = Pio::new(p.PIO0);
//     let mut cfg = PioConfig::default();
//     // Configure the data bus (input) on pins 10..17 and use pin 18 as the wait pin for WR.
//     cfg.in_pin_base = p.PIN_10.number();
//     cfg.wait_pin = Some(p.PIN_18.number());
//     // Load the reader program into PIO and initialize state machine 1.
//     cfg.use_program(&pio.common().load_program(&prg.program), &[]);

//     let mut sm = pio.init_sm(1, &cfg);

//     // Configure an output for ACK on pin 19.
//     let mut ack_pin = Output::new(p.PIN_19, Level::Low);

//     loop {
//         // Wait for a byte from the parallel bus.
//         let data: u32 = sm.rx().pop();
//         let byte = data as u8;

//         // Log the read value over UART0.
//         let msg = defmt::format!("Read value: {}\r\n", byte);
//         uart.blocking_write(msg.as_bytes()).unwrap();

//         // Pulse ACK: set the ACK pin high briefly then return it low.
//         ack_pin.set_high();
//         cortex_m::asm::delay(100_000);
//         ack_pin.set_low();
//     }
// }

//! This example test the RP Pico on board LED.
//!
//! It does not work with the RP Pico W board. See wifi_blinky.rs.

#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::gpio;
use embassy_time::Timer;
use gpio::{Level, Output};
use {defmt_rtt as _, panic_probe as _};

// Program metadata for `picotool info`.
// This isn't needed, but it's recomended to have these minimal entries.
#[link_section = ".bi_entries"]
#[used]
pub static PICOTOOL_ENTRIES: [embassy_rp::binary_info::EntryAddr; 4] = [
    embassy_rp::binary_info::rp_program_name!(c"Blinky Example"),
    embassy_rp::binary_info::rp_program_description!(
        c"This example tests the RP Pico on board LED, connected to gpio 25"
    ),
    embassy_rp::binary_info::rp_cargo_version!(),
    embassy_rp::binary_info::rp_program_build_attribute!(),
];

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    let mut led = Output::new(p.PIN_25, Level::Low);

    loop {
        info!("led on!");
        led.set_high();
        Timer::after_millis(250).await;

        info!("led off!");
        led.set_low();
        Timer::after_millis(250).await;
    }
}
