#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::bind_interrupts;
use embassy_rp::gpio::{Level, Output};
use embassy_rp::peripherals::{PIO0};
use embassy_rp::pio::program::pio_asm;
use embassy_rp::pio::{Common, Config as PioConfig, InterruptHandler as PioInterruptHandler, Pio, Pin, ShiftDirection, StateMachine};
use embassy_time::Timer;
use fixed::traits::ToFixed;
use fixed_macro::types::U56F8;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => PioInterruptHandler<PIO0>;
});

/// Configure PIO to output data from FIFO to pins 16-31
fn setup_pio_output<'a>(pio: &mut Common<'a, PIO0>, sm: &mut StateMachine<'a, PIO0, 0>, pins: &[&Pin<'a, PIO0>]) {
    // PIO program to output 16 bits to pins from FIFO
    let prg = pio_asm!(
        "set pindirs, 1",  // Set pins as outputs
        ".wrap_target",
        "out pins, 16",    // Output 16 bits to pins
        "pull block",      // Wait for more data from FIFO
        ".wrap",
    );

    let mut cfg = PioConfig::default();
    cfg.use_program(&pio.load_program(&prg.program), &[]);
    cfg.set_out_pins(pins);
    cfg.clock_divider = (U56F8!(125_000_000) / 10000).to_fixed(); // 10 kHz update rate
    cfg.shift_out.auto_fill = true;
    cfg.shift_out.direction = ShiftDirection::Right;
    sm.set_config(&cfg);
    sm.set_pin_dirs(embassy_rp::pio::Direction::Out, pins);
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    // non parallel leds
    let mut led = Output::new(p.PIN_4, Level::Low);

    // PIO setup for output pins 16-31
    let Pio { mut common, mut sm0, .. } = Pio::new(p.PIO0, Irqs);

    // Create pin references for pins 16-31 using `common.make_pio_pin`
    let pins: [&_ ; 16] = [
        &common.make_pio_pin(p.PIN_16),
        &common.make_pio_pin(p.PIN_17),
        &common.make_pio_pin(p.PIN_18),
        &common.make_pio_pin(p.PIN_19),
        &common.make_pio_pin(p.PIN_20),
        &common.make_pio_pin(p.PIN_21),
        &common.make_pio_pin(p.PIN_22),
        &common.make_pio_pin(p.PIN_23),
        &common.make_pio_pin(p.PIN_24),
        &common.make_pio_pin(p.PIN_25),
        &common.make_pio_pin(p.PIN_26),
        &common.make_pio_pin(p.PIN_27),
        &common.make_pio_pin(p.PIN_28),
        &common.make_pio_pin(p.PIN_29),
        &common.make_pio_pin(p.PIN_30),
        &common.make_pio_pin(p.PIN_31),
    ];

    // Configure PIO for output
    setup_pio_output(&mut common, &mut sm0, &pins);

    // Enable the state machine
    sm0.set_enable(true);

    let v: u16 = 0xFFFF; // Constant high for 16 bits

    loop {
        // Push value to FIFO, output to pins 16-31
        sm0.tx().wait_push(v as u32).await;
        Timer::after_millis(50).await;
    }
}