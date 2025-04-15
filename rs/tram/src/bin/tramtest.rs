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

#[unsafe(link_section = ".bi_entries")]
#[used]
pub static PICOTOOL_ENTRIES: [embassy_rp::binary_info::EntryAddr; 4] = [
    embassy_rp::binary_info::rp_program_name!(c"Blinky Example Expanded"),
    embassy_rp::binary_info::rp_program_description!(
        c"This example cycles through gpio pins 0-47 (skipping 42-45), turning each on and off."
    ),
    embassy_rp::binary_info::rp_cargo_version!(),
    embassy_rp::binary_info::rp_program_build_attribute!(),
];

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    // Create outputs for pins 0-41 and 46-47 (skipping 42, 43, 44, 45).
    let mut outputs = [
        Output::new(p.PIN_0, Level::Low),
        Output::new(p.PIN_1, Level::Low),
        Output::new(p.PIN_2, Level::Low),
        Output::new(p.PIN_3, Level::Low),
        Output::new(p.PIN_4, Level::Low),
        Output::new(p.PIN_5, Level::Low),
        Output::new(p.PIN_6, Level::Low),
        Output::new(p.PIN_7, Level::Low),
        Output::new(p.PIN_8, Level::Low),
        Output::new(p.PIN_9, Level::Low),
        Output::new(p.PIN_10, Level::Low),
        Output::new(p.PIN_11, Level::Low),
        Output::new(p.PIN_12, Level::Low),
        Output::new(p.PIN_13, Level::Low),
        Output::new(p.PIN_14, Level::Low),
        Output::new(p.PIN_15, Level::Low),
        Output::new(p.PIN_16, Level::Low),
        Output::new(p.PIN_17, Level::Low),
        Output::new(p.PIN_18, Level::Low),
        Output::new(p.PIN_19, Level::Low),
        Output::new(p.PIN_20, Level::Low),
        Output::new(p.PIN_21, Level::Low),
        Output::new(p.PIN_22, Level::Low),
        Output::new(p.PIN_23, Level::Low),
        Output::new(p.PIN_24, Level::Low),
        Output::new(p.PIN_25, Level::Low),
        Output::new(p.PIN_26, Level::Low),
        Output::new(p.PIN_27, Level::Low),
        Output::new(p.PIN_28, Level::Low),
        Output::new(p.PIN_29, Level::Low),
        Output::new(p.PIN_30, Level::Low),
        Output::new(p.PIN_31, Level::Low),
        Output::new(p.PIN_32, Level::Low),
        Output::new(p.PIN_33, Level::Low),
        Output::new(p.PIN_34, Level::Low),
        Output::new(p.PIN_35, Level::Low),
        Output::new(p.PIN_36, Level::Low),
        Output::new(p.PIN_37, Level::Low),
        Output::new(p.PIN_38, Level::Low),
        Output::new(p.PIN_39, Level::Low),
        Output::new(p.PIN_40, Level::Low),
        Output::new(p.PIN_41, Level::Low),
        Output::new(p.PIN_46, Level::Low),
        Output::new(p.PIN_47, Level::Low),
    ];

    loop {
        for (i, pin) in outputs.iter_mut().enumerate() {
            info!("Turning on output {}...", i);
            pin.set_high();
            Timer::after_millis(250).await;

            info!("Turning off output {}...", i);
            pin.set_low();
            Timer::after_millis(250).await;
        }
    }
}
