use crate::ws2812::Ws2812;
use ws2812_spi as ws2812;
use stm32f1xx_hal::{
    spi::{NoMiso, NoSck, Spi, Spi1NoRemap},
    gpio::{
        Alternate, Pin, PullUp, PushPull, CRL,
    }
};
use crate::constants::*;
use smart_leds::{SmartLedsWrite, RGB};
use crate::keyboard::Key;
use crate::track::{Mode, Note};

pub struct LedDriver {
    pub ws: Ws2812<
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
    pub fn new(spi: Spi<
            stm32f1xx_hal::pac::SPI1,
            Spi1NoRemap,
            (NoSck, NoMiso, Pin<Alternate<PushPull>, CRL, 'A', 7>),
            u8,
        >) -> LedDriver {
        let mut led_driver = LedDriver{
            leds: [RGB::default(); LED_COUNT],
            ws: Ws2812::new(spi),
        };
        led_driver.clear().write();
        return led_driver;
    }

    pub fn set_note(&mut self, index: usize, note: Note) -> &mut Self {
        self.leds[match_step_to_led(index)] = match_note_to_color(note);
        return self;
    }

    // set active gate
    pub fn set_gate_on(&mut self, index: usize) -> &mut Self {
        self.leds[match_step_to_led(index)] = LED_GATE_COLOR;
        return self;
    }

    // set active track color on Fn1 key
    pub fn set_active_track(&mut self, index: usize) -> &mut Self {
        self.leds[match_key_to_led(Key::Fn1)] = LED_ACTIVE_TRACK_COLOR[index];
        return self;
    }

    // set clock
    pub fn set_clock(&mut self, index: usize) -> &mut Self {
        self.leds[match_step_to_led(index)] = LED_CLOCK_COLOR;
        return self;
    }

    // set active track move on Shift key
    pub fn set_track_mode(&mut self, track_mode: Mode) -> &mut Self {
        if track_mode == Mode::CV {
            self.leds[match_key_to_led(Key::Shift)] = LED_TRACK_CV_MODE_COLOR;
        } else {
            self.leds[match_key_to_led(Key::Shift)] = LED_TRACK_GATE_MODE_COLOR;
        }
        return self;
    }

    // switch off all lights (clock and gate and button state)
    pub fn clear(&mut self) -> &mut Self {
        for i in 0..LED_COUNT {
            self.leds[i as usize] = LED_OFF_COLOR;
        }
        return self;
    }

    pub fn write(&mut self) {
        self.ws.write(self.leds.iter().cloned()).unwrap();
    }
}

pub fn match_note_to_color(note: Note) -> RGB<u8> {
    match (note) {
        Note::A => LED_NOTE_COLOR_A,
        Note::B => LED_NOTE_COLOR_B,
        Note::C => LED_NOTE_COLOR_C,
        Note::D => LED_NOTE_COLOR_D,
        Note::E => LED_NOTE_COLOR_E,
        Note::F => LED_NOTE_COLOR_F,
        Note::G => LED_NOTE_COLOR_G,
        Note::Ab => LED_NOTE_COLOR_AB,
        Note::Bb => LED_NOTE_COLOR_BB,
        Note::Db => LED_NOTE_COLOR_DB,
        Note::Eb => LED_NOTE_COLOR_EB,
        Note::Gb => LED_NOTE_COLOR_GB,
        _ => todo!(),
    }
}

pub fn match_step_to_led(index: usize) -> usize {
    match (index) {
        0 => 8,
        1 => 9,
        2 => 10,
        3 => 11,
        4 => 12,
        5 => 13,
        6 => 14,
        7 => 15,
        _ => todo!(),
    }
}

pub fn match_key_to_led(key: Key)-> usize {
    match (key) {
        Key::Fn1=> 0,
        Key::Fn2=> 7,
        Key::Shift=> 8,
        Key::Back=> 16,
        Key::Forward=> 17,
        Key::K0=> 8,
        Key::K1=> 9,
        Key::K2=> 10,
        Key::K3=> 11,
        Key::K4=> 12,
        Key::K5=> 13,
        Key::K6=> 14,
        Key::K7=> 15,
        Key::K8=> 1,
        Key::K9=> 2,
        Key::K10=> 3,
        Key::K11=> 4,
        Key::K12=> 5,
        Key::Unknown=> 0,
    }
}

pub fn match_note_to_led(note: Note)-> usize {
    match (note) {
        Note::C=> 8,
        Note::D=> 9,
        Note::E=> 10,
        Note::F=> 11,
        Note::G=> 12,
        Note::A=> 13,
        Note::B=> 14,
        Note::Db=> 1,
        Note::Eb=> 2,
        Note::Gb=> 3,
        Note::Ab=> 4,
        Note::Bb=> 5,
        Note::Unknown=> 0,
    }
}
