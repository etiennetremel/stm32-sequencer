use crate::app::set_led;
use rtic::mutex_prelude::*;
use rtt_target::rprintln;
use smart_leds::SmartLedsWrite;

use embedded_hal::adc::OneShot;

pub(crate) fn set_led(mut cx: set_led::Context) {
    cx.local
        .ws
        .write(cx.shared.leds.lock(|leds| *leds).iter().cloned())
        .unwrap();
}
