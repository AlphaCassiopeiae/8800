#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::bind_interrupts;
use embassy_rp::gpio::{Input, Pull};
use embassy_rp::peripherals::{PIO0, PIO1};
use embassy_rp::pio::program::pio_asm;
use embassy_rp::pio::{Common, Config as PioConfig, InterruptHandler as PioInterruptHandler, Pio, Pin, ShiftDirection, StateMachine};
use embassy_time::Timer;
use fixed::traits::ToFixed;
use fixed_macro::types::U56F8;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => PioInterruptHandler<PIO0>;
    PIO1_IRQ_0 => PioInterruptHandler<PIO1>;
});

/// Configure PIO for SMEMR# control (pin 34)
fn setup_smemr<'a>(
    pio: &mut Common<'a, PIO1>, 
    sm: &mut StateMachine<'a, PIO1, 0>,
    smemr_pin: &Pin<'a, PIO1>,
) {
    // PIO program to control SMEMR# signal which is out of the lower 0-31 pin range
    let prg = pio_asm!(
        "set pindirs, 1",
        ".wrap_target",
        "set pins, 1",       // Deassert SMEMR# (inactive high)
        "pull block",        // Wait for signal from main SM to start cycle
        "set pins, 0",       // Assert SMEMR# (active low)
        "pull block",
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
fn setup_reads<'a>(
    pio: &mut Common<'a, PIO0>, 
    sm: &mut StateMachine<'a, PIO0, 1>,
    addr_pins: &[&Pin<'a, PIO0>],
    data_pins: &[&Pin<'a, PIO0>]
) {
    // PIO program for address output and data input
    let prg = pio_asm!(
        ".wrap_target",        
        // Wait for instruction from main CPU

        // set addr to output and data to input
        // "pull block",
        // "out pindirs, 24",    // set all data addr pins to output and data pins to input

        "pull block",         // Get address from TX FIFO
        
        "out pins, 16",       // Output address to pins
        
        "wait 1 gpio 4",      // Wait for XRDY high
        
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
    cfg.shift_out.auto_fill = false;
    cfg.shift_out.direction = ShiftDirection::Right;
    cfg.shift_in.auto_fill = false;
    cfg.shift_in.direction = ShiftDirection::Left;
    
    sm.set_config(&cfg);

    // set addr pin direction
    sm.set_pin_dirs(embassy_rp::pio::Direction::Out, addr_pins);
    // sm.set_pin_dirs(embassy_rp::pio::Direction::In, data_pins); // not needed
}

// Configure PIO for address output and data output (write cycle, with MWRT# control)
fn setup_writes<'a>(
    pio: &mut Common<'a, PIO0>, 
    sm: &mut StateMachine<'a, PIO0, 2>,
    data_addr_pins: &[&Pin<'a, PIO0>],
    mwrt_pin: &Pin<'a, PIO0>,
) {
    // PIO program for address output, data output, and MWRT# control
    let prg = pio_asm!(
        ".wrap_target",

        "set pins, 1",        // Deassert MWRT# (inactive high)

        // set all data addr pins to output
        "pull block",
        "out pindirs, 24",    

        "pull block",         // Get address from TX FIFO
        "out pins, 24",       // Output address to pins

        "set pins, 0",        // Assert MWRT# (active low)

        "wait 1 gpio 4",      // Wait for XRDY high (write complete)
        "irq 2",              // notify write complete

        // set data to input and addr to output
        "pull block",
        "out pindirs, 24",    // set all data addr pins to output and data pins to input

        ".wrap"
    );

    let mut cfg = PioConfig::default();
    cfg.use_program(&pio.load_program(&prg.program), &[]);
    cfg.set_out_pins(data_addr_pins);
    cfg.set_set_pins(&[mwrt_pin]);
    cfg.clock_divider = (U56F8!(125_000_000) / 20 / 200).to_fixed(); // 100kHz operation
    cfg.shift_out.auto_fill = false;
    cfg.shift_out.direction = ShiftDirection::Right;

    sm.set_config(&cfg);

    // set swrite pin as output
    sm.set_pin_dirs(embassy_rp::pio::Direction::Out, &[mwrt_pin]);
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    // Create PIO state machines
    let Pio { mut common, mut irq1, mut irq2, mut sm1, mut sm2, .. } = Pio::new(p.PIO0, Irqs);
    
    // make a new pio block (pio0) but don't deconstruct
    let mut pio1 = Pio::new(p.PIO1, Irqs);
    

    // Setup pins for address bus (A15-A0)
    let data_addr_pins = [
        // data pins
        &common.make_pio_pin(p.PIN_8),
        &common.make_pio_pin(p.PIN_9),
        &common.make_pio_pin(p.PIN_10),
        &common.make_pio_pin(p.PIN_11),
        &common.make_pio_pin(p.PIN_12),
        &common.make_pio_pin(p.PIN_13),
        &common.make_pio_pin(p.PIN_14),
        &common.make_pio_pin(p.PIN_15),
        // address pins
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

    // Take slice of address pins (A15-A0)
    let addr_pins = &data_addr_pins[8..];
    // Take slice of data pins (D7-D0)
    let data_pins = &data_addr_pins[0..8];

    // Setup pin for SMEMR# control and xrdy
    let smemr_pin = &pio1.common.make_pio_pin(p.PIN_34);  // SMEMR# pin (output, active low)
    common.make_pio_pin(p.PIN_4); // XRDY pin (input, active high)
    let mwrt_pin = &common.make_pio_pin(p.PIN_6); // active low


    // Configure PIO state machines
    setup_smemr(&mut pio1.common, &mut pio1.sm0, smemr_pin);
    setup_reads(&mut common, &mut sm1, &addr_pins, &data_pins);
    
    setup_writes(&mut common, &mut sm2, &data_addr_pins, mwrt_pin);


    // Enable state machines
    pio1.sm0.set_enable(true);
    sm1.set_enable(true);
    sm2.set_enable(true); // write machine

    // Define test addresses and data for our read/write operations
    let test_addresses = [
        0xF0F0, // Address 0x2008 (as shown in timing diagram)
        0xF0F0, // Another test address
        0x00FF, // Another test address
        0xFFFF, // Highest possible address
    ];
    let test_data = [
        0x22, // Test data
        0x31, // Test data
        0x41, // Test data
        0xF1, // Test data
    ];

    let mut current_address = 0;

    info!("Starting CPU read/write transaction loop with split state machines");

    let state = 0;

    let mut stop = Input::new(p.PIN_3, Pull::None);


    loop {

        // if
         
        // start transaction
        if current_address % 2 == 0 {
            let index = current_address / 2;
            let address = test_addresses[index];
            
            // Start a new read transaction
            info!("Initiating read from address 0x{:08X}", address);

            // set address to output
            // sm1.tx().wait_push(0x00FFFF).await;
            
            // Push address to address/data state machine
            sm1.tx().wait_push(address as u32).await;

            
            pio1.sm0.tx().wait_push(0).await; // pull smemr low
            
            // Read the data received
            let data = sm1.rx().wait_pull().await as u32;
            info!("Read data 0x{:02X} from address 0x{:08X}", data, address);

            
            pio1.sm0.tx().wait_push(0).await; // pull smemr high
        }
        else {
            // Write cycle

            // set address to output and data to output pindirs
            sm2.tx().wait_push(0xFFFFFF).await;

            let idx = current_address / 2 % test_addresses.len();
            let address = test_addresses[idx];
            let data = test_data[idx];
            info!("Initiating write of 0x{:02X} to address 0x{:08X}", data, address);

            let value = (address << 8) | (data as u32);

            sm2.tx().wait_push(value as u32).await;

            irq2.wait().await;
            // set address to output and data to input pindirs
            sm2.tx().wait_push(0xFFFF00).await;
            info!("Write complete to address 0x{:08X}", address);
        }

        current_address = (current_address + 1) % (test_addresses.len() * 2);



        // Wait before next transaction
        Timer::after_millis(8000).await;
    }
}