//! This example test the RP Pico on board LED.
//!
//! It does not work with the RP Pico W board. See wifi_blinky.rs.

#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::{gpio, usb::Out};
use embassy_time::Timer;
use gpio::{Level, Output, Input};
use {defmt_rtt as _, panic_probe as _};

// Program metadata for `picotool info`.
// This isn't needed, but it's recomended to have these minimal entries.

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    // Create an array of Output pins for GPIOs 0-39
    let mut leds = [
        Input::new(p.PIN_0, gpio::Pull::None),
        Input::new(p.PIN_1, gpio::Pull::None),
        Input::new(p.PIN_2, gpio::Pull::None),
        Input::new(p.PIN_3, gpio::Pull::None),
        Input::new(p.PIN_4, gpio::Pull::None),
        Input::new(p.PIN_5, gpio::Pull::None),
        Input::new(p.PIN_6, gpio::Pull::None),
        Input::new(p.PIN_7, gpio::Pull::None),
        Input::new(p.PIN_8, gpio::Pull::None),
        Input::new(p.PIN_9, gpio::Pull::None),
        Input::new(p.PIN_10, gpio::Pull::None),
        Input::new(p.PIN_11, gpio::Pull::None),
        Input::new(p.PIN_12, gpio::Pull::None),
        Input::new(p.PIN_13, gpio::Pull::None),
        Input::new(p.PIN_14, gpio::Pull::None),
        Input::new(p.PIN_15, gpio::Pull::None),
        Input::new(p.PIN_16, gpio::Pull::None),
        Input::new(p.PIN_17, gpio::Pull::None),
        Input::new(p.PIN_18, gpio::Pull::None),
        Input::new(p.PIN_19, gpio::Pull::None),
        Input::new(p.PIN_20, gpio::Pull::None),
        Input::new(p.PIN_21, gpio::Pull::None),
        Input::new(p.PIN_22, gpio::Pull::None),
        Input::new(p.PIN_23, gpio::Pull::None),
        Input::new(p.PIN_24, gpio::Pull::None),
        Input::new(p.PIN_25, gpio::Pull::None),
        Input::new(p.PIN_26, gpio::Pull::None),
        Input::new(p.PIN_27, gpio::Pull::None),
        Input::new(p.PIN_28, gpio::Pull::None),
        Input::new(p.PIN_29, gpio::Pull::None),
        Input::new(p.PIN_30, gpio::Pull::None),
        Input::new(p.PIN_31, gpio::Pull::None),
        Input::new(p.PIN_32, gpio::Pull::None),
        Input::new(p.PIN_33, gpio::Pull::None),
        Input::new(p.PIN_34, gpio::Pull::None),
        Input::new(p.PIN_35, gpio::Pull::None),
        // Input::new(p.PIN_36, gpio::Pull::None),
        // Input::new(p.PIN_37, gpio::Pull::None),
        // Input::new(p.PIN_38, gpio::Pull::None),
        // Input::new(p.PIN_39, gpio::Pull::None),
        Input::new(p.PIN_40, gpio::Pull::None),
        Input::new(p.PIN_41, gpio::Pull::None),
        Input::new(p.PIN_42, gpio::Pull::None),
        Input::new(p.PIN_43, gpio::Pull::None),
        Input::new(p.PIN_44, gpio::Pull::None),
        Input::new(p.PIN_45, gpio::Pull::None),
        Input::new(p.PIN_46, gpio::Pull::None),
        Input::new(p.PIN_47, gpio::Pull::None),
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

        for (index, led) in leds.iter().enumerate() {
            if led.is_high() {
                // print not high and led index
                info!("led at index {} is low", index);
            }
        }

        // light up each data pin one by one
        // for led in data_outputs.iter_mut() {
        //     led.set_high();
        //     Timer::after_millis(250).await;
        //     led.set_low();
        // }
    }
}
