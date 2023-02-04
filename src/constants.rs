use smart_leds::RGB;

// sequencer
pub const STEPS_COUNT: usize = 8;
pub const TRACKS_COUNT: usize = 8;
pub const BPM: f64 = 120.0;
pub const GATE_LENGTH: f64 = 0.5;

// keyboard
pub const KEYBOARD_KEY_COUNT: usize = 18;
pub const KEYBOARD_KEY_PRESS_DELAY_MS: u64 = 250;
pub const KEYBOARD_REFRESH_MS: u64 = 50;

// cv out
pub const CV_REFRESH_MS: u64 = 100;

// leds
pub const LED_COUNT: usize = 18;
pub const LED_REFRESH_MS: u64 = 100;

pub const LED_GATE_COLOR: RGB<u8> = RGB { r: 0x00, g: 0x00, b: 0x10 };
pub const LED_OFF_COLOR: RGB<u8> = RGB{ r: 0x00, g: 0x00, b: 0x00 };
// pub const LED_ACTIVE_NOTE_COLOR: RGB<u8> = RGB {b: 0x05, r: 0x10, g: 0x08};
pub const LED_ACTIVE_TRACK_COLOR: [RGB<u8>; TRACKS_COUNT] = [
    RGB { b: 0x05, r: 0x00, g: 0x00 },
    RGB { b: 0x05, r: 0x05, g: 0x00 },
    RGB { b: 0x05, r: 0x05, g: 0x05 },
    RGB { b: 0x10, r: 0x00, g: 0x00 },
    RGB { b: 0x10, r: 0x05, g: 0x00 },
    RGB { b: 0x10, r: 0x05, g: 0x05 },
    RGB { b: 0x10, r: 0x10, g: 0x00 },
    RGB { b: 0x10, r: 0x10, g: 0x05 },
];
pub const LED_TRACK_GATE_MODE_COLOR: RGB<u8> = RGB {b: 0x00, r: 0x10, g: 0x00};
pub const LED_TRACK_CV_MODE_COLOR: RGB<u8> = RGB {b: 0x00, r: 0x10, g: 0x10};
pub const LED_CLOCK_COLOR: RGB<u8> = RGB {b: 0x00, r: 0x00, g: 0x10};

pub const LED_NOTE_COLOR_A: RGB<u8> = RGB {b: 0x00, r: 0x00, g: 0x00};
pub const LED_NOTE_COLOR_B: RGB<u8> = RGB {b: 0x00, r: 0x00, g: 0x01};
pub const LED_NOTE_COLOR_C: RGB<u8> = RGB {b: 0x00, r: 0x01, g: 0x00};
pub const LED_NOTE_COLOR_D: RGB<u8> = RGB {b: 0x00, r: 0x01, g: 0x01};
pub const LED_NOTE_COLOR_E: RGB<u8> = RGB {b: 0x01, r: 0x00, g: 0x00};
pub const LED_NOTE_COLOR_F: RGB<u8> = RGB {b: 0x01, r: 0x00, g: 0x01};
pub const LED_NOTE_COLOR_G: RGB<u8> = RGB {b: 0x01, r: 0x01, g: 0x00};
pub const LED_NOTE_COLOR_AB: RGB<u8> = RGB {b: 0x01, r: 0x01, g: 0x01};
pub const LED_NOTE_COLOR_BB: RGB<u8> = RGB {b: 0x02, r: 0x00, g: 0x00};
pub const LED_NOTE_COLOR_DB: RGB<u8> = RGB {b: 0x02, r: 0x00, g: 0x01};
pub const LED_NOTE_COLOR_EB: RGB<u8> = RGB {b: 0x02, r: 0x01, g: 0x00};
pub const LED_NOTE_COLOR_GB: RGB<u8> = RGB {b: 0x02, r: 0x01, g: 0x01};
