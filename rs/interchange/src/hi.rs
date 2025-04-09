//! This example shows how to communicate asynchronous using i2c with external chips.
//!
//! Example written for the [`MCP23017 16-Bit I2C I/O Expander with Serial Interface`] chip.
//! (https://www.microchip.com/en-us/product/mcp23017)

#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::bind_interrupts;
use embassy_rp::i2c::{self, Config, InterruptHandler};
use embassy_rp::peripherals::I2C0;
use embassy_time::Timer;
use embedded_hal_async::i2c::I2c;
use {defmt_rtt as _, panic_probe as _};
use gpio::{Level, Output};
use embassy_rp::gpio;

bind_interrupts!(struct Irqs {
    I2C0_IRQ => InterruptHandler<I2C0>;
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

#[embassy_executor::main]
async fn main(_spawner: Spawner) {

    // let p = embassy_rp::init(Default::default());
    // let mut led = Output::new(p.PIN_47, Level::Low);

    // loop {
    //     info!("led on!");
    //     led.set_high();
    //     Timer::after_millis(250).await;

    //     info!("led off!");
    //     led.set_low();
    //     Timer::after_millis(250).await;
    // }

    let p = embassy_rp::init(Default::default());

    let sda = p.PIN_0;
    let scl = p.PIN_1;

    info!("set up i2c ");
    let mut i2c = i2c::I2c::new_async(p.I2C0, scl, sda, Irqs, Config::default());

    use mcp23017::*;

    info!("init mcp23017 config for IxpandO");
    // init - a inputs, b inputs
    i2c.write(ADDR, &[IODIRA, 0xff]).await.unwrap(); // all inputs
    i2c.write(ADDR, &[IODIRB, 0xff]).await.unwrap(); // all inputs
    i2c.write(ADDR, &[GPPUA, 0xff]).await.unwrap(); // pull up inputs
    i2c.write(ADDR, &[GPPUB, 0xff]).await.unwrap(); // pull up inputs

    loop {
        let mut addr0 = [0];
        let mut addr1 = [0];
        let mut data = [0];

    //     if (reset) {
    //         nRst = 0;
    //         addr = 0;
    //     }
    //     else {
    //         nRst = 1;
    //         if (clr) {
    //             addr = 0;
    //             data = hiz;
    //             clock = clock;
    //         }
    //         else if (stop) {
    //             clock = clock;
    //         }
    //         else if (run) {
    //             clock = !clock;
    //         }
    //         else if (single_step) {
    //             single_step = 1;
    //         }
    //         else if (examine) {
    //             // read addr pins
    //             i2c.write_read(ADDR, &[GPIOB], &mut addr0).await.unwrap();
    //             i2c.write_read(ADDR1, &[GPIOA], &mut addr1).await.unwrap();

    //             // flog loan word instruction over the bus
    //             panel = 1;
    //             // send addr
    //             panel = 0;
    //         }
    //         else if (examine_next) {
    //             // flog loan word with next addr
    //             panel = 1;
    //             panel = 0;
    //         }
    //         else if (deposit) {
    //             panel = 1;
    //             // read data
    //             i2c.write_read(ADDR, &[GPIOA], &mut data).await.unwrap();
    //             // send data sw (addr)
    //             panel = 0;
    //         }
    //         else if (deposit_next) {
    //             panel = 1;
    //             // read data
    //             i2c.write_read(ADDR, &[GPIOA], &mut data).await.unwrap();
    //             // send data sw addr + 1
    //             panel = 0;
    //         }
    //     }

        // Read from port A, top IC (data)
        i2c.write_read(ADDR, &[GPIOA], &mut data).await.unwrap();
        info!("data = {:02x}", data[0]);
        
        // Read from port B, top IC (lower addr)
        i2c.write_read(ADDR, &[GPIOB], &mut addr0).await.unwrap();
        info!("addr0 = {:02x}", addr0[0]);

        // Read from port A, bottom IC (top addr)
        i2c.write_read(ADDR1, &[GPIOA], &mut addr1).await.unwrap();
        info!("addr1 = {:02x}", addr1[0]);



        Timer::after_millis(500).await;
    }
}

// fn setup_pio_task_sm1<'a>(pio: &mut Common<'a, PIO0>, sm: &mut StateMachine<'a, PIO0, 1>) {
//     // Setupm sm1

//     // Read 0b10101 repeatedly until ISR is full
//     let prg = pio_asm!(
//         "set x, 0x15",
//         ".wrap_target",
//         "in x, 5 [31]",
//         ".wrap",
//     );

//     let mut cfg = Config::default();
//     cfg.use_program(&pio.load_program(&prg.program), &[]);
//     cfg.clock_divider = (U56F8!(125_000_000) / 2000).to_fixed();
//     cfg.shift_in.auto_fill = true;
//     cfg.shift_in.direction = ShiftDirection::Right;
//     sm.set_config(&cfg);
// }
