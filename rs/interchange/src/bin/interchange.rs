#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::bind_interrupts;
use embassy_rp::gpio::{Level, Output};
use embassy_rp::i2c::{self, Config, InterruptHandler};
use embassy_rp::peripherals::{I2C0, PIO0};
use embassy_rp::pio::program::pio_asm;
use embassy_rp::pio::{Common, Config as PioConfig, InterruptHandler as PioInterruptHandler, Pio, Pin, ShiftDirection, StateMachine};
use embassy_time::Timer;
use embedded_hal_async::i2c::I2c;
use fixed::traits::ToFixed;
use fixed_macro::types::U56F8;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    I2C0_IRQ => InterruptHandler<I2C0>;
    PIO0_IRQ_0 => PioInterruptHandler<PIO0>;
});

#[allow(dead_code)]
mod mcp23017 {
    pub const ADDR: u8 = 0x20; // default addr
    pub const ADDR1: u8 = 0x21;

    macro_rules! mcpregs {
        ($($name:ident : $val:expr),* $(,)?) => {
            $(
                pub const $name: u8 = $val;
            )*

            pub fn regname(reg: u8) -> &'static str {
                match reg {
                    $(
                        $val => stringify!($name),
                    )*
                    _ => panic!("bad reg"),
                }
            }
        }
    }

    // These are correct for IOCON.BANK=0
    mcpregs! {
        IODIRA: 0x00,
        IPOLA: 0x02,
        GPINTENA: 0x04,
        DEFVALA: 0x06,
        INTCONA: 0x08,
        IOCONA: 0x0A,
        GPPUA: 0x0C,
        INTFA: 0x0E,
        INTCAPA: 0x10,
        GPIOA: 0x12,
        OLATA: 0x14,
        IODIRB: 0x01,
        IPOLB: 0x03,
        GPINTENB: 0x05,
        DEFVALB: 0x07,
        INTCONB: 0x09,
        IOCONB: 0x0B,
        GPPUB: 0x0D,
        INTFB: 0x0F,
        INTCAPB: 0x11,
        GPIOB: 0x13,
        OLATB: 0x15,
    }
}

/// Configure PIO to output data to pins 40-47
fn setup_pio_output<'a>(pio: &mut Common<'a, PIO0>, sm: &mut StateMachine<'a, PIO0, 0>, pins: &[&Pin<'a, PIO0>]) {
    // PIO program to output data to 8 pins (40-47)
    let prg = pio_asm!(
        "set pindirs, 1",  // Set pins as outputs
        ".wrap_target",
        "pull block",      // Wait for more data from FIFO
        "out pins, 8",     // Output 8 bits to pins
        ".wrap",
    );

    let mut cfg = PioConfig::default();
    cfg.use_program(&pio.load_program(&prg.program), &[]);
    
    // Configure pins for output
    cfg.set_out_pins(pins);
    cfg.clock_divider = (U56F8!(125_000_000) / 10000).to_fixed(); // 10 kHz update rate
    cfg.shift_out.auto_fill = true; // Corrected field name
    cfg.shift_out.direction = ShiftDirection::Right;
    
    sm.set_config(&cfg);
    
    // Configure pins as outputs
    sm.set_pin_dirs(embassy_rp::pio::Direction::Out, pins); // Corrected method signature
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    // non parallel leds
    // let mut led = Output::new(p.PIN_4, Level::Low);

    // I2C setup
    let sda = p.PIN_4;
    let scl = p.PIN_5;
    let mut i2c = i2c::I2c::new_async(p.I2C0, scl, sda, Irqs, Config::default());

    // PIO setup for output pins 40-47
    let Pio { mut common, mut sm0, .. } = Pio::new(p.PIO0, Irqs);
    
    // Create pin references for pins 40-47 using `common.make_pio_pin`
    let pins = [
        &common.make_pio_pin(p.PIN_16),
        &common.make_pio_pin(p.PIN_17),
        &common.make_pio_pin(p.PIN_18),
        &common.make_pio_pin(p.PIN_19),
        &common.make_pio_pin(p.PIN_20),
        &common.make_pio_pin(p.PIN_21),
        &common.make_pio_pin(p.PIN_22),
        &common.make_pio_pin(p.PIN_23),
    ];
    
    // Configure PIO for output
    setup_pio_output(&mut common, &mut sm0, &pins);
    
    // Enable the state machine
    sm0.set_enable(true);

    use mcp23017::*;

    info!("init mcp23017 config for IxpandO");
    // init - a inputs, b inputs
    /* FOR ACTUAL HARDWARE, GPPU needs to be 0x00 instead of 0xFF since the pulls are different */
    i2c.write(ADDR1, &[IODIRA, 0xff]).await.unwrap(); // all inputs
    i2c.write(ADDR1, &[IODIRB, 0xff]).await.unwrap(); // all inputs
    i2c.write(ADDR1, &[GPPUA, 0xff]).await.unwrap(); // pull up inputs
    i2c.write(ADDR1, &[GPPUB, 0xff]).await.unwrap(); // pull up inputs
    // init - a inputs, b inputs
    i2c.write(ADDR, &[IODIRA, 0xff]).await.unwrap(); // all inputs
    i2c.write(ADDR, &[IODIRB, 0xff]).await.unwrap(); // all inputs
    i2c.write(ADDR, &[GPPUA, 0xff]).await.unwrap(); // pull up inputs
    i2c.write(ADDR, &[GPPUB, 0xff]).await.unwrap(); // pull up inputs

    loop {
        let mut addr0 = [0];
        let mut addr1 = [0];
        let mut data = [0];
        let mut tmp = [0];

        // Read from port B, top IC (data)
        i2c.write_read(ADDR1, &[GPIOA], &mut data).await.unwrap();
        
        
        // // Read from port B, top IC (lower addr)
        i2c.write_read(ADDR1, &[GPIOB], &mut addr0).await.unwrap();
        // // Read from port A, bottom IC (top addr)
        i2c.write_read(ADDR, &[GPIOA], &mut addr1).await.unwrap();
        i2c.write_read(ADDR, &[GPIOB], &mut tmp).await.unwrap();
        
        info!("data = {:02x}  {:02x}  {:02x}  {:02x}", data[0], addr0[0], addr1[0],  tmp[0]);

        // combine addr0 and addr1 into a single address
        let address = ((addr1[0] as u32) << 8) | (addr0[0] as u32);

        // if (reset) {
        //     nRst = 0;
        //     addr = 0;
        // }
        // else {
        //     nRst = 1;
        //     if (clr) {
        //         addr = 0;
        //         data = hiz;
        //         clock = clock;
        //     }
        //     else if (stop) {
        //         clock = clock;
        //     }
        //     else if (run) {
        //         clock = !clock;
        //     }
        //     else if (single_step) {
        //         single_step = 1;
        //     }
        //     else if (examine) {
        //         // read addr pins
        //         i2c.write_read(ADDR, &[GPIOB], &mut addr0).await.unwrap();
        //         i2c.write_read(ADDR1, &[GPIOA], &mut addr1).await.unwrap();

        //         // flog loan word instruction over the bus
        //         panel = 1;
        //         // send addr
        //         panel = 0;
        //     }
        //     else if (examine_next) {
        //         // flog loan word with next addr
        //         panel = 1;
        //         panel = 0;
        //     }
        //     else if (deposit) {
        //         panel = 1;
        //         // read data
        //         i2c.write_read(ADDR, &[GPIOA], &mut data).await.unwrap();
        //         // send data sw (addr)
        //         panel = 0;
        //     }
        //     else if (deposit_next) {
        //         panel = 1;
        //         // read data
        //         i2c.write_read(ADDR, &[GPIOA], &mut data).await.unwrap();
        //         // send data sw addr + 1
        //         panel = 0;
        //     }
        // }

        // Send the data to the PIO FIFO - this will be output to pins 40-47
        sm0.tx().wait_push(data[0] as u32).await;
        
        // Optional: Add delay between updates
        Timer::after_millis(50).await;
    }
}