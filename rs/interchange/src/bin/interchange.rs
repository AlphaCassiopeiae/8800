#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::bind_interrupts;
use embassy_rp::gpio::{Input, Output, Level, Pull};
use embassy_rp::i2c::{self, Config, InterruptHandler};
use embassy_rp::pac::usb::regs::BuffCpuShouldHandle;
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
    pub const BUTTONS_ADDR: u8 = 0x20; // default addr
    pub const ASWITCHES_ADDR: u8 = 0x21;

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

/* NOTES */
// Buttons are all on one expander, Address switches on other
// All buttons are active low

// rework: negative edge detection across entire 8-bit bank
fn negative_edges(last_state: u8, curr_state: u8) -> u8{
    last_state & !curr_state
}

// BUTTONS A
fn get_examnext(edges: u8) -> bool {
    (edges & 0x01) != 0
}
fn get_run(edges: u8) -> bool {
    ((edges >> 1) & 0x01) != 0
}
fn get_stop(edges: u8) -> bool {
    ((edges >> 2) & 0x01) != 0
}
fn get_singstep(edges: u8) -> bool {
    ((edges >> 3) & 0x01) != 0
}
fn get_deponext(edges: u8) -> bool {
    ((edges >> 4) & 0x01) != 0
}
fn get_clr(edges: u8) -> bool {
    ((edges >> 5) & 0x01) != 0
}
fn get_unprot(edges: u8) -> bool {
    ((edges >> 6) & 0x01) != 0
}
fn get_aux1(edges: u8) -> bool {
    ((edges >> 7) & 0x01) != 0
}

// BUTTONS B
fn get_prot(edges: u8) -> bool {
    // bit 0
    (edges & 0x01) != 0
}
fn get_reset(edges: u8) -> bool {
    // assuming reset signal stored in bit 1
    ((edges >> 1) & 0x01) != 0
}
fn get_deposit(edges: u8) -> bool {
    // bit 2
    ((edges >> 2) & 0x01) != 0
}
fn get_examine(edges: u8) -> bool {
    // bit 3
    ((edges >> 3) & 0x01) != 0
}
fn get_aux2(edges: u8) -> bool {
    // bit 4
    ((edges >> 4) & 0x01) != 0
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    // GPIO setup
    let mut rst = Output::new(p.PIN_29, Level::High);

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
    i2c.write(ASWITCHES_ADDR, &[IODIRA, 0xff]).await.unwrap(); // all inputs
    i2c.write(ASWITCHES_ADDR, &[IODIRB, 0xff]).await.unwrap(); // all inputs
    i2c.write(ASWITCHES_ADDR, &[GPPUA, 0xff]).await.unwrap(); // pull up inputs
    i2c.write(ASWITCHES_ADDR, &[GPPUB, 0xff]).await.unwrap(); // pull up inputs
    // init - a inputs, b inputs
    i2c.write(BUTTONS_ADDR, &[IODIRA, 0xff]).await.unwrap(); // all inputs
    i2c.write(BUTTONS_ADDR, &[IODIRB, 0xff]).await.unwrap(); // all inputs
    i2c.write(BUTTONS_ADDR, &[GPPUA, 0xff]).await.unwrap(); // pull up inputs
    i2c.write(BUTTONS_ADDR, &[GPPUB, 0xff]).await.unwrap(); // pull up inputs

    // set up buffers for inputs
    let mut buttons_a = [0];
    let mut buttons_b = [0];
    let mut switches_a = [0];
    let mut switches_b = [0];
    let mut last_buttons_a = [0xFF];
    let mut last_buttons_b = [0xFF];
    let mut last_switches_a = [0xFF];
    let mut last_switches_b = [0xFF];

    loop {
        // Read port A, buttons IC
        // Gives [EXAMNEXT,RUN,STOP,SINGSTEP,DEPONEXT,CLR,UNPROT,AUX1] (potentially reversed)
        i2c.write_read(BUTTONS_ADDR, &[GPIOA], &mut buttons_a).await.unwrap();
        // Read port B, buttons IC
        // Gives [PROT,RESET,DEPOSIT,EXAMINE,AUX2,NULL,NULL,NULL] (potentially reversed)
        i2c.write_read(BUTTONS_ADDR, &[GPIOB], &mut buttons_b).await.unwrap();;
        // Read port A, switches IC
        // Gives A15-A8
        i2c.write_read(ASWITCHES_ADDR, &[GPIOA], &mut switches_a).await.unwrap();
        // Read port B, switches IC
        // Gives A7-A0
        i2c.write_read(ASWITCHES_ADDR, &[GPIOB], &mut switches_b).await.unwrap();

        // info!("buttons = {:02x}  {:02x}\naddress = {:02x}  {:02x}", buttons_a[0], buttons_b[0], switches_a[0],  switches_b[0]);

        // combine addr0 and addr1 into a single address
        // let address: u16 = ((switches_a[0] as u16) << 8) | (switches_b[0] as u16);
        let button_a_edges = negative_edges(last_buttons_a[0], buttons_a[0]);
        let button_b_edges = negative_edges(last_buttons_b[0], buttons_b[0]);

        // structure these conditions to achieve desired functionality
        if get_reset(button_b_edges) {
            info!("RESET Pressed!");
            rst.set_low();
        }
        else if get_prot(button_b_edges) {
            info!("PROT Pressed!");
        }
        else if get_deposit(button_b_edges) {
            info!("DEPOSIT Pressed!");
        }
        else if get_examine(button_b_edges) {
            info!("EXAMINE Pressed!");
        }
        else if get_aux2(button_b_edges) {
            info!("AUX2 Pressed!");
        }
        else if get_examnext(button_a_edges) {
            info!("EXAMINE NEXT Pressed!");
        }
        else if get_run(button_a_edges) {
            info!("RUN Pressed!");
        }
        else if get_stop(button_a_edges) {
            info!("STOP Pressed!");
        }
        else if get_singstep(button_a_edges) {
            info!("SINGLE STEP Pressed!");
        }
        else if get_deponext(button_a_edges) {
            info!("DEPOSIT NEXT Pressed!");
        }
        else if get_clr(button_a_edges) {
            info!("CLEAR Pressed!");
        }
        else if get_unprot(button_a_edges) {
            info!("UNPROT Pressed!");
        }
        else if get_aux1(button_a_edges) {
            info!("AUX1 Pressed!");
        }

        last_buttons_b[0] = buttons_b[0];
        last_buttons_a[0] = buttons_a[0];
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
        // sm0.tx().wait_push(data[0] as u32).await;
        
        // Optional: Add delay between updates
        Timer::after_millis(50).await;
    }
}