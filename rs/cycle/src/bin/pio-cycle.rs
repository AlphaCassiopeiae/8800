#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::bind_interrupts;
use embassy_rp::gpio::{Level, Output};
use embassy_rp::peripherals::PIO0;
use embassy_rp::pio::program::pio_asm;
use embassy_rp::pio::{Common, Config as PioConfig, InterruptHandler as PioInterruptHandler, Pio, Pin, ShiftDirection, StateMachine};
use embassy_time::Timer;
use fixed::traits::ToFixed;
use fixed_macro::types::U56F8;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => PioInterruptHandler<PIO0>;
});

/// Configure PIO for SMEMR# control (pin 34)
fn setup_smemr_control<'a>(
    pio: &mut Common<'a, PIO0>, 
    sm: &mut StateMachine<'a, PIO0, 0>,
    smemr_pin: &Pin<'a, PIO0>,
) {
    // PIO program to control SMEMR# signal
    let prg = pio_asm!(
        "set pins, 1",       // SMEMR# default high (inactive)
        ".wrap_target",
        "wait 1 irq 1",        // Wait for signal from main SM to start cycle
        "set pins, 0",       // Assert SMEMR# (active low)
        "wait 1 irq 2",        // Wait for signal to end cycle
        "set pins, 1",       // Deassert SMEMR# (inactive high)
        ".wrap"
    );

    let mut cfg = PioConfig::default();
    cfg.use_program(&pio.load_program(&prg.program), &[]);
    
    // Configure SMEMR pin for output
    cfg.set_set_pins(&[smemr_pin]);
    
    // Set clock rate
    cfg.clock_divider = (U56F8!(125_000_000) / 20 / 200).to_fixed(); // 100kHz operation
    
    sm.set_config(&cfg);
    
    // Set SMEMR pin as output
    sm.set_pin_dirs(embassy_rp::pio::Direction::Out, &[smemr_pin]);
}

/// Configure PIO for address output and data input
fn setup_addr_data_control<'a>(
    pio: &mut Common<'a, PIO0>, 
    sm: &mut StateMachine<'a, PIO0, 1>,
    addr_pins: &[&Pin<'a, PIO0>],
    data_pins: &[&Pin<'a, PIO0>]
) {
    // PIO program for address output and data input
    let prg = pio_asm!(
        ".wrap_target",        
        // Wait for instruction from main CPU
        "pull block",         // Get address from TX FIFO
        
        "out pins, 16",       // Output address to pins
        
        "irq 1",              // Signal SMEMR# SM to assert SMEMR#
        
        "wait 1 gpio 4",      // Wait for XRDY high
        "irq 2",              // Signal SMEMR# SM to deassert SMEMR#
        
        "in pins, 8",         // Read 8-bit data from pins
        "push block",         // Push data to RX FIFO
        ".wrap"
    );

    let mut cfg = PioConfig::default();
    cfg.use_program(&pio.load_program(&prg.program), &[]);
    
    // Configure pins
    cfg.set_out_pins(addr_pins);
    cfg.set_in_pins(data_pins);
    
    // Set clock rate
    cfg.clock_divider = (U56F8!(125_000_000) / 20 / 200).to_fixed(); // 100kHz operation
    
    // Configure shift registers
    cfg.shift_out.auto_fill = true;
    cfg.shift_out.direction = ShiftDirection::Right;
    cfg.shift_in.auto_fill = true;
    cfg.shift_in.direction = ShiftDirection::Left;
    
    sm.set_config(&cfg);

    // Set pin directions
    sm.set_pin_dirs(embassy_rp::pio::Direction::Out, addr_pins);
    sm.set_pin_dirs(embassy_rp::pio::Direction::In, data_pins);
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    // Create PIO state machines
    let Pio { mut common, mut sm0, mut sm1, .. } = Pio::new(p.PIO0, Irqs);
    
    // Setup pins for address bus (A15-A0)
    let addr_pins = [
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

    // Setup pins for data bus (D7-D0)
    let data_pins = [
        &common.make_pio_pin(p.PIN_8),
        &common.make_pio_pin(p.PIN_9),
        &common.make_pio_pin(p.PIN_10),
        &common.make_pio_pin(p.PIN_11),
        &common.make_pio_pin(p.PIN_12),
        &common.make_pio_pin(p.PIN_13),
        &common.make_pio_pin(p.PIN_14),
        &common.make_pio_pin(p.PIN_15),
    ];

    // Setup pin for SMEMR# control
    let smemr_pin = &common.make_pio_pin(p.PIN_34);

    // Configure PIO state machines
    setup_smemr_control(&mut common, &mut sm0, smemr_pin);
    setup_addr_data_control(&mut common, &mut sm1, &addr_pins, &data_pins);
    
    // Enable state machines
    sm0.set_enable(true);
    sm1.set_enable(true);

    // Define test addresses for our read operations
    let test_addresses = [
        0x2008, // Address 0x2008 (as shown in timing diagram)
        0x3000, // Another test address
        0x4000, // Another test address
        0xFFFF, // Highest possible address
    ];
    
    let mut current_address = 0;
    
    info!("Starting CPU read transaction loop with split state machines");
    
    loop {
        // Get the next address to read
        let address = test_addresses[current_address];
        current_address = (current_address + 1) % test_addresses.len();
        
        // Start a new read transaction
        info!("Initiating read from address 0x{:08X}", address);
        
        // Push address to address/data state machine
        sm1.tx().wait_push(address as u32).await;

        // Read the data received
        let data = sm1.rx().wait_pull().await as u32;
        info!("Read data 0x{:02X} from address 0x{:08X}", data, address);
        
        // Wait before next transaction
        Timer::after_millis(500).await;
    }
}