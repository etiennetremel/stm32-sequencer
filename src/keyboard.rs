// use crate::app::keyboard;
use keypad::embedded_hal::digital::v2::InputPin;
use rtic::mutex_prelude::*;
use rtt_target::rprintln;
use embedded_hal::adc::OneShot;

use crate::constants::*;
use crate::track::{Track, Note};

use keypad::{keypad_new, keypad_struct, KeypadInput};
use core::convert::Infallible;
use stm32f1xx_hal::gpio::{
    Pin,
    CRH, CRL,
    gpioa::{PA0, PA1, PA10, PA2, PA3, PA4, PA5, PA8, PA9},
    OpenDrain, Output, Input, PullUp,
};

// initialise keyboard
keypad_struct! {
    pub struct Keypad<Error = Infallible> {
        rows: (
            PA0<Input<PullUp>>,
            PA1<Input<PullUp>>,
            PA2<Input<PullUp>>,
            PA3<Input<PullUp>>,
            PA4<Input<PullUp>>,
            PA5<Input<PullUp>>,
        ),
        columns: (
            PA8<Output<OpenDrain>>,
            PA9<Output<OpenDrain>>,
            PA10<Output<OpenDrain>>,
        ),
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Key {
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
    Fn1,
    Fn2,
    Shift,
    Back,
    Forward,
    Unknown,
}

// #[derive(Copy, Clone, Debug, PartialEq)]
// pub struct KeyDown {
//     k0: bool,
//     k1: bool,
//     k10: bool,
//     k11: bool,
//     k12: bool,
//     k2: bool,
//     k3: bool,
//     k4: bool,
//     k5: bool,
//     k6: bool,
//     k7: bool,
//     k8: bool,
//     k9: bool,
//     fn1: bool,
//     fn2: bool,
//     shift: bool,
//     back: bool,
//     forward: bool,
// }

pub struct Keyboard {
    // pub key_lock: bool,
    pub keypad: Keypad,
    // key_down: KeyDown,
}
use stm32f1xx_hal::gpio::gpioa::Parts;

pub struct Key {
    pub key: Key,
    pub state: bool,
}

impl Keyboard {
    pub fn new(
        r0: Pin<Input<PullUp>, CRL, 'A', 0>,
        r1: Pin<Input<PullUp>, CRL, 'A', 1>,
        r2: Pin<Input<PullUp>, CRL, 'A', 2>,
        r3: Pin<Input<PullUp>, CRL, 'A', 3>,
        r4: Pin<Input<PullUp>, CRL, 'A', 4>,
        r5: Pin<Input<PullUp>, CRL, 'A', 5>,
        c0: Pin<Output<OpenDrain>, CRH, 'A', 8>,
        c1: Pin<Output<OpenDrain>, CRH, 'A', 9>,
        c2: Pin<Output<OpenDrain>, CRH, 'A', 10>,
    ) -> Keyboard {
        Keyboard{
            // key_down: [Key::Unknown; KEYBOARD_KEY_COUNT],
            keypad: keypad_new!(Keypad {
                rows: (
                    r0, r1, r2, r3, r4, r5,
                ),
                columns: (
                    c0, c1, c2,
                ),
            })
        }
    }

    fn is_key_down(&self, key: Key) -> bool {
        return self.key_down.contains(&key);
    }

    // match row/column index to key
    fn match_key(&self, row_index: usize, col_index: usize) -> Key {
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

    // match key to note
    pub fn match_note(&self, key: Key) -> Note {
        match (key) {
            Key::K0 => Note::C,
            Key::K1 => Note::D,
            Key::K2 => Note::E,
            Key::K3 => Note::F,
            Key::K4 => Note::G,
            Key::K5 => Note::A,
            Key::K6 => Note::B,
            Key::K7 => Note::C,
            Key::K8 => Note::Db,
            Key::K9 => Note::Eb,
            Key::K10 => Note::Gb,
            Key::K11 => Note::Ab,
            Key::K12 => Note::Bb,
            _ => Note::Unknown,
        }
    }

    // match key to step index
    pub fn match_step(&self, key: Key) -> i32 {
        match (key) {
            Key::K0 => 0,
            Key::K1 => 1,
            Key::K2 => 2,
            Key::K3 => 3,
            Key::K4 => 4,
            Key::K5 => 5,
            Key::K6 => 6,
            Key::K7 => 7,
            Key::K8 => 8,
            Key::K9 => 9,
            Key::K10 => 10,
            Key::K11 => 11,
            Key::K12 => 12,
            _ => -1,
        }
    }

    pub fn read(&self) -> (Key, Key, Key, Key) {
        let mut fn_key: Key = Key::Unknown;
        let mut shift_key: Key = Key::Unknown;
        let mut nav_key: Key = Key::Unknown;
        let mut note_key: Key = Key::Unknown;

        for (row_index, row) in self.keypad.decompose().iter().enumerate() {
            for (col_index, k) in row.iter().enumerate() {
                if k.is_low().unwrap() {
                    rprintln!("========================================");
                    rprintln!("Pressed: ({}, {})", row_index, col_index);
                    let mut key = self.match_key(row_index, col_index);

                    match key {
                        Key::Fn1 => {
                            fn_key= Key::Fn1;
                        },
                        Key::Shift => {
                            shift_key = Key::Shift;
                        },
                        Key::K0 => {
                            note_key = Key::K0;
                        },
                        Key::K1 => {
                            note_key = Key::K1;
                        },
                        Key::K2 => {
                            note_key = Key::K2;
                        },
                        Key::K3 => {
                            note_key = Key::K3;
                        },
                        Key::K4 => {
                            note_key = Key::K4;
                        },
                        Key::K5 => {
                            note_key = Key::K5;
                        },
                        Key::K6 => {
                            note_key = Key::K6;
                        },
                        Key::K7 => {
                            note_key = Key::K7;
                        },
                        Key::K8 => {
                            note_key = Key::K8;
                        },
                        Key::K9 => {
                            note_key = Key::K9;
                        },
                        Key::K10 => {
                            note_key = Key::K10;
                        },
                        Key::K11 => {
                            key= Key::K11;
                        },
                        Key::K12 => {
                            key= Key::K12;
                        },
                        Key::Fn2 => {
                            fn_key= Key::Fn2;
                        },
                        Key::Back => {
                            nav_key= Key::Back;
                        },
                        Key::Forward => {
                            nav_key= Key::Forward;
                        },
                        _ => {},
                    };
                    rprintln!("KEY: {:?}", key);
                    rprintln!("========================================");
                }
            }
        }

        return (fn_key, shift_key, nav_key, note_key);
    }
}
