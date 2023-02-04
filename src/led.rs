use crate::constants::*;
use crate::keyboard::{CodeKey, FunctionKey, Key, ModifierKey, NavKey};
use crate::track::{Note, TrackMode};
use crate::ws2812::Ws2812;
use smart_leds::{SmartLedsWrite, RGB};
use stm32f1xx_hal::{
    gpio::{Alternate, Pin, PullUp, PushPull, CRL},
    spi::{NoMiso, NoSck, Spi, Spi1NoRemap},
};
use ws2812_spi as ws2812;

pub struct LedDriver {
    ws: Ws2812<
        Spi<
            stm32f1xx_hal::pac::SPI1,
            Spi1NoRemap,
            (NoSck, NoMiso, Pin<Alternate<PushPull>, CRL, 'A', 7>),
            u8,
        >,
    >,
    pub leds: [RGB<u8>; LED_COUNT],
}

impl LedDriver {
    pub fn new(
        spi: Spi<
            stm32f1xx_hal::pac::SPI1,
            Spi1NoRemap,
            (NoSck, NoMiso, Pin<Alternate<PushPull>, CRL, 'A', 7>),
            u8,
        >,
    ) -> LedDriver {
        let mut led_driver = LedDriver {
            leds: [RGB::default(); LED_COUNT],
            ws: Ws2812::new(spi),
        };
        led_driver.clear().write();
        return led_driver;
    }

    // set current recording cursor position
    pub fn set_recording_position(&mut self, index: usize, note: Note) -> &mut Self {
        if let Some(led) = match_step_to_led(index) {
            self.leds[led] = match_note_to_color(note).unwrap();
        }
        return self;
    }

    // set note color
    pub fn set_note(&mut self, note: Note) -> &mut Self {
        if let Some(led) = match_note_to_led(note) {
            self.leds[led] = match_note_to_color(note).unwrap();
        }
        return self;
    }

    // set active gate color
    pub fn set_gate_on(&mut self, index: usize) -> &mut Self {
        if let Some(led) = match_step_to_led(index) {
            self.leds[led] = LED_GATE_COLOR;
        }
        return self;
    }

    // set active track color on Fn1 key
    pub fn set_active_track(&mut self, index: usize) -> &mut Self {
        if let Some(led) = match_key_to_led(Key::FunctionKey(FunctionKey::FN1)) {
            self.leds[led] = LED_ACTIVE_TRACK_COLOR[index];
        }
        return self;
    }

    // set current clock position
    pub fn set_clock(&mut self, index: usize) -> &mut Self {
        if let Some(led) = match_step_to_led(index) {
            self.leds[led] = LED_CLOCK_COLOR;
        }
        return self;
    }

    // set active track move under Shift key
    pub fn set_track_mode(&mut self, track_mode: TrackMode) -> &mut Self {
        if let Some(led) = match_key_to_led(Key::ModifierKey(ModifierKey::SHIFT)) {
            if track_mode == TrackMode::CV {
                self.leds[led] = LED_TRACK_CV_MODE_COLOR;
            } else {
                self.leds[led] = LED_TRACK_GATE_MODE_COLOR;
            }
        }
        return self;
    }

    // switch off all lights (clock and gate and button state)
    pub fn clear(&mut self) -> &mut Self {
        for i in 0..LED_COUNT {
            self.leds[i] = LED_OFF_COLOR;
        }
        return self;
    }

    pub fn write(&mut self) {
        self.ws.write(self.leds.iter().cloned()).unwrap();
    }
}

// return RGB color based on a given note
fn match_note_to_color(note: Note) -> Option<RGB<u8>> {
    match note {
        Note::A => Some(LED_NOTE_COLOR_A),
        Note::B => Some(LED_NOTE_COLOR_B),
        Note::C => Some(LED_NOTE_COLOR_C),
        Note::D => Some(LED_NOTE_COLOR_D),
        Note::E => Some(LED_NOTE_COLOR_E),
        Note::F => Some(LED_NOTE_COLOR_F),
        Note::G => Some(LED_NOTE_COLOR_G),
        Note::Ab => Some(LED_NOTE_COLOR_AB),
        Note::Bb => Some(LED_NOTE_COLOR_BB),
        Note::Db => Some(LED_NOTE_COLOR_DB),
        Note::Eb => Some(LED_NOTE_COLOR_EB),
        Note::Gb => Some(LED_NOTE_COLOR_GB),
    }
}

// return led position based on a step index
fn match_step_to_led(index: usize) -> Option<usize> {
    match index {
        0 => Some(8),
        1 => Some(9),
        2 => Some(10),
        3 => Some(11),
        4 => Some(12),
        5 => Some(13),
        6 => Some(14),
        7 => Some(15),
        _ => todo!(),
    }
}

// return led position based on a given key
fn match_key_to_led(key: Key) -> Option<usize> {
    match key {
        Key::FunctionKey(FunctionKey::FN1) => Some(0),
        Key::FunctionKey(FunctionKey::FN2) => Some(6),
        Key::ModifierKey(ModifierKey::SHIFT) => Some(7),
        Key::NavKey(NavKey::BACK) => Some(16),
        Key::NavKey(NavKey::FORWARD) => Some(17),
        Key::CodeKey(CodeKey::KEY0) => Some(8),
        Key::CodeKey(CodeKey::KEY1) => Some(9),
        Key::CodeKey(CodeKey::KEY2) => Some(10),
        Key::CodeKey(CodeKey::KEY3) => Some(11),
        Key::CodeKey(CodeKey::KEY4) => Some(12),
        Key::CodeKey(CodeKey::KEY5) => Some(13),
        Key::CodeKey(CodeKey::KEY6) => Some(14),
        Key::CodeKey(CodeKey::KEY7) => Some(15),
        Key::CodeKey(CodeKey::KEY8) => Some(1),
        Key::CodeKey(CodeKey::KEY9) => Some(2),
        Key::CodeKey(CodeKey::KEY10) => Some(3),
        Key::CodeKey(CodeKey::KEY11) => Some(4),
        Key::CodeKey(CodeKey::KEY12) => Some(5),
    }
}

// return led position based on a given note
fn match_note_to_led(note: Note) -> Option<usize> {
    match note {
        Note::C => Some(8),
        Note::D => Some(9),
        Note::E => Some(10),
        Note::F => Some(11),
        Note::G => Some(12),
        Note::A => Some(13),
        Note::B => Some(14),
        Note::Db => Some(1),
        Note::Eb => Some(2),
        Note::Gb => Some(3),
        Note::Ab => Some(4),
        Note::Bb => Some(5),
    }
}
