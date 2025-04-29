#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::bind_interrupts;
use embassy_rp::gpio::{Flex, Input, Level, Output, Pull};
use embassy_rp::i2c::{self, Config, InterruptHandler};
use embassy_rp::peripherals::{I2C1};
use embassy_time::Timer;
use embedded_hal_async::i2c::I2c;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    I2C1_IRQ => InterruptHandler<I2C1>;
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

// fn write_address_to_pins(address: u16, pins: &mut [&mut Output<'_, AnyPin>]) {
//     for (i, pin) in pins.iter_mut().enumerate() {
//         if ((address >> i) & 1) != 0 {
//             pin.set_high();
//         } else {
//             pin.set_low();
//         }
//     }
// }

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
// fn get_unprot(edges: u8) -> bool {
//     ((edges >> 6) & 0x01) != 0
// }
// fn get_aux1(edges: u8) -> bool {
//     ((edges >> 7) & 0x01) != 0
// }

// BUTTONS B
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
// fn get_prot(edges: u8) -> bool {
//     // bit 0
//     (edges & 0x01) != 0
// }
// fn get_aux2(edges: u8) -> bool {
//     // bit 4
//     ((edges >> 4) & 0x01) != 0
// }

// for correcting address from MCP
fn reverse_bits(mut x: u8) -> u8 {
    x = (x >> 1 & 0x55) | (x << 1 & 0xaa);
    x = (x >> 2 & 0x33) | (x << 2 & 0xcc);
    x = (x >> 4 & 0x0f) | (x << 4 & 0xf0);
    x
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    // Changed from Vec to fixed-size array
    let mut a_pins: [Flex<'_>; 16] = [
        Flex::new(p.PIN_31),
        Flex::new(p.PIN_41),
        Flex::new(p.PIN_33),
        Flex::new(p.PIN_35),
        Flex::new(p.PIN_16),
        Flex::new(p.PIN_18),
        Flex::new(p.PIN_22),
        Flex::new(p.PIN_20),
        Flex::new(p.PIN_42),
        Flex::new(p.PIN_43),
        Flex::new(p.PIN_45),
        Flex::new(p.PIN_3),
        Flex::new(p.PIN_12),
        Flex::new(p.PIN_10),
        Flex::new(p.PIN_7),
        Flex::new(p.PIN_4),
    ];
    
    // Example read function: reads pin levels into a u16 - modified to work with array directly
    fn read_address_from_pins(pins: &mut [Flex<'_>]) -> u16 {
        let mut value = 0u16;
        for (i, pin) in pins.iter_mut().enumerate() {
            if pin.get_level() == Level::High {
                value |= 1 << i;
            }
        }
        value
    }

    fn write_address_to_pins(address: u16, pins: &mut [Flex<'_>]) {
        for (i, pin) in pins.iter_mut().enumerate() {
            if ((address >> i) & 1) != 0 {
                pin.set_high();
            } else {
                pin.set_low();
            }
        }
    }

    // GPIO setup
    let mut rst = Output::new(p.PIN_29, Level::High);
    Input::new(p.PIN_0, Pull::Down); // hlta
    let mut panel = Output::new(p.PIN_27, Level::High);
    let mut clock = Output::new(p.PIN_24, Level::Low);

    let mut mwrt = Flex::new(p.PIN_26);
    mwrt.set_as_input();
    let mut smemr = Flex::new(p.PIN_6);
    smemr.set_as_input();
    let mut xrdy = Input::new(p.PIN_47, Pull::None);

    // I2C setup
    let sda = p.PIN_38;
    let scl = p.PIN_39;
    let mut i2c = i2c::I2c::new_async(p.I2C1, scl, sda, Irqs, Config::default());
    
    // Create pin references for pins 40-47 using `common.make_pio_pin`
    let mut data_pins: [Flex<'_>; 8] = [
        Flex::new(p.PIN_40),
        Flex::new(p.PIN_30),
        Flex::new(p.PIN_32),
        Flex::new(p.PIN_34),
        Flex::new(p.PIN_17),
        Flex::new(p.PIN_19),
        Flex::new(p.PIN_21),
        Flex::new(p.PIN_23),
    ];

    // function for set each flex pin as input
    fn set_pins_as_input(pins: &mut [Flex<'_>; 8]) {
        for i in 0..8 {
            pins[i].set_as_input();
        }
    }

    // function for set each flex pin as output
    fn set_pins_as_output(pins: &mut [Flex<'_>; 8]) {
        for i in 0..8 {
            pins[i].set_as_output();
        }
    }

    fn set_addr_pins_as_input(pins: &mut [Flex<'_>; 16]) {
        for i in 0..16 {
            pins[i].set_as_input();
        }
    }

    fn set_addr_pins_as_output(pins: &mut [Flex<'_>; 16]) {
        for i in 0..16 {
            pins[i].set_as_output();
        }
    }
    

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

    let mut run_mode: bool = false;
    // set_pins_as_input(&mut data_pins);
    
    let mut bus_address: u16 = 0;
    
    loop {
        /* I2C Stuff */
        // Read port A, buttons IC
        // Gives [EXAMNEXT,RUN,STOP,SINGSTEP,DEPONEXT,CLR,UNPROT,AUX1] (potentially reversed)
        i2c.write_read(BUTTONS_ADDR, &[GPIOA], &mut buttons_a).await.unwrap();
        // Read port B, buttons IC
        // Gives [PROT,RESET,DEPOSIT,EXAMINE,AUX2,NULL,NULL,NULL] (potentially reversed)
        i2c.write_read(BUTTONS_ADDR, &[GPIOB], &mut buttons_b).await.unwrap();;
        // Read port A, switches IC
        // Gives A8-A15
        i2c.write_read(ASWITCHES_ADDR, &[GPIOA], &mut switches_a).await.unwrap();
        // Read port B, switches IC
        // Gives A0-A7
        i2c.write_read(ASWITCHES_ADDR, &[GPIOB], &mut switches_b).await.unwrap();

        // info!("buttons = {:02x}  {:02x}\naddress = {:02x}  {:02x}", buttons_a[0], buttons_b[0], switches_a[0],  switches_b[0]);

        // combine addr0 and addr1 into a single address
        let address: u16 = ((reverse_bits(switches_a[0]) as u16) << 8) | (reverse_bits(switches_b[0]) as u16);
        let button_a_edges = negative_edges(last_buttons_a[0], buttons_a[0]);
        let button_b_edges = negative_edges(last_buttons_b[0], buttons_b[0]);
        // info!("current address: {}", address);

        /* GPIO Logic */
        // default values for control signals, works well for pulsing signals after button press
        

        clock.set_low();
        rst.set_high();

        if run_mode {
            clock.set_high();
        }

        if get_reset(button_b_edges) {
            panel.set_high();
            run_mode = false;

            rst.set_low();
            
            info!("RESET triggered!");
        }
        else if get_stop(button_a_edges) {

            // // capture the current address
            // bus_address = read_address_from_pins(&mut a_pins);

            // front panel asserts bus control
            run_mode = false;


            info!("STOP triggered");
        }
        else if get_run(button_a_edges) {
            run_mode = true;
            info!("RUN triggered");
        }
        else if get_singstep(button_a_edges) && run_mode == false {
            clock.set_high();
            info!("STEP triggered");
        }
        else if get_deposit(button_b_edges) && run_mode == false {
            info!("DEPOSIT triggered");
            // get address from gpios
            let bus_address = read_address_from_pins(&mut a_pins);
            
            panel.set_low();
            //print address
            info!("DEPOSIT address: 0x{:04X}", bus_address);

            // get data from switches (LSBs of address). slice address
            let data = address & 0x00FF;
            // print data
            info!("DEPOSIT data: 0x{:04X}", data);

            // set data pins as output
            set_pins_as_output(&mut data_pins);
            // push data to pins
            for i in 0..8 {
                if ((data >> i) & 1) != 0 {
                    data_pins[i].set_high();
                } else {
                    data_pins[i].set_low();
                }
            }

            mwrt.set_as_output();
            mwrt.set_low();

            xrdy.wait_for_high().await; // wait for ready

            mwrt.set_high(); // turn off write
            mwrt.set_as_input(); // set back to input
            
            // turn off data pins
            set_pins_as_input(&mut data_pins);
            
            panel.set_high();
            
            Timer::after_millis(10).await;
        }

        else if get_examine(button_b_edges) && run_mode == false {
            info!("EXAMINE Pressed!");

            // print address
            info!("EXAMINE address: 0x{:04X}", address);
            
            panel.set_high();

            // set address pins as output
            set_addr_pins_as_output(&mut a_pins);

            // put address from pins on address bus
            write_address_to_pins(address, &mut a_pins);

            // assert mread
            smemr.set_as_output();
            smemr.set_low();
            // wait for ready
            xrdy.wait_for_high().await; // wait for ready
            
            // read data pins
            let bus_data = read_address_from_pins(&mut data_pins);

            smemr.set_high(); // turn off read
            smemr.set_as_input(); // set back to input

            // print data
            info!("EXAMINE data: 0x{:04X}", bus_data);

            panel.set_low();
            // set address pins as input
            set_addr_pins_as_input(&mut a_pins);


        }

        // else if get_deponext(button_a_edges) {
        //     info!("DEPOSIT NEXT Pressed!");
        //     // put whatever is on the switches at value on addr LEDs, but also increments address
        // }

        // if !run_mode {prdy.set_low();}
        
        // // button press logic
        // if panel.get_level() == Level::Low {
        //     // stuff that can only be done if panel has control (enter run, single step, examine/deposit, etc.)
        //     // cpu is halted 
        //     if get_reset(button_b_edges) {
        //         info!("RESET triggered!");
        //         rst.set_low();
        //     }
        //     else if get_singstep(button_a_edges) {
        //         info!("SINGLE STEP Pressed!");
        //         prdy.set_high();
        //     }
        //     else if get_run(button_a_edges) {
        //         info!("RUN Pressed!");
        //         run_mode = true;
        //         prdy.set_high();
        //     }
        //     else if get_deposit(button_b_edges) {
        //         info!("DEPOSIT Pressed!");
        //         // put whatever is on the switches at value on addr LEDs
        //     }
        //     else if get_deponext(button_a_edges) {
        //         info!("DEPOSIT NEXT Pressed!");
        //         // put whatever is on the switches at value on addr LEDs, but also increments address
        //     }
        //     else if get_examine(button_b_edges) {
        //         info!("EXAMINE Pressed!");
        //         // read whatever is at the memory address on addr LEDs
        //     }
        //     else if get_examnext(button_a_edges) {
        //         info!("EXAMINE NEXT Pressed!");
        //         // read whetever is at the memory address on addr LEDs, but also increments address
        //     }
        //     else if get_clr(button_a_edges) {
        //         info!("CLEAR Pressed!");
        //         // reset addr (addr LEDs) to 0x0000
        //     }
        //     // else if get_prot(button_b_edges) {
        //     //     info!("PROT Pressed!");
        //     // }
        //     // else if get_unprot(button_a_edges) {
        //     //     info!("UNPROT Pressed!");
        //     // }
        //     // else if get_aux2(button_b_edges) {
        //     //     info!("AUX2 Pressed!");
        //     // }
        //     // else if get_aux1(button_a_edges) {
        //     //     info!("AUX1 Pressed!");
        //     // }
        // } else {
        //     // stuff that can be done while CPU running (reset, stop, etc.)
        //     if get_reset(button_b_edges) {
        //         info!("RESET triggered!");
        //         rst.set_low();
        //     }
        //     else if get_stop(button_a_edges) {
        //         info!("STOP Pressed!");
        //         run_mode = false;
        //     }
        // }
        
        // button edge detection
        last_buttons_b[0] = buttons_b[0];
        last_buttons_a[0] = buttons_a[0];
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

        // Optional: Add delay between updates
        Timer::after_millis(50).await;
    }
}
