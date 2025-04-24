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
        Input::new(p.PIN_0, Pull::None),
        Input::new(p.PIN_1, Pull::None),
        Input::new(p.PIN_2, Pull::None),
        Input::new(p.PIN_3, Pull::None),
        Input::new(p.PIN_4, Pull::None),
        Input::new(p.PIN_5, Pull::None),
        Input::new(p.PIN_6, Pull::None),
        Input::new(p.PIN_7, Pull::None),
        Input::new(p.PIN_8, Pull::None),
        Input::new(p.PIN_9, Pull::None),
        Input::new(p.PIN_10, Pull::None),
        Input::new(p.PIN_11, Pull::None),
        Input::new(p.PIN_12, Pull::None),
        Input::new(p.PIN_13, Pull::None),
        Input::new(p.PIN_14, Pull::None),
        Input::new(p.PIN_15, Pull::None),
        Input::new(p.PIN_16, Pull::None),
        Input::new(p.PIN_17, Pull::None),
        Input::new(p.PIN_18, Pull::None),
        Input::new(p.PIN_19, Pull::None),
        Input::new(p.PIN_20, Pull::None),
        Input::new(p.PIN_21, Pull::None),
        Input::new(p.PIN_22, Pull::None),
        Input::new(p.PIN_23, Pull::None),
        Input::new(p.PIN_24, Pull::None),
        Input::new(p.PIN_25, Pull::None),
        Input::new(p.PIN_26, Pull::None),
        Input::new(p.PIN_27, Pull::None),
        Input::new(p.PIN_28, Pull::None),
        Input::new(p.PIN_29, Pull::None),
        Input::new(p.PIN_30, Pull::None),
        Input::new(p.PIN_31, Pull::None),
        Input::new(p.PIN_32, Pull::None),
        Input::new(p.PIN_33, Pull::None),
        Input::new(p.PIN_34, Pull::None),
        Input::new(p.PIN_35, Pull::None),
        Input::new(p.PIN_36, Pull::None),
        Input::new(p.PIN_37, Pull::None),
        Input::new(p.PIN_38, Pull::None),
        Input::new(p.PIN_39, Pull::None),
        Input::new(p.PIN_40, Pull::None),
        Input::new(p.PIN_41, Pull::None),
        Input::new(p.PIN_42, Pull::None),
        Input::new(p.PIN_43, Pull::None),
        Input::new(p.PIN_44, Pull::None),
        Input::new(p.PIN_45, Pull::None),
        Input::new(p.PIN_46, Pull::None),
        Input::new(p.PIN_47, Pull::None),
    ];

    loop {
        Timer::after_millis(250).await;
    }
}