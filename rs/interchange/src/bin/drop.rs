//! This example test the RP Pico on board LED.
//!
//! It does not work with the RP Pico W board. See wifi_blinky.rs.

#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::gpio;
use embassy_rp::gpio::{Input, Pull};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

// Program metadata for `picotool info`.
// This isn't needed, but it's recomended to have these minimal entries.

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    // Hold Input objects so pins remain in input mode with no pull
    let _pins = [
        Input::new(p.PIN_0, Pull::Down),
        Input::new(p.PIN_1, Pull::Down),
        Input::new(p.PIN_2, Pull::Down),
        Input::new(p.PIN_3, Pull::Down),
        Input::new(p.PIN_4, Pull::Down),
        Input::new(p.PIN_5, Pull::Down),
        Input::new(p.PIN_6, Pull::Down),
        Input::new(p.PIN_7, Pull::Down),
        Input::new(p.PIN_8, Pull::Down),
        Input::new(p.PIN_9, Pull::Down),
        Input::new(p.PIN_10, Pull::Down),
        Input::new(p.PIN_11, Pull::Down),
        Input::new(p.PIN_12, Pull::Down),
        Input::new(p.PIN_13, Pull::Down),
        Input::new(p.PIN_14, Pull::Down),
        Input::new(p.PIN_15, Pull::Down),
        Input::new(p.PIN_16, Pull::Down),
        Input::new(p.PIN_17, Pull::Down),
        Input::new(p.PIN_18, Pull::Down),
        Input::new(p.PIN_19, Pull::Down),
        Input::new(p.PIN_20, Pull::Down),
        Input::new(p.PIN_21, Pull::Down),
        Input::new(p.PIN_22, Pull::Down),
        Input::new(p.PIN_23, Pull::Down),
        Input::new(p.PIN_24, Pull::Down),
        Input::new(p.PIN_25, Pull::Down),
        Input::new(p.PIN_26, Pull::Down),
        Input::new(p.PIN_27, Pull::Down),
        Input::new(p.PIN_28, Pull::Down),
        Input::new(p.PIN_29, Pull::Down),
        Input::new(p.PIN_30, Pull::Down),
        Input::new(p.PIN_31, Pull::Down),
        Input::new(p.PIN_32, Pull::Down),
        Input::new(p.PIN_33, Pull::Down),
        Input::new(p.PIN_34, Pull::Down),
        Input::new(p.PIN_35, Pull::Down),
        Input::new(p.PIN_36, Pull::Down),
        Input::new(p.PIN_37, Pull::Down),
        Input::new(p.PIN_38, Pull::Down),
        Input::new(p.PIN_39, Pull::Down),
        Input::new(p.PIN_40, Pull::Down),
        Input::new(p.PIN_41, Pull::Down),
        Input::new(p.PIN_42, Pull::Down),
        Input::new(p.PIN_43, Pull::Down),
        Input::new(p.PIN_44, Pull::Down),
        Input::new(p.PIN_45, Pull::Down),
        Input::new(p.PIN_46, Pull::Down),
        Input::new(p.PIN_47, Pull::Down),
    ];

    loop {
        Timer::after_millis(250).await;
    }
}
