#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_rp::gpio::{Input, Pull};
use embassy_rp::pio::{Config as PioConfig, Pio};
use embassy_rp::uart;
use defmt::info;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    // Setup UART1 on pins 42 (TX) and 43 (RX) for logging.
    let mut uart = uart::Uart::new_blocking(p.UART1, p.PIN_42, p.PIN_43, uart::Config::default());

    // Define a PIO program that writes a byte on an 8-bit bus and pulses a write-strobe (WR).
    // Data bus is output via the "pins" group (8 pins) and WR is driven via side-set.
    let prg = embassy_rp::pio::program::pio_asm!(
        "
        pull             ; pull data from TX FIFO into OSR
        out pins, 8 side 1  ; drive 8-bit data on data bus, set WR high (inactive) via side-set
        nop         side  ; pulse: set WR low (active)
        nop         side 1  ; return WR high
        "
    );

    // Initialize the PIO state machine.
    let pio = Pio::new(p.PIO0);
    let mut cfg = PioConfig::default();
    // Configure to use 8 consecutive pins for the data bus and one separate GPIO for WR:
    // Here, data bus: pins 10..17, and WR: pin 18.
    cfg.out_pin_base = p.PIN_10.number();
    cfg.side_set_config = Some(embassy_rp::pio::SideSetConfig {
        enabled: true,
        optional: false,
        num_pins: 1,
        base_pin: p.PIN_18.number(),
    });
    // Load the program into PIO instruction memory.
    cfg.use_program(&pio.common().load_program(&prg.program), &[]);

    // Initialize state machine 0 with the configuration.
    let mut sm = pio.init_sm(0, &cfg);

    // Use an input pin for ACK (from the reader) on pin 19.
    let ack_pin = Input::new(p.PIN_19, Pull::None);

    let mut counter: u8 = 0;
    loop {
        // Push the data byte to the state machine's TX FIFO.
        sm.tx().push(counter as u32);

        // Log the value being written.
        let msg = defmt::format!("Writing value: {}\r\n", counter);
        uart.blocking_write(msg.as_bytes()).unwrap();

        // Wait for an ACK pulse:
        // Block until the ACK pin goes high then low, ensuring the reader has processed the data.
        while !ack_pin.is_high() {}
        while ack_pin.is_high() {}

        counter = counter.wrapping_add(1);
        cortex_m::asm::delay(500_000);
    }
}