#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::bind_interrupts;
use embassy_rp::gpio::{Level, Output};
use embassy_rp::i2c::{self, Config, InterruptHandler};
use embassy_rp::peripherals::{I2C1, PIO0};
use embassy_rp::pio::program::pio_asm;
use embassy_rp::pio::{Common, Config as PioConfig, InterruptHandler as PioInterruptHandler, Pio, Pin, ShiftDirection, StateMachine};
use embassy_time::Timer;
use embedded_hal_async::i2c::I2c;
use fixed::traits::ToFixed;
use fixed_macro::types::U56F8;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    I2C1_IRQ => InterruptHandler<I2C1>;
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
    let mut led = Output::new(p.PIN_4, Level::Low);

    // I2C setup
    let sda = p.PIN_38;
    let scl = p.PIN_39;
    let mut i2c = i2c::I2c::new_async(p.I2C1, scl, sda, Irqs, Config::default());

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
    i2c.write(ADDR1, &[IODIRA, 0xff]).await.unwrap(); // all inputs
    i2c.write(ADDR1, &[IODIRB, 0xff]).await.unwrap(); // all inputs
    i2c.write(ADDR1, &[GPPUA, 0x00]).await.unwrap(); // pull up inputs
    i2c.write(ADDR1, &[GPPUB, 0x00]).await.unwrap(); // pull up inputs
    // init - a inputs, b inputs
    i2c.write(ADDR, &[IODIRA, 0xff]).await.unwrap(); // all inputs
    i2c.write(ADDR, &[IODIRB, 0xff]).await.unwrap(); // all inputs
    i2c.write(ADDR, &[GPPUA, 0x00]).await.unwrap(); // pull up inputs
    i2c.write(ADDR, &[GPPUB, 0x00]).await.unwrap(); // pull up inputs

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

        // Send the data to the PIO FIFO - this will be output to pins 40-47
        sm0.tx().wait_push(data[0] as u32).await;
        
        // Optional: Add delay between updates
        Timer::after_millis(50).await;
    }
}

// ! This example shows powerful PIO module in the RP2040 chip.

// #![no_std]
// #![no_main]
// use defmt::info;
// use embassy_executor::Spawner;
// use embassy_rp::peripherals::PIO0;
// use embassy_rp::pio::program::pio_asm;
// use embassy_rp::pio::{Common, Config, InterruptHandler, Irq, Pio, PioPin, ShiftDirection, StateMachine};
// use embassy_rp::{bind_interrupts, Peri};
// use fixed::traits::ToFixed;
// use fixed_macro::types::U56F8;
// use {defmt_rtt as _, panic_probe as _};

// bind_interrupts!(struct Irqs {
//     PIO0_IRQ_0 => InterruptHandler<PIO0>;
// });

// fn setup_pio_task_sm0<'a>(pio: &mut Common<'a, PIO0>, sm: &mut StateMachine<'a, PIO0, 0>, pin: Peri<'a, impl PioPin>) {
//     // Setup sm0

//     // Send data serially to pin
//     let prg = pio_asm!(
//         ".origin 16",
//         "set pindirs, 1",
//         ".wrap_target",
//         "out pins,1 [19]",
//         ".wrap",
//     );

//     let mut cfg = Config::default();
//     cfg.use_program(&pio.load_program(&prg.program), &[]);
//     let out_pin = pio.make_pio_pin(pin);
//     cfg.set_out_pins(&[&out_pin]);
//     cfg.set_set_pins(&[&out_pin]);
//     cfg.clock_divider = (U56F8!(125_000_000) / 20 / 200).to_fixed();
//     cfg.shift_out.auto_fill = true;
//     sm.set_config(&cfg);
// }

// #[embassy_executor::task]
// async fn pio_task_sm0(mut sm: StateMachine<'static, PIO0, 0>) {
//     sm.set_enable(true);

//     let mut v = 0x0f0caffa;
//     loop {
//         sm.tx().wait_push(v).await;
//         v ^= 0xffff;
//         info!("Pushed {:032b} to FIFO", v);
//     }
// }

// fn setup_pio_task_sm1<'a>(pio: &mut Common<'a, PIO0>, sm: &mut StateMachine<'a, PIO0, 1>) {
//     // Setupm sm1

//     // Read 0b10101 repeatedly until ISR is full
//     let prg = pio_asm!(
//         //
//         ".origin 8",
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

// #[embassy_executor::task]
// async fn pio_task_sm1(mut sm: StateMachine<'static, PIO0, 1>) {
//     sm.set_enable(true);
//     loop {
//         let rx = sm.rx().wait_pull().await;
//         info!("Pulled {:032b} from FIFO", rx);
//     }
// }

// fn setup_pio_task_sm2<'a>(pio: &mut Common<'a, PIO0>, sm: &mut StateMachine<'a, PIO0, 2>) {
//     // Setup sm2

//     // Repeatedly trigger IRQ 3
//     let prg = pio_asm!(
//         ".origin 0",
//         ".wrap_target",
//         "set x,10",
//         "delay:",
//         "jmp x-- delay [15]",
//         "irq 3 [15]",
//         ".wrap",
//     );
//     let mut cfg = Config::default();
//     cfg.use_program(&pio.load_program(&prg.program), &[]);
//     cfg.clock_divider = (U56F8!(125_000_000) / 2000).to_fixed();
//     sm.set_config(&cfg);
// }

// #[embassy_executor::task]
// async fn pio_task_sm2(mut irq: Irq<'static, PIO0, 3>, mut sm: StateMachine<'static, PIO0, 2>) {
//     sm.set_enable(true);
//     loop {
//         irq.wait().await;
//         info!("IRQ trigged");
//     }
// }

// #[embassy_executor::main]
// async fn main(spawner: Spawner) {
//     let p = embassy_rp::init(Default::default());
//     let pio = p.PIO0;

//     let Pio {
//         mut common,
//         irq3,
//         mut sm0,
//         mut sm1,
//         mut sm2,
//         ..
//     } = Pio::new(pio, Irqs);

//     setup_pio_task_sm0(&mut common, &mut sm0, p.PIN_45);
//     // setup_pio_task_sm1(&mut common, &mut sm1);
//     // setup_pio_task_sm2(&mut common, &mut sm2);
//     spawner.spawn(pio_task_sm0(sm0)).unwrap();
//     spawner.spawn(pio_task_sm1(sm1)).unwrap();
//     spawner.spawn(pio_task_sm2(irq3, sm2)).unwrap();
// }
