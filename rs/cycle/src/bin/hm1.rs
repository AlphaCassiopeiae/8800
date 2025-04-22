#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::bind_interrupts;
use embassy_rp::peripherals::PIO0;
use embassy_rp::pio::program::pio_asm;
use embassy_rp::pio::{Common, Config as PioConfig, InterruptHandler as PioInterruptHandler, Pin, PinConfig, Pio, ShiftDirection, StateMachine};
use fixed::traits::ToFixed;
use fixed_macro::types::U56F8;
use embassy_time::{Timer};
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => PioInterruptHandler<PIO0>;
});

/// Configure PIO for generating continuous clock pulses
fn setup_clock_generator<'a>(
    pio: &mut Common<'a, PIO0>, 
    sm: &mut StateMachine<'a, PIO0, 0>,
    clk_pin: &Pin<'a, PIO0>,
) {
    // PIO program for generating continuous clock pulses
    // This program toggles the output pin at a fixed rate
    let prg = pio_asm!(
        // Main loop - toggle pin with delay
        "set pins, 1 [30]"
        ".wrap_target",
        "pull block"
        "set pins, 0 [30]",    // PIN 4
        "wait 0 gpio 30",
        "set pins, 1 [30]",    // Pin 4
        ".wrap",                // Wrap around to start (infinite loop)
    );

    // Configure the state machine
    let mut cfg = PioConfig::default();
    cfg.use_program(&pio.load_program(&prg.program), &[]);

    // Configure pins
    cfg.set_set_pins(&[clk_pin]);
    
    cfg.clock_divider = (U56F8!(125_000_000) / 10 / 200).to_fixed(); // 1MHz base clock
    
    sm.set_config(&cfg);
    
    // Set CLK pin as output
    sm.set_pin_dirs(embassy_rp::pio::Direction::Out, &[clk_pin]);
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    info!("Clock generator starting");

    // Create PIO state machine
    let Pio { mut common, mut sm0, .. } = Pio::new(p.PIO0, Irqs);
    
    // Setup pin for clock output (using PIN_4)
    let clk_pin = &common.make_pio_pin(p.PIN_29);
    
    // Configure PIO state machine
    setup_clock_generator(&mut common, &mut sm0, clk_pin);
    
    // Enable state machine - it will run autonomously from now on
    sm0.set_enable(true);

    info!("Clock generator running");
    
    // Nothing more for the CPU to do - the PIO runs the clock
    // independently and continuously
    loop {
        sm0.tx().wait_push(1).await;
        let level= sm0.tx().level();
        info!("sent push. level = {:02x}", level);
        Timer::after_millis(1000).await;

    }
}