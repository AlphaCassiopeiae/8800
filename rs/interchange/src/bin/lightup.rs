//! This example test the RP Pico on board LED.
//!
//! It does not work with the RP Pico W board. See wifi_blinky.rs.

#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::{gpio, usb::Out};
use embassy_time::Timer;
use gpio::{Level, Output};
use {defmt_rtt as _, panic_probe as _};

// Program metadata for `picotool info`.
// This isn't needed, but it's recomended to have these minimal entries.

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    // Create an array of Output pins for GPIOs 0-39
    let mut leds = [
        Output::new(p.PIN_0, Level::Low),
        Output::new(p.PIN_1, Level::Low),
        Output::new(p.PIN_2, Level::Low),
        Output::new(p.PIN_3, Level::Low),
        Output::new(p.PIN_4, Level::Low),
        Output::new(p.PIN_5, Level::Low),
        Output::new(p.PIN_6, Level::Low),
        Output::new(p.PIN_7, Level::Low),
        Output::new(p.PIN_8, Level::Low),
        Output::new(p.PIN_9, Level::Low),
        Output::new(p.PIN_10, Level::Low),
        Output::new(p.PIN_11, Level::Low),
        Output::new(p.PIN_12, Level::Low),
        Output::new(p.PIN_13, Level::Low),
        Output::new(p.PIN_14, Level::Low),
        Output::new(p.PIN_15, Level::Low),
        Output::new(p.PIN_16, Level::Low),
        Output::new(p.PIN_17, Level::Low),
        Output::new(p.PIN_18, Level::Low),
        Output::new(p.PIN_19, Level::Low),
        Output::new(p.PIN_20, Level::Low),
        Output::new(p.PIN_21, Level::Low),
        Output::new(p.PIN_22, Level::Low),
        Output::new(p.PIN_23, Level::Low),
        // Output::new(p.PIN_24, Level::Low),
        // Output::new(p.PIN_25, Level::Low),
        // Output::new(p.PIN_26, Level::Low),
        // Output::new(p.PIN_27, Level::Low),
        // Output::new(p.PIN_28, Level::Low),
        // Output::new(p.PIN_29, Level::Low),
        Output::new(p.PIN_30, Level::Low),
        Output::new(p.PIN_31, Level::Low),
        Output::new(p.PIN_32, Level::Low),
        Output::new(p.PIN_33, Level::Low),
        Output::new(p.PIN_34, Level::Low),
        Output::new(p.PIN_35, Level::Low),
        // Output::new(p.PIN_36, Level::Low),
        // Output::new(p.PIN_37, Level::Low),
        // Output::new(p.PIN_38, Level::Low),
        // Output::new(p.PIN_39, Level::Low),
        Output::new(p.PIN_40, Level::Low),
        Output::new(p.PIN_41, Level::Low),
        Output::new(p.PIN_42, Level::Low),
        Output::new(p.PIN_43, Level::Low),
        Output::new(p.PIN_44, Level::Low),
        Output::new(p.PIN_45, Level::Low)
    ];

    // actually let's just rewrite this to be slice of outputs
    // let mut data_outputs = [
    //     Output::new(p.PIN_40, Level::Low),
    //     Output::new(p.PIN_30, Level::Low),
    //     Output::new(p.PIN_32, Level::Low),
    //     Output::new(p.PIN_34, Level::Low),
    //     Output::new(p.PIN_17, Level::Low),
    //     Output::new(p.PIN_19, Level::Low),
    //     Output::new(p.PIN_21, Level::Low),
    //     Output::new(p.PIN_23, Level::Low)
    // ];


    // drop all gpio pins

    loop {
        // info!("all gpios on!");
        // for led in leds.iter_mut() {
        //     led.set_high();
        // }
        // Timer::after_millis(250).await;

        // info!("all gpios off!");
        // for led in leds.iter_mut() {
        //     led.set_low();
        // }
        // Timer::after_millis(250).await;

        for led in leds.iter_mut() {
            led.set_high();
            Timer::after_millis(250).await;
            led.set_low();
        }

        // light up each data pin one by one
        // for led in data_outputs.iter_mut() {
        //     led.set_high();
        //     Timer::after_millis(250).await;
        //     led.set_low();
        // }
    }
}
