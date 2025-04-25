#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::bind_interrupts;
use embassy_rp::gpio::{Level, Output};
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

fn setup_smemr_control<'a>(
    pio: &mut Common<'a, PIO1>, 
    sm: &mut StateMachine<'a, PIO1, 0>,
    smemr_pin: &Pin<'a, PIO1>,
) {
    // PIO program to control SMEMR# signal which is out of the lower 0-31 pin range
    let prg = pio_asm!(
        "set pindirs, 0",
        ".wrap_target",
        "wait 0 gpio 18",    // PIN 34 offset 16   // Deassert SMEMR# (inactive high)
        "irq 1",        // Wait for signal from main SM to start cycle
        "wait 1 gpio 18", //  ensure transaction done
        // "irq 2",
        ".wrap"
    );

    let mut cfg = PioConfig::default();
    cfg.use_program(&pio.load_program(&prg.program), &[]);
    
    cfg.set_in_pins(&[smemr_pin]); // even though now an in pin we set this to trigger the gpiobase. MUST BE SET
    
    // Set clock rate
    cfg.clock_divider = (U56F8!(125_000_000) / 20 / 200).to_fixed(); // 100kHz operation
    
    sm.set_pin_dirs(embassy_rp::pio::Direction::In, &[smemr_pin]);

    sm.set_config(&cfg);
}

// Define the memory contents (simple ROM for testing)
const MEMORY_SIZE: usize = 0x10000; // 64KB address space

/// Configure PIO for memory data output and control signal monitoring
fn setup_mem_data_control<'a>(
    pio: &mut Common<'a, PIO0>, 
    sm: &mut StateMachine<'a, PIO0, 1>,
    data_pins: &[&Pin<'a, PIO0>],
    addr_pins: &[&Pin<'a, PIO0>],
    xrdy_pin: &Pin<'a, PIO0>,
) {
    //control xrdy and data
    let prg = pio_asm!(
        ".wrap_target",
        "set pins, 0", // set default xrdy

        "pull block",

        // // Wait for address monitor to signal address is ready
        "in pins, 16",         // Sample the 16-bit address
        "push block",          // Push address to RX FIFO
        
        // // Pull data value to output to data bus
        "pull block",          // Get data value from TX FIFO
        "out pins, 8",         // Output data to pins

        // // Signal memory is ready by asserting XRDY
        "set pins, 1 [30]",    // Set XRDY high with delay for setup

        "irq 2",        // tell cpu set

        // Ready for next cycle
        ".wrap",           // Loop back for next transaction
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

    // Configure shift registers
    cfg.shift_in.auto_fill = false;
    cfg.shift_in.direction = ShiftDirection::Left;
    
    sm.set_config(&cfg);
    
    // Set data pins as outputs
    sm.set_pin_dirs(embassy_rp::pio::Direction::Out, data_pins);
    sm.set_pin_dirs(embassy_rp::pio::Direction::In, addr_pins);
    
    // Set XRDY pin as output
    sm.set_pin_dirs(embassy_rp::pio::Direction::Out, &[xrdy_pin]);
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    // Create PIO state machines
    let Pio { mut common, mut irq2, mut sm1, .. } = Pio::new(p.PIO0, Irqs);

    let mut pio1 = Pio::new(p.PIO1, Irqs);
    
    // Setup pins for address bus (A15-A0) (pins 4-19)
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

    // Setup pins for data bus (D7-D0) (pins 20-27)
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

    // Setup pin for XRDY signal
    let xrdy_pin = &common.make_pio_pin(p.PIN_4);
    let smemr_pin = &pio1.common.make_pio_pin(p.PIN_34);
    
    // Configure PIO state machines
    setup_mem_data_control(&mut common, &mut sm1, &data_pins, &addr_pins, xrdy_pin);
    setup_smemr_control(&mut pio1.common, &mut pio1.sm0, smemr_pin);
    
    // Create a simple memory array (for demo purposes)
    let memory = create_test_memory();
    
    // Enable state machines
    sm1.set_enable(true);
    pio1.sm0.set_enable(true);

    info!("Memory simulator ready");
    
    loop {
        // info!("Waiting for memory access request...");
        // Wait for address monitor to receive an address
        pio1.irq1.wait().await; // wait for smemr

        sm1.tx().wait_push(0).await;
        let address = sm1.rx().wait_pull().await as u32;
        // info!("recieved");
        // // print rx level
        // let level = sm1.rx().level();
        // info!("rx level = {:02x}", level);
        
        // Log the memory access
        info!("Memory read request: address 0x{:08X}", address);
        
        // Look up the data at this address
        let data = memory[address as usize];
        info!("Returning data: 0x{:02X}", data);
        
        // Push the data to the data control SM
        sm1.tx().wait_push(data as u32).await;
        irq2.wait().await; // wait for transaction to be done

        // pio1.sm0.tx().push(0); // push to smemr control to let it know we are done
        // print tx level
        // let level = sm1.tx().level();
        // info!("tx level = {:02x}", level);

        // info!("Data pushed to data control");
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
    memory[0x0000] = 0xAA;    // Address 0x0000 -> 0xAA
    memory[0x2008] = 0x5A;    // Address 0x2008 -> 0x5A (as in timing diagram)
    memory[0x3000] = 0x33;    // Address 0x3000 -> 0x33
    memory[0x4000] = 0x44;    // Address 0x4000 -> 0x44
    memory[0xFFFF] = 0xFF;    // Highest address -> 0xFF
    
    memory
}