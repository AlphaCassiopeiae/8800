#![no_std]

use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::bind_interrupts;
use embassy_rp::peripherals::{PIO0, PIO1};
use embassy_rp::pio::program::pio_asm;
use embassy_rp::pio::{Common, Config as PioConfig, InterruptHandler as PioInterruptHandler, Pio, Pin, ShiftDirection, StateMachine, Irq};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use embassy_time::Timer;
use fixed::traits::ToFixed;
use fixed_macro::types::U56F8;
use static_cell::StaticCell;

// import panic
use core::panic;

// Define channels for communication
pub type RamChannel = Channel<CriticalSectionRawMutex, RamRequest, 16>;
pub type RamResponseChannel = Channel<CriticalSectionRawMutex, RamResponse, 16>;

// Request and response types
#[derive(Debug)]
pub enum RamRequest {
    Read(usize),
    Write(usize, u8),
}

#[derive(Debug)]
pub enum RamResponse {
    ReadResult(u8),
    WriteComplete,
}

// Bind interrupts for PIO hardware
bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => PioInterruptHandler<PIO0>;
    PIO1_IRQ_0 => PioInterruptHandler<PIO1>;
});

// Statically allocated channels
static RAM_REQUESTS: StaticCell<RamChannel> = StaticCell::new();
static RAM_RESPONSES: StaticCell<RamResponseChannel> = StaticCell::new();

// Client API for accessing RAM
pub struct RamClient {
    requests: &'static RamChannel,
    responses: &'static RamResponseChannel,
}

impl RamClient {
    pub fn new(requests: &'static RamChannel, responses: &'static RamResponseChannel) -> Self {
        Self { requests, responses }
    }

    pub async fn read(&self, address: usize) -> u8 {
        // Send read request and await response
        self.requests.send(RamRequest::Read(address)).await;
        match self.responses.receive().await {
            RamResponse::ReadResult(data) => data,
            _ => panic!("Unexpected response type"),
        }
    }

    pub async fn write(&self, address: usize, data: u8) -> () {
        // Send write request and await completion
        self.requests.send(RamRequest::Write(address, data)).await;
        match self.responses.receive().await {
            RamResponse::WriteComplete => (),
            _ => panic!("Unexpected response type"),
        }
    }
}

// PIO setup functions 
fn setup_smemr<'a>(
    pio: &mut Common<'a, PIO1>, 
    sm: &mut StateMachine<'a, PIO1, 0>,
    smemr_pin: &Pin<'a, PIO1>,
) {
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
    cfg.set_set_pins(&[smemr_pin]);
    cfg.clock_divider = (U56F8!(125_000_000) / 20 / 200).to_fixed(); // 100kHz operation
    
    sm.set_config(&cfg);
    sm.set_pin_dirs(embassy_rp::pio::Direction::Out, &[smemr_pin]);
}

fn setup_reads<'a>(
    pio: &mut Common<'a, PIO0>, 
    sm: &mut StateMachine<'a, PIO0, 1>,
    addr_pins: &[&Pin<'a, PIO0>],
    data_pins: &[&Pin<'a, PIO0>]
) {
    let prg = pio_asm!(
        ".wrap_target",
        "pull block",         // Get address from TX FIFO
        "out pins, 16",       // Output address to pins
        "wait 1 gpio 4",      // Wait for XRDY high
        "in pins, 8",         // Read 8-bit data from pins
        "push block",         // Push data to RX FIFO
        ".wrap"
    );

    let mut cfg = PioConfig::default();
    cfg.use_program(&pio.load_program(&prg.program), &[]);
    cfg.set_out_pins(addr_pins);
    cfg.set_in_pins(data_pins);
    cfg.clock_divider = (U56F8!(125_000_000) / 20 / 200).to_fixed(); // 100kHz operation
    cfg.shift_out.auto_fill = false;
    cfg.shift_out.direction = ShiftDirection::Right;
    cfg.shift_in.auto_fill = false;
    cfg.shift_in.direction = ShiftDirection::Left;
    
    sm.set_config(&cfg);
    sm.set_pin_dirs(embassy_rp::pio::Direction::Out, addr_pins);
}

fn setup_writes<'a>(
    pio: &mut Common<'a, PIO0>, 
    sm: &mut StateMachine<'a, PIO0, 2>,
    data_addr_pins: &[&Pin<'a, PIO0>],
    mwrt_pin: &Pin<'a, PIO0>,
) {
    let prg = pio_asm!(
        ".wrap_target",
        "set pins, 1",        // Deassert MWRT# (inactive high)
        "pull block",         // Get pin direction configuration
        "out pindirs, 24",    // Set pin directions
        "pull block",         // Get address/data
        "out pins, 24",       // Output address/data to pins
        "set pins, 0",        // Assert MWRT# (active low)
        "wait 1 gpio 4",      // Wait for XRDY high (write complete)
        "irq 2",              // Notify write complete
        "pull block",         // Get pin direction for cleanup
        "out pindirs, 24",    // Restore pin directions
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
    sm.set_pin_dirs(embassy_rp::pio::Direction::Out, &[mwrt_pin]);
}

// Initialize channels and spawn RAM task
pub async fn init_ram(spawner: &Spawner) -> RamClient {
    let requests: &mut Channel<CriticalSectionRawMutex, RamRequest, 16> = RAM_REQUESTS.init(Channel::new());
    let responses: &mut Channel<CriticalSectionRawMutex, RamResponse, 16> = RAM_RESPONSES.init(Channel::new());
    
    spawner.spawn(ram_task(requests, responses)).unwrap();
    
    RamClient::new(requests, responses)
}

// RAM task that handles hardware access
#[embassy_executor::task]
async fn ram_task(
    requests: &'static RamChannel,
    responses: &'static RamResponseChannel,
) {
    info!("Starting RAM task");
    
    let p = embassy_rp::init(Default::default());

    // Initialize PIO hardware - keep these alive for the entire task
    let mut pio0 = Pio::new(p.PIO0, Irqs);
    let mut pio1 = Pio::new(p.PIO1, Irqs);
    
    // Create pins
    let data_addr_pins = [
        // data pins (D0-D7)
        &pio0.common.make_pio_pin(p.PIN_8),
        &pio0.common.make_pio_pin(p.PIN_9),
        &pio0.common.make_pio_pin(p.PIN_10),
        &pio0.common.make_pio_pin(p.PIN_11),
        &pio0.common.make_pio_pin(p.PIN_12),
        &pio0.common.make_pio_pin(p.PIN_13),
        &pio0.common.make_pio_pin(p.PIN_14),
        &pio0.common.make_pio_pin(p.PIN_15),
        // address pins (A0-A15)
        &pio0.common.make_pio_pin(p.PIN_16),
        &pio0.common.make_pio_pin(p.PIN_17),
        &pio0.common.make_pio_pin(p.PIN_18),
        &pio0.common.make_pio_pin(p.PIN_19),
        &pio0.common.make_pio_pin(p.PIN_20),
        &pio0.common.make_pio_pin(p.PIN_21),
        &pio0.common.make_pio_pin(p.PIN_22),
        &pio0.common.make_pio_pin(p.PIN_23),
        &pio0.common.make_pio_pin(p.PIN_24),
        &pio0.common.make_pio_pin(p.PIN_25),
        &pio0.common.make_pio_pin(p.PIN_26),
        &pio0.common.make_pio_pin(p.PIN_27),
        &pio0.common.make_pio_pin(p.PIN_28),
        &pio0.common.make_pio_pin(p.PIN_29),
        &pio0.common.make_pio_pin(p.PIN_30),
        &pio0.common.make_pio_pin(p.PIN_31),
    ];

    let addr_pins = &data_addr_pins[8..];
    let data_pins = &data_addr_pins[0..8];
    
    let smemr_pin = pio1.common.make_pio_pin(p.PIN_34);
    pio0.common.make_pio_pin(p.PIN_4);
    let mwrt_pin = pio0.common.make_pio_pin(p.PIN_6);

    // Configure state machines directly
    setup_smemr(&mut pio1.common, &mut pio1.sm0, &smemr_pin);
    setup_reads(&mut pio0.common, &mut pio0.sm1, addr_pins, data_pins);
    setup_writes(&mut pio0.common, &mut pio0.sm2, &data_addr_pins, &mwrt_pin);

    // Enable state machines
    pio1.sm0.set_enable(true);
    pio0.sm1.set_enable(true);
    pio0.sm2.set_enable(true);

    info!("RAM hardware initialized");

    // Process requests
    loop {
        match requests.receive().await {
            RamRequest::Read(address) => {
                info!("RAM task: Initiating read from 0x{:04X}", address);
                
                // Perform read cycle using direct references to state machines
                pio0.sm1.tx().wait_push(address as u32).await;
                pio1.sm0.tx().wait_push(0).await; // Pull SMEMR low
                
                let data = pio0.sm1.rx().wait_pull().await as u8;
                pio1.sm0.tx().wait_push(1).await; // Pull SMEMR high
                
                info!("RAM task: Read 0x{:02X} from 0x{:04X}", data, address);
                responses.send(RamResponse::ReadResult(data)).await;
            },
            
            RamRequest::Write(address, data) => {
                info!("RAM task: Writing 0x{:02X} to 0x{:04X}", data, address);
                
                // Perform write cycle
                pio0.sm2.tx().wait_push(0xFFFFFF).await; // Set pins to output
                
                let value = ((address as u32) << 8) | (data as u32);
                pio0.sm2.tx().wait_push(value).await;
                
                // Wait for write to complete (signaled by IRQ)
                pio0.irq2.wait().await;
                
                // Clean up pin directions
                pio0.sm2.tx().wait_push(0xFFFF00).await;
                
                info!("RAM task: Write complete");
                responses.send(RamResponse::WriteComplete).await;
            }
        }
    }
}