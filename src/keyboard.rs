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
pub enum CodeKey {
    KEY0,
    KEY1,
    KEY10,
    KEY11,
    KEY12,
    KEY2,
    KEY3,
    KEY4,
    KEY5,
    KEY6,
    KEY7,
    KEY8,
    KEY9,
}

pub struct Keyboard {
    keypad: Keypad,
    pub key_event: KeyEvent,
}
use stm32f1xx_hal::gpio::gpioa::Parts;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ModifierKey {
    SHIFT,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum FunctionKey {
    FN1,
    FN2,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum NavKey {
    BACK,
    FORWARD,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Key {
    NavKey(NavKey),
    CodeKey(CodeKey),
    FunctionKey(FunctionKey),
    ModifierKey(ModifierKey),
}

#[derive(Debug)]
pub struct KeyEvent {
    pub code: Option<CodeKey>,
    pub nav: Option<NavKey>,
    pub modifier: Option<ModifierKey>,
    pub function: Option<FunctionKey>,
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
            key_event: KeyEvent {
                code: None,
                nav: None,
                modifier: None,
                function: None,
            },
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

    // match key to note
    pub fn match_note(&self, key: CodeKey) -> Option<Note> {
        match (key) {
            CodeKey::KEY0 => Some(Note::C),
            CodeKey::KEY1 => Some(Note::D),
            CodeKey::KEY2 => Some(Note::E),
            CodeKey::KEY3 => Some(Note::F),
            CodeKey::KEY4 => Some(Note::G),
            CodeKey::KEY5 => Some(Note::A),
            CodeKey::KEY6 => Some(Note::B),
            CodeKey::KEY7 => Some(Note::C),
            CodeKey::KEY8 => Some(Note::Db),
            CodeKey::KEY9 => Some(Note::Eb),
            CodeKey::KEY10 => Some(Note::Gb),
            CodeKey::KEY11 => Some(Note::Ab),
            CodeKey::KEY12 => Some(Note::Bb),
        }
    }

    // match key to step index
    pub fn match_step(&self, key: CodeKey) -> Option<usize> {
        match (key) {
            CodeKey::KEY0 => Some(0),
            CodeKey::KEY1 => Some(1),
            CodeKey::KEY2 => Some(2),
            CodeKey::KEY3 => Some(3),
            CodeKey::KEY4 => Some(4),
            CodeKey::KEY5 => Some(5),
            CodeKey::KEY6 => Some(6),
            CodeKey::KEY7 => Some(7),
            CodeKey::KEY8 => Some(8),
            CodeKey::KEY9 => Some(9),
            CodeKey::KEY10 => Some(10),
            CodeKey::KEY11 => Some(11),
            CodeKey::KEY12 => Some(12),
        }
    }

    pub fn read(&mut self) -> &mut Self {
        self.key_event.code = None;
        self.key_event.modifier = None;
        self.key_event.nav = None;
        self.key_event.function = None;

        for (row_index, row) in self.keypad.decompose().iter().enumerate() {
            for (col_index, k) in row.iter().enumerate() {
                if k.is_low().unwrap() {
                    rprintln!("Pressed: ({}, {})", row_index, col_index);

                    match (row_index, col_index) {
                        (0,0)=> {
                            self.key_event.function = Some(FunctionKey::FN1)
                        },
                        (0,1)=> {
                            self.key_event.modifier = Some(ModifierKey::SHIFT)
                        },
                        (0,2)=> {
                            self.key_event.code = Some(CodeKey::KEY0)
                        },
                        (1,0)=> {
                            self.key_event.code = Some(CodeKey::KEY8)
                        },
                        (1,1)=> {
                            self.key_event.code = Some(CodeKey::KEY1)
                        },
                        (1,2)=> {
                            self.key_event.code = Some(CodeKey::KEY2)
                        },
                        (2,0)=> {
                            self.key_event.code = Some(CodeKey::KEY9)
                        },
                        (2,1)=> {
                            self.key_event.code = Some(CodeKey::KEY3)
                        },
                        (2,2)=> {
                            self.key_event.code = Some(CodeKey::KEY4)
                        },
                        (3,0)=> {
                            self.key_event.code = Some(CodeKey::KEY10)
                        },
                        (3,1)=> {
                            self.key_event.code = Some(CodeKey::KEY5)
                        },
                        (3,2)=> {
                            self.key_event.code = Some(CodeKey::KEY6)
                        },
                        (4,0)=> {
                            self.key_event.code = Some(CodeKey::KEY11)
                        },
                        (4,1)=> {
                            self.key_event.code = Some(CodeKey::KEY12)
                        },
                        (4,2)=> {
                            self.key_event.code = Some(CodeKey::KEY7)
                        },
                        (5,0)=> {
                            // there is a bug there, for some reason when
                            // pressing Fn1 and Forward key, Fn2 also appear
                            // so to prevent it to override, only set if
                            // FN1 hasn't been pressed already
                            if self.key_event.function == None {
                                self.key_event.function = Some(FunctionKey::FN2)
                            }
                        },
                        (5,1)=> {
                            self.key_event.nav = Some(NavKey::BACK)
                        },
                        (5,2)=> {
                            self.key_event.nav = Some(NavKey::FORWARD)
                        },
                        (_, _) => todo!(),
                    }
                }
            }
        }

        return self;
    }
}
