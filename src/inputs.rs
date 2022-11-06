use crate::app::keyboard;
use keypad::embedded_hal::digital::v2::InputPin;
use rtic::mutex_prelude::*;
use rtt_target::rprintln;
use embedded_hal::adc::OneShot;

use crate::constants::*;

#[derive(Copy, Clone, Debug)]
enum Key {
    Back,
    Fn1,
    Fn2,
    Forward,
    K0,
    K1,
    K10,
    K11,
    K12,
    K2,
    K3,
    K4,
    K5,
    K6,
    K7,
    K8,
    K9,
    Shift,
    Unknown,
}

// match key based on row/column index
fn match_key(row_index: usize, col_index: usize) -> Key {
    match (row_index, col_index) {
        (0,0)=> Key::Fn1,
        (0,1)=> Key::Shift,
        (0,2)=> Key::K0,
        (1,0)=> Key::K8,
        (1,1)=> Key::K1,
        (1,2)=> Key::K2,
        (2,0)=> Key::K9,
        (2,1)=> Key::K3,
        (2,2)=> Key::K4,
        (3,0)=> Key::K10,
        (3,1)=> Key::K5,
        (3,2)=> Key::K6,
        (4,0)=> Key::K11,
        (4,1)=> Key::K12,
        (4,2)=> Key::K7,
        (5,0)=> Key::Fn2,
        (5,1)=> Key::Back,
        (5,2)=> Key::Forward,
        _ => Key::Unknown
    }
}

pub(crate) fn keyboard(mut cx: keyboard::Context) {
    // let input_voltage = cx.local.adc1.read(&mut *cx.local.button).unwrap();
    // let key = get_key(input_voltage);
    // if key > -1 {
    //     rprintln!("Button read {:?} => {:?}", input_voltage, key);
    //     (cx.shared.tracks, cx.shared.current_track).lock(|tracks, current_track| {
    //         if key == 1 {
    //             tracks[*current_track].randomize();
    //             return
    //         }
    //         tracks[*current_track].toggle_step(key as usize);
    //     });
    //     keyboard::spawn_after(systick_monotonic::ExtU64::millis(500));
    //     return;
    // }

    (cx.shared.tracks, cx.shared.current_track).lock(|tracks, current_track| {
        // TODO: change to smarter key detection logic
        let mut pressed_key: i32 = -1;
        let mut pressed_fn_1 = false;
        let mut pressed_fn_2 = false;
        let mut pressed_shift = false;
        let mut pressed_forward = false;
        let mut pressed_back = false;

        for (row_index, row) in cx.local.keypad.decompose().iter().enumerate() {
            for (col_index, k) in row.iter().enumerate() {
                let is_pressed = if k.is_low().unwrap() { true } else { false };
                if is_pressed {
                    rprintln!("========================================");
                    rprintln!("Pressed: ({}, {})", row_index, col_index);
                    let key = match_key(row_index, col_index);

                    match key {
                        Key::Fn1 => {
                            pressed_fn_1 = true;
                        },
                        Key::Shift => {
                            pressed_shift = true;
                        },
                        Key::K0 => {
                            pressed_key = 0;
                        },
                        Key::K1 => {
                            pressed_key = 1;
                        },
                        Key::K2 => {
                            pressed_key = 2;
                        },
                        Key::K3 => {
                            pressed_key = 3;
                        },
                        Key::K4 => {
                            pressed_key = 4;
                        },
                        Key::K5 => {
                            pressed_key = 5;
                        },
                        Key::K6 => {
                            pressed_key = 6;
                        },
                        Key::K7 => {
                            pressed_key = 7;
                        },
                        Key::K8 => {
                            pressed_key = 8;
                        },
                        Key::K9 => {
                            pressed_key = 9;
                        },
                        Key::K10 => {
                            pressed_key = 10;
                        },
                        Key::K11 => {
                            pressed_key = 11;
                        },
                        Key::K12 => {
                            pressed_key = 12;
                        },
                        Key::Fn2 => {
                            pressed_fn_2 = true;
                        },
                        Key::Back => {
                            pressed_back = true;
                        },
                        Key::Forward => {
                            pressed_forward = true;
                        },
                        _ => {},
                    };
                    rprintln!("KEY: {:?}", key);
                    rprintln!("========================================");
                }
            }
        }

        // switch track
        if pressed_fn_1 && pressed_key >= 0 && pressed_key < 4 {
            rprintln!("Pressed Fn1+{}", pressed_key);
            *current_track = pressed_key as usize;
            keyboard::spawn_after(systick_monotonic::ExtU64::millis(500));
            return;
        }

        // randomize gates with probability based on the key pressed
        if pressed_fn_2 && pressed_key >= 0 {
            rprintln!("Pressed Fn2+{}", pressed_key);
            tracks[*current_track].randomize(pressed_key as f64 / 8.0);
            keyboard::spawn_after(systick_monotonic::ExtU64::millis(KEYBOARD_KEY_PRESS_DELAY_MS));
            return;
        }

        // toggle step
        if pressed_key >= 0 && pressed_key <= 8 {
            tracks[*current_track].toggle_step(pressed_key as usize);
            keyboard::spawn_after(systick_monotonic::ExtU64::millis(KEYBOARD_KEY_PRESS_DELAY_MS));
            return;
        }
        return;
    });

    keyboard::spawn_after(systick_monotonic::ExtU64::millis(KEYBOARD_REFRESH_MS));
}

// fn get_key(input_voltage: u32) -> i32 {
//     match input_voltage {
//         1000..2000=>0,
//         2000..3000=>1,
//         3000..4000=>2,
//         4000..5000=>3,
//         _ => -1,
//     }
// }
