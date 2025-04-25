#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::bind_interrupts;
use embassy_rp::peripherals::PIO0;
use embassy_rp::pio::program::pio_asm;
use embassy_rp::pio::{Common, Config as PioConfig, FifoJoin, InterruptHandler as PioInterruptHandler, Pio, Pin, StateMachine};
use embassy_time::{Timer};
use fixed::traits::ToFixed;
use fixed_macro::types::U56F8;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => PioInterruptHandler<PIO0>;
});

/// Configure PIO to detect rising edges on the clock signal and acknowledge
fn setup_edge_detector<'a>(
    pio: &mut Common<'a, PIO0>, 
    sm: &mut StateMachine<'a, PIO0, 0>,
    input_pin: &Pin<'a, PIO0>,
    output_pin: &Pin<'a, PIO0>,
) {
    // PIO program for detecting rising edges on input pin and acknowledging
    let prg = pio_asm!(
        // Initial setup - set output pin direction
        "set pindirs, 1",       // Set output pin as output
        "set pins, 1",          // Initialize output pin low
        
        // Main detection loop
        ".wrap_target",
        // wait message
        "wait 0 gpio 4",        // Wait for PIN_4 (clock) to go high
        "irq 3",                // Signal IRQ to CPU that a rising edge was detected
        // send acknowledge
        "set pins, 0 [30]",     // Set output PIN_5 high as acknowledgment with delay
        "set pins, 1 [30]",
        ".wrap",                // Loop back
    );

    // Configure the state machine
    let mut cfg = PioConfig::default();
    cfg.use_program(&pio.load_program(&prg.program), &[]);
    
    // Configure pins
    cfg.set_set_pins(&[output_pin]);
    
    // Configure clock
    cfg.clock_divider = (U56F8!(125_000_000) / 10 / 200).to_fixed(); // 1/2 MHz
    
    // Set output pin direction
    sm.set_pin_dirs(embassy_rp::pio::Direction::Out, &[output_pin]);
    
    sm.set_config(&cfg);
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    info!("Clock signal detector starting");

    // Create PIO state machine
    let Pio { mut common, mut irq3, mut sm0, .. } = Pio::new(p.PIO0, Irqs);
    
    // Configure PIN_4 as input for the clock signal
    let clk_pin = &common.make_pio_pin(p.PIN_4);
    let ack_pin = &common.make_pio_pin(p.PIN_5); // PIN_5 for acknowledgment
    
    // Configure PIO state machine
    setup_edge_detector(&mut common, &mut sm0, clk_pin, ack_pin);
    
    // Enable state machine
    sm0.set_enable(true);

    info!("Clock signal detector running");
    
    // Process rising edges detected by PIO
    loop {
        // Wait for the IRQ indicating a clock pulse was detected
        irq3.wait().await;
        info!("Clock pulse received");
        
        // Process the received signal here
        // ...your processing logic...
        
        // Small delay to avoid tight loop, but not too long to ensure responsiveness
        Timer::after_millis(10).await;
    }
}