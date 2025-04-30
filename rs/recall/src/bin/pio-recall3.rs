#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::bind_interrupts;
use embassy_rp::gpio::{Input, Level, Output, Pull};
use embassy_rp::peripherals::{PIO0, PIO1};
use embassy_rp::pio::program::pio_asm;
use embassy_rp::pio::{
    Common, Config as PioConfig, InterruptHandler as PioInterruptHandler, Pin, Pio, ShiftDirection,
    StateMachine,
};
use embassy_time::Timer;
use fixed::traits::ToFixed;
use fixed_macro::types::U56F8;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => PioInterruptHandler<PIO0>;
    PIO1_IRQ_0 => PioInterruptHandler<PIO1>;
});

fn setup_smemr<'a>(
    pio: &mut Common<'a, PIO1>,
    sm: &mut StateMachine<'a, PIO1, 0>,
    smemr_pin: &Pin<'a, PIO1>,
) {
    // PIO program to control SMEMR# signal which is out of the lower 0-31 pin range
    let prg = pio_asm!(
        "set pindirs, 0",
        ".wrap_target",
        "wait 0 gpio 18", // PIN 34 offset 16   // Deassert SMEMR# (inactive high)
        "push block",     // Wait for signal from main SM to start cycle
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
fn setup_reads<'a>(
    pio: &mut Common<'a, PIO0>,
    sm: &mut StateMachine<'a, PIO0, 1>,
    data_pins: &[&Pin<'a, PIO0>],
    addr_pins: &[&Pin<'a, PIO0>],
    xrdy_pin: &Pin<'a, PIO0>,
) {
    //control xrdy and data
    let prg = pio_asm!(
        ".wrap_target",
        "set pindirs, 0", // set xrdy input
        "pull block",  // wait for smemr to be asserted
        "set pindirs, 1", // set xrdy output
        "set pins, 0", // set default xrdy
        // set data pins to output
        "pull block",
        "out pindirs, 8",
        // // Wait for address monitor to signal address is ready
        "in pins, 16", // Sample the 16-bit address
        "push block",  // Push address to RX FIFO
        // // Pull data value to output to data bus
        "pull block",       // Get data value from TX FIFO
        "out pins, 8 [10]", // Output data to pins
        // // Signal memory is ready by asserting XRDY
        "set pins, 1 [10]", // Set XRDY high with delay for setup
        "irq 2",            // tell cpu set
        // set data pins to input
        "pull block",
        "out pindirs, 8",
        // Ready for next cycle
        ".wrap", // Loop back for next transaction
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

/// Configure PIO for memory write operations
fn setup_writes<'a>(
    pio: &mut Common<'a, PIO0>,
    sm: &mut StateMachine<'a, PIO0, 0>,
    data_addr_pins: &[&Pin<'a, PIO0>],
    xrdy_pin: &Pin<'a, PIO0>,
) {
    let prg = pio_asm!(
        ".wrap_target",
        "set pindirs, 0", // set xrdy input
        "wait 0 gpio 6 [30]", // Wait for MWRT# to be asserted
        "set pindirs, 1", // set xrdy output
        "set pins, 0", // set default xrdy low
        // notify main thread to start write cycle
        "push block",
        // set addr and data to input
        "pull block",
        "out pindirs, 24",
        "in pins, 24", // Sample the 16-bit address
        "push block",  // Push address to RX FIFO
        "set pins, 1 [30]", // Set XRDY high with delay for setup
        // could check from main thread that it was actually written?
        "wait 1 gpio 6", // Wait for MWRT# to be deasserted
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
    let Pio {
        mut common,
        mut irq2,
        mut sm0,
        mut sm1,
        ..
    } = Pio::new(p.PIO0, Irqs);

    let mut pio1 = Pio::new(p.PIO1, Irqs);

    // Setup pins for address bus (A15-A0) (pins 4-19)
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

    // Setup pin for XRDY signal
    let xrdy_pin = &common.make_pio_pin(p.PIN_4);
    let smemr_pin = &pio1.common.make_pio_pin(p.PIN_34);
    let panel = Input::new(p.PIN_1, Pull::None);
    common.make_pio_pin(p.PIN_6); // write pin

    // Configure PIO state machines
    setup_reads(&mut common, &mut sm1, &data_pins, &addr_pins, xrdy_pin);
    setup_smemr(&mut pio1.common, &mut pio1.sm0, smemr_pin);

    setup_writes(&mut common, &mut sm0, &data_addr_pins, xrdy_pin);

    // Create a simple memory array (for demo purposes)
    let mut memory = create_test_memory();

    // Enable state machines
    sm0.set_enable(true);
    sm1.set_enable(true);
    pio1.sm0.set_enable(true);

    info!("Memory simulator ready");

    loop {
        // only respond to requests when the panel is low
        if panel.is_low() {
            // reads
            if !pio1.sm0.rx().empty() {
                // if smemr
                pio1.sm0.rx().wait_pull().await; // pop notification

                sm1.tx().wait_push(0).await; // notify state machine to start read

                // set the data to output
                sm1.tx().wait_push(0xFF).await;

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

                // set the data to input
                sm1.tx().wait_push(0x00).await;

                // pio1.sm0.tx().push(0); // push to smemr control to let it know we are done
                // print tx level
                // let level = sm1.tx().level();
                // info!("tx level = {:02x}", level);

                // info!("Data pushed to data control");
            }

            // writes
            if !sm0.rx().empty() {
                // pop notification
                sm0.rx().wait_pull().await;

                // set addr and data to input
                sm0.tx().wait_push(0x000000).await; // set addr and data to input

                let write = sm0.rx().wait_pull().await as u32;

                // split the address and data
                let address = (write >> 8) as usize; // Address is in the upper 8 bits
                let data = (write & 0xFF) as u8; // Data is in the lower 8 bits

                // Log the memory write access
                info!(
                    "Memory write request: address 0x{:04X}, data 0x{:02X}",
                    address, data
                );

                // Update the memory
                memory[address] = data;
                info!("Memory updated");

                // could send back to state machine it was written
            }
        }
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
    memory[0x0000] = 0xAA; // Address 0x0000 -> 0xAA
    memory[0x2008] = 0x5A; // Address 0x2008 -> 0x5A (as in timing diagram)
    memory[0x3000] = 0x33; // Address 0x3000 -> 0x33
    memory[0x4000] = 0x44; // Address 0x4000 -> 0x44
    memory[0xFFFF] = 0xFF; // Highest address -> 0xFF

    memory
}
