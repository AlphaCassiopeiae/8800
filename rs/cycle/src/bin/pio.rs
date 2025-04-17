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

/// Configure PIO for CPU Control signals (SMEMR#, pDBIN)
fn setup_cpu_control<'a>(
    pio: &mut Common<'a, PIO0>, 
    sm: &mut StateMachine<'a, PIO0, 1>,
    control_pins: &[&Pin<'a, PIO0>],
    addr_pins: &[&Pin<'a, PIO0>],
    data_pins: &[&Pin<'a, PIO0>]
) {
    // CPU Control Signals PIO program
    let prg = pio_asm!(
        // T1: Start of cycle - all signals inactive
        ".wrap_target",
        "set pins, 1 [30]",      // SMEMR# high (2nd bit)
        // "pull block",
        // "out pins, 16",
        "set pins, 0 [30]",
        ".wrap"
        
        // ".wrap_target",
        // // Wait for instruction from main CPU
        // "pull block",

        // // send addr
        // "out pins, 16",
        
        // // T2: Assert SMEMR# (active low)
        // "set pins, 0b00 [30]", // SMEMR# low, pDBIN still high
        
        // // Wait for XRDY to indicate data is ready
        // "wait 0 gpio 30", // Wait for XRDY low
        
        // "in pins, 8",          // Read 8-bit data from pins
        // "push block",          // Push data to RX FIFO

        // // End of T3: De-assert all signals
        // "set pins, 0b01",      // Return all signals to inactive
        // ".wrap"
    );

    let mut cfg = PioConfig::default();
    cfg.use_program(&pio.load_program(&prg.program), &[]);
    
    // Configure control pins
    cfg.set_set_pins(control_pins);

    // // Configure address pins for output
    // cfg.set_out_pins(addr_pins);

    // // Configure data pins for input (during read)
    // cfg.set_in_pins(data_pins);
    
    // Set clock rate
    cfg.clock_divider = (U56F8!(125_000_000) / 20 / 200).to_fixed(); // 100kHz operation
    
    // Configure shift registers
    cfg.shift_out.auto_fill = true;
    cfg.shift_out.direction = ShiftDirection::Right;
    cfg.shift_in.auto_fill = true;
    cfg.shift_in.direction = ShiftDirection::Right;

    // Set address pins as outputs
    sm.set_pin_dirs(embassy_rp::pio::Direction::Out, addr_pins);
    
    // Set data pins as inputs for reading
    sm.set_pin_dirs(embassy_rp::pio::Direction::In, data_pins);

    sm.set_config(&cfg);
    
    // Set control pins as outputs
    sm.set_pin_dirs(embassy_rp::pio::Direction::Out, control_pins);
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    // Create PIO state machines
    let Pio { mut common, mut sm1, .. } = Pio::new(p.PIO0, Irqs);
    
    // Setup pins for address bus (A15-A0) (pins 4-19)
    let addr_pins = [
        &common.make_pio_pin(p.PIN_4),
        &common.make_pio_pin(p.PIN_5),
        &common.make_pio_pin(p.PIN_6),
        &common.make_pio_pin(p.PIN_7),
        &common.make_pio_pin(p.PIN_8),
        &common.make_pio_pin(p.PIN_9),
        &common.make_pio_pin(p.PIN_10),
        &common.make_pio_pin(p.PIN_11),
        &common.make_pio_pin(p.PIN_12),
        &common.make_pio_pin(p.PIN_13),
        &common.make_pio_pin(p.PIN_14),
        &common.make_pio_pin(p.PIN_15),
        &common.make_pio_pin(p.PIN_16),
        &common.make_pio_pin(p.PIN_17),
        &common.make_pio_pin(p.PIN_18),
        &common.make_pio_pin(p.PIN_19),
    ];

    // Setup pins for data bus (D7-D0) (pins 20-27)
    let data_pins = [
        &common.make_pio_pin(p.PIN_20),
        &common.make_pio_pin(p.PIN_21),
        &common.make_pio_pin(p.PIN_22),
        &common.make_pio_pin(p.PIN_23),
        &common.make_pio_pin(p.PIN_24),
        &common.make_pio_pin(p.PIN_25),
        &common.make_pio_pin(p.PIN_26),
        &common.make_pio_pin(p.PIN_27),
    ];

    // Setup pins for control signals
    let control_pins = [
        &common.make_pio_pin(p.PIN_34), // SMEMR#
    ];

    // Configure PIO state machines
    setup_cpu_control(&mut common, &mut sm1, &control_pins, &addr_pins, &data_pins);
    
    // Enable state machines
    sm1.set_enable(true);

    // Define test addresses for our read operations
    let test_addresses = [
        0x2008, // Address 0x2008 (as shown in timing diagram)
        0x3000, // Another test address
        0x4000, // Another test address
        0xFFFF, // Highest possible address
    ];
    
    let mut current_address = 0;
    
    info!("Starting CPU read transaction loop");
    
    loop {
        // Get the next address to read
        // let address = test_addresses[current_address];
        // current_address = (current_address + 1) % test_addresses.len();
        info!("hi");
        // // Start a new read transaction
        // info!("Initiating read from address 0x{:04X}", address);
        
        // // Push address to address/data state machine
        // sm1.tx().wait_push(address as u32).await;

        // info!("sent data");

        // // Read the data received
        // let data = sm1.rx().wait_pull().await as u8;
        // info!("Read data 0x{:02X} from address 0x{:04X}", data, address);
        
        // Wait before next transaction
        Timer::after_millis(1000).await;
    }
}