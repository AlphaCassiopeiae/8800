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

// Define the memory contents (simple ROM for testing)
const MEMORY_SIZE: usize = 0x10000; // 64KB address space

/// Configure PIO for memory read operations
fn setup_mem_read_control<'a>(
    pio: &mut Common<'a, PIO0>, 
    sm: &mut StateMachine<'a, PIO0, 0>,
    data_pins: &[&Pin<'a, PIO0>],
    addr_pins: &[&Pin<'a, PIO0>],
    xrdy_pin: &Pin<'a, PIO0>,
) {
    let prg = pio_asm!(
        ".wrap_target",
        "set pins, 0", // set default xrdy low

        "wait 0 gpio 34",      // Wait for SMEMR# to be asserted

        "in pins, 16",         // Sample the 16-bit address
        "push block",          // Push address to RX FIFO
        
        "pull block",          // Get data value from TX FIFO
        "out pins, 8",         // Output data to pins

        "set pins, 1 [30]",    // Set XRDY high with delay for setup
        
        //"wait 1 gpio 34",      // Wait for SMEMR# to be deasserted
        
        ".wrap"
    );

    let mut cfg = PioConfig::default();
    cfg.use_program(&pio.load_program(&prg.program), &[]);
    
    // Configure pins
    cfg.set_out_pins(data_pins);
    cfg.set_in_pins(addr_pins);
    cfg.set_set_pins(&[xrdy_pin]);
    
    // Set clock rate
    cfg.clock_divider = (U56F8!(125_000_000) / 20 / 200).to_fixed(); // 100kHz operation
    
    // Configure shift registers
    cfg.shift_out.auto_fill = false;
    cfg.shift_out.direction = ShiftDirection::Right;
    cfg.shift_in.auto_fill = false;
    cfg.shift_in.direction = ShiftDirection::Left;
    
    sm.set_config(&cfg);
    
    // Set pin directions
    sm.set_pin_dirs(embassy_rp::pio::Direction::Out, data_pins);
    sm.set_pin_dirs(embassy_rp::pio::Direction::In, addr_pins);
    sm.set_pin_dirs(embassy_rp::pio::Direction::Out, &[xrdy_pin]);
}

/// Configure PIO for memory write operations
fn setup_mem_write_control<'a>(
    pio: &mut Common<'a, PIO0>, 
    sm: &mut StateMachine<'a, PIO0, 1>,
    data_addr_pins: &[&Pin<'a, PIO0>],
    xrdy_pin: &Pin<'a, PIO0>,
) {
    let prg = pio_asm!(
        ".wrap_target",
        "set pins, 0", // set default xrdy low
        "out pindirs, 1",
        "wait 0 gpio 6",       // Wait for MWRT# to be asserted

        "in pins, 24",         // Sample the 16-bit address
        "push block",          // Push address to RX FIFO

        "set pins, 1 [30]",    // Set XRDY high with delay for setup
        
        "wait 1 gpio 6",       // Wait for MWRT# to be deasserted
        
        ".wrap"
    );

    let mut cfg = PioConfig::default();
    cfg.use_program(&pio.load_program(&prg.program), &[]);
    
    // Configure pins
    cfg.set_in_pins(data_addr_pins);
    cfg.set_set_pins(&[xrdy_pin]);
    
    // Set clock rate
    cfg.clock_divider = (U56F8!(125_000_000) / 20 / 200).to_fixed(); // 100kHz operation
    
    // Configure shift registers
    cfg.shift_in.auto_fill = false;
    cfg.shift_in.direction = ShiftDirection::Left;
    
    sm.set_config(&cfg);

    
    // Set pin directions
    sm.set_pin_dirs(embassy_rp::pio::Direction::Out, &[xrdy_pin]);
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    // Create PIO state machines
    let Pio { mut common, mut sm0, mut sm1, .. } = Pio::new(p.PIO0, Irqs);
    
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

    // Setup pins for control signals
    let xrdy_pin = &common.make_pio_pin(p.PIN_4);
    common.make_pio_pin(p.PIN_34); // read pin
    common.make_pio_pin(p.PIN_6); // write pin
    
    // Configure PIO state machines
    setup_mem_read_control(&mut common, &mut sm0, &data_pins, &addr_pins, xrdy_pin);
    //setup_mem_write_control(&mut common, &mut sm1, &data_addr_pins, xrdy_pin);
    
    // Create a mutable memory array
    let mut memory = create_test_memory();
    
    // Enable state machines
    // xrdy driven by 2 state machines
    sm0.set_enable(true);
    // sm1.set_enable(false);

    info!("Memory simulator ready for read and write operations");


    loop {
        // Check for read operations (from SM0)
        if !sm0.rx().empty() {
            let address = sm0.rx().wait_pull().await as usize;
            
            // Log the memory read access
            info!("Memory read request: address 0x{:04X}", address);
            
            // Look up the data at this address
            let data = memory[address];

            info!("Returning data: 0x{:02X}", data);
            // Push the data to the read control SM
            sm0.tx().wait_push(data as u32).await;
        }
        
        // Check for write operations (from SM1)
        // if !sm1.rx().empty() {
        //     let write = sm1.rx().wait_pull().await as u32;

        //     // split the address and data
        //     let address = (write >> 8) as usize; // Address is in the upper 8 bits
        //     let data = (write & 0xFF) as u8; // Data is in the lower 8 bits
            
        //     // Log the memory write access
        //     info!("Memory write request: address 0x{:04X}, data 0x{:02X}", address, data);
            
        //     // Update the memory
        //     memory[address] = data;
        //     info!("Memory updated");
        // }
        
        // Small yield to avoid tight loop
        Timer::after_micros(10).await;
    }
}

/// Create a test memory array with some predefined values
fn create_test_memory() -> [u8; MEMORY_SIZE] {
    let mut memory = [0u8; MEMORY_SIZE];
    
    // Pattern for easy visual identification of address
    for i in 0..MEMORY_SIZE {
        memory[i] = (i & 0xFF) as u8;
    }
    
    // Special values for our test addresses
    memory[0x70ff] = 0xAA;    // Address 0x0000 -> 0xAA
    memory[0x0000] = 0xAA;    // Address 0x2008 -> 0x5A (as in timing diagram)
    memory[0x3000] = 0x33;    // Address 0x3000 -> 0x33
    memory[0x4000] = 0x44;    // Address 0x4000 -> 0x44
    memory[0xFFFF] = 0xFF;    // Highest address -> 0xFF
    
    memory
}