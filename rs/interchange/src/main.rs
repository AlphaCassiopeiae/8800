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

bind_interrupts!(struct Irqs {
    I2C0_IRQ => InterruptHandler<I2C0>;
});

#[allow(dead_code)]
mod mcp23017 {
    pub const ADDR: u8 = 0x00; // default addr

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

    // let sda = p.PIN_0;
    // let scl = p.PIN_1;

    // info!("set up i2c ");
    // let mut i2c = i2c::I2c::new_async(p.I2C0, scl, sda, Irqs, Config::default());

    // use mcp23017::*;

    // info!("init mcp23017 config for IxpandO");
    // // init - a inputs, b inputs
    // i2c.write(ADDR, &[IODIRA, 0xff]).await.unwrap(); // all inputs
    // i2c.write(ADDR, &[IODIRB, 0xff]).await.unwrap(); // all inputs
    // i2c.write(ADDR, &[GPPUA, 0xff]).await.unwrap(); // pull up inputs
    // i2c.write(ADDR, &[GPPUB, 0xff]).await.unwrap(); // pull up inputs

    // loop {
    //     let mut porta = [0];
    //     let mut portb = [0];

    //     // Read from port A (buttons)
    //     i2c.blocking_read(ADDR, &mut porta).unwrap();
    //     info!("porta = {:02x}", porta[0]);
        
    //     // Read from port B (switches)
    //     i2c.blocking_read(ADDR, &mut portb).unwrap();
    //     info!("portb = {:02x}", portb[0]);

    //     Timer::after_millis(100).await;
    // }
}
