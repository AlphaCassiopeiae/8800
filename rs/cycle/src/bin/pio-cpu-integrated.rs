#![no_std]
#![no_main]


use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::bind_interrupts;
use embassy_rp::gpio::{Level, Output};
use embassy_rp::peripherals::{PIO0, PIO1};
use embassy_rp::pio::program::pio_asm;
use embassy_rp::pio::{Common, Config as PioConfig, InterruptHandler as PioInterruptHandler, Pio, Pin, ShiftDirection, StateMachine, Irq};
use embassy_rp::pac;
use embassy_time::Timer;
use fixed::traits::ToFixed;
use fixed_macro::types::U56F8;
use {defmt_rtt as _, panic_probe as _};

// Import CPU emulator components
use cpu_emulator::cpu::{CPU, Reg};
use cpu_emulator::instructions::*;
use cpu_emulator::ram::RAM;

// Bus transaction handling
mod bus_interface {
    use super::*;

    /// State machines for bus transactions
    pub struct BusSms {
        pub smemr_sm: StateMachine<'static, PIO1, 0>,
        pub read_sm: StateMachine<'static, PIO0, 1>,
        pub write_sm: StateMachine<'static, PIO0, 2>,
        pub irq2: Irq<'static, PIO0, 2>,
    }

    /// Performs a memory read transaction over the S100 bus
    pub async fn bus_read(bus_sms: &mut BusSms, address: u16) -> u8 {
        info!("Bus read from address 0x{:04X}", address);

        // Push address to address/data state machine
        bus_sms.read_sm.tx().wait_push(address as u32).await;
        
        // Pull SMEMR# low to start memory read transaction
        bus_sms.smemr_sm.tx().wait_push(0).await;
        
        // Read the data received
        let data = bus_sms.read_sm.rx().wait_pull().await as u8;
        
        // Return SMEMR# high to end the transaction
        bus_sms.smemr_sm.tx().wait_push(0).await;
        
        info!("Completed bus read: 0x{:02X} from address 0x{:04X}", data, address);
        data
    }

    /// Performs a memory write transaction over the S100 bus
    pub async fn bus_write(bus_sms: &mut BusSms, address: u16, data: u8) {
        info!("Bus write to address 0x{:04X}, data 0x{:02X}", address, data);
        
        // Set address and data pins to output
        bus_sms.write_sm.tx().wait_push(0xFFFFFF).await;

        // Combine address and data for the write transaction
        let value = ((address as u32) << 8) | (data as u32);
        
        // Send address and data
        bus_sms.write_sm.tx().wait_push(value as u32).await;

        // Wait for write to complete
        bus_sms.irq2.wait().await;
        
        // Reset pin directions
        bus_sms.write_sm.tx().wait_push(0xFFFF00).await;
        
        info!("Completed bus write to address 0x{:04X}", address);
    }

    /// Bus-based RAM implementation
    pub struct BusCPU {
        cpu: CPU,
        bus_sms: BusSms, // Direct ownership instead of borrowing
    }

    impl BusCPU {
        pub fn new(
            smemr_sm: StateMachine<'static, PIO1, 0>,
            read_sm: StateMachine<'static, PIO0, 1>,
            write_sm: StateMachine<'static, PIO0, 2>,
            irq2: Irq<'static, PIO0, 2>
        ) -> Self {
            // Create a dummy RAM (we don't use it, but CPU needs one initially)
            let ram = RAM::new();
            let cpu = CPU::new(ram);
            
            // Create BusSms directly inside BusCPU
            let bus_sms = BusSms {
                smemr_sm,
                read_sm,
                write_sm,
                irq2,
            };
            
            Self { cpu, bus_sms }
        }

        pub fn reset(&mut self) {
            self.cpu.reset();
        }

        pub async fn fetch_instruction(&mut self) -> u8 {
            let addr = self.cpu.pc;
            self.read_mem(addr).await
        }

        pub async fn fetch_byte(&mut self, pc: usize) -> u8 {
            self.read_mem(pc + 1).await
        }

        pub async fn fetch_word(&mut self, pc: usize) -> u16 {
            let lower = self.read_mem(pc + 1).await as u16;
            let higher = self.read_mem(pc + 2).await as u16;
            (higher << 8) | lower
        }

        pub async fn read_mem(&mut self, addr: usize) -> u8 {
            bus_read(&mut self.bus_sms, addr as u16).await
        }

        pub async fn write_mem(&mut self, addr: usize, data: u8) {
            bus_write(&mut self.bus_sms, addr as u16, data).await
        }

        pub async fn execute_instruction(&mut self, instruction: u8) {
            info!("Executing instruction: 0x{:02X}", instruction);
            
            // Get instruction info for logging
            let info = INSTRUCTION_TABLE[instruction as usize];
            info!("Instruction: {}", info.mnemonic);
            
            // Many instructions might need to be adapted to use async memory access
            // This is a simple implementation for demonstration purposes
            
            match instruction {
                // Sample handling for a few instruction types
                
                // Memory read instructions
                0x3A => { // LDA a16
                    let addr = self.fetch_word(self.cpu.pc).await;
                    let data = self.read_mem(addr as usize).await;
                    self.cpu.set_reg(Reg::A, data);
                    self.cpu.pc += 3; // Advance PC past the instruction and operands
                },
                
                // Memory write instructions
                0x32 => { // STA a16
                    let addr = self.fetch_word(self.cpu.pc).await;
                    let data = self.cpu.read_reg(Reg::A);
                    self.write_mem(addr as usize, data).await;
                    self.cpu.pc += 3; // Advance PC past the instruction and operands
                },
                
                // MOV M instructions
                0x46 => { // MOV B,M
                    let addr = self.cpu.read_hl() as usize;
                    let data = self.read_mem(addr).await;
                    self.cpu.set_reg(Reg::B, data);
                    self.cpu.pc += 1;
                },
                
                0x70..=0x75 | 0x77 => { // MOV M,r
                    let addr = self.cpu.read_hl() as usize;
                    let reg = Reg::from((instruction & 0x07) as u8);
                    let data = self.cpu.read_reg(reg);
                    self.write_mem(addr, data).await;
                    self.cpu.pc += 1;
                },
                
                // HLT instruction
                0x76 => {
                    self.cpu.halt = true;
                    self.cpu.pc += 1;
                },
                
                // Other instructions would need similar implementations
                // For now we'll handle just these examples to demonstrate
                
                _ => {
                    info!("Unhandled instruction: 0x{:02X}", instruction);
                    self.cpu.pc += info.bytes as usize; // Skip unhandled instruction
                }
            }
            
            self.cpu.show_state();
        }

        pub fn halted(&self) -> bool {
            self.cpu.halt
        }
    }
}

bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => PioInterruptHandler<PIO0>;
    PIO1_IRQ_0 => PioInterruptHandler<PIO1>;
});

/// Configure PIO for SMEMR# control
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
    
    // Configure SMEMR pin for output
    cfg.set_set_pins(&[smemr_pin]);
    
    // Set clock rate
    cfg.clock_divider = (U56F8!(125_000_000) / 20 / 200).to_fixed(); // 100kHz operation
    
    sm.set_config(&cfg);
    
    // Set SMEMR pin as output
    sm.set_pin_dirs(embassy_rp::pio::Direction::Out, &[smemr_pin]);
}

/// Configure PIO for address output and data input (read cycle)
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
}

/// Configure PIO for address output and data output (write cycle)
fn setup_writes<'a>(
    pio: &mut Common<'a, PIO0>, 
    sm: &mut StateMachine<'a, PIO0, 2>,
    data_addr_pins: &[&Pin<'a, PIO0>],
    mwrt_pin: &Pin<'a, PIO0>,
) {
    let prg = pio_asm!(
        ".wrap_target",
        "set pins, 1",        // Deassert MWRT# (inactive high)
        "pull block",
        "out pindirs, 24",    // Set pin directions
        "pull block",         // Get address+data from TX FIFO
        "out pins, 24",       // Output address+data to pins
        "set pins, 0",        // Assert MWRT# (active low)
        "wait 1 gpio 4",      // Wait for XRDY high (write complete)
        "irq 2",              // Notify write complete
        "pull block",
        "out pindirs, 24",    // Reset pin directions
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

    // Set MWRT pin as output
    sm.set_pin_dirs(embassy_rp::pio::Direction::Out, &[mwrt_pin]);
}

/// Initialize the test program to be executed by the CPU
async fn initialize_program(bus_cpu: &mut bus_interface::BusCPU) {
    info!("Initializing test program in memory");
    
    // Simple test program that performs a few memory operations:
    // 1. Load the value at address 0x2008 into register A (LDA 0x2008)
    // 2. Store register A to address 0x3000 (STA 0x3000)
    // 3. Halt (HLT)
    
    // Program bytes: LDA 0x2008
    bus_cpu.write_mem(0x0000, 0x3A).await;  // LDA a16
    bus_cpu.write_mem(0x0001, 0x08).await;  // Low byte of address
    bus_cpu.write_mem(0x0002, 0x20).await;  // High byte of address
    
    // STA 0x3000
    bus_cpu.write_mem(0x0003, 0x32).await;  // STA a16
    bus_cpu.write_mem(0x0004, 0x00).await;  // Low byte of address
    bus_cpu.write_mem(0x0005, 0x30).await;  // High byte of address
    
    // HLT
    bus_cpu.write_mem(0x0006, 0x76).await;  // HLT
    
    info!("Test program initialized");
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    // Create PIO state machines
    let Pio { mut common, mut irq2, mut sm1, mut sm2, .. } = Pio::new(p.PIO0, Irqs);
    let mut pio1 = Pio::new(p.PIO1, Irqs);
    
    // Setup pins for address and data buses
    let data_addr_pins = [
        // data pins (D0-D7)
        &common.make_pio_pin(p.PIN_8),
        &common.make_pio_pin(p.PIN_9),
        &common.make_pio_pin(p.PIN_10),
        &common.make_pio_pin(p.PIN_11),
        &common.make_pio_pin(p.PIN_12),
        &common.make_pio_pin(p.PIN_13),
        &common.make_pio_pin(p.PIN_14),
        &common.make_pio_pin(p.PIN_15),
        // address pins (A0-A15)
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

    // Take slices of address and data pins
    let addr_pins = &data_addr_pins[8..24]; // A0-A15
    let data_pins = &data_addr_pins[0..8];  // D0-D7

    // Setup control pins
    let smemr_pin = &pio1.common.make_pio_pin(p.PIN_34);  // SMEMR# pin
    common.make_pio_pin(p.PIN_4); // XRDY pin (input from memory board)
    let mwrt_pin = &common.make_pio_pin(p.PIN_6); // MWRT# pin

    // Configure PIO state machines
    setup_smemr(&mut pio1.common, &mut pio1.sm0, smemr_pin);
    setup_reads(&mut common, &mut sm1, &addr_pins, &data_pins);
    setup_writes(&mut common, &mut sm2, &data_addr_pins, mwrt_pin);

    // Enable state machines
    pio1.sm0.set_enable(true);
    sm1.set_enable(true);
    sm2.set_enable(true);

    // Create the CPU
    let mut bus_cpu = bus_interface::BusCPU::new(pio1.sm0, sm1, sm2, irq2);
    bus_cpu.reset();

    // Write our test program to memory
    initialize_program(&mut bus_cpu).await;

    info!("Starting CPU execution");

    // Run the CPU until it halts
    loop {
        // Fetch and execute the next instruction
        let instr = bus_cpu.fetch_instruction().await;
        bus_cpu.execute_instruction(instr).await;
        
        // Check if the CPU has halted
        if bus_cpu.halted() {
            info!("CPU halted, execution complete");
            break;
        }
        
        // Small delay for easier debugging
        Timer::after_millis(500).await;
    }

    // Keep the program running to observe results
    loop {
        Timer::after_secs(1).await;
    }
}