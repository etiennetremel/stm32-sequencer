#![allow(unreachable_code)]
#![allow(unused_imports)]
#![allow(warnings)]
#![deny(unsafe_code)]
#![feature(exclusive_range_pattern)]
#![no_main]
#![no_std]

use panic_rtt_target as _;

use rtic::app;
use rtt_target::{rprintln, rtt_init_print};

use stm32f1xx_hal::gpio::{
    gpioa::{PA0, PA1, PA10, PA2, PA3, PA4, PA5, PA6, PA7, PA8, PA9},
    gpiob::{PB0, PB12, PB13, PB14, PB15},
    Alternate, Analog, Floating, Input, OpenDrain, Output, Pin, PullUp, PushPull, CRH, CRL,
};
use keypad::{keypad_new, keypad_struct, KeypadInput};
use stm32f1xx_hal::{
    prelude::*,
    adc,
    pac,
    spi::{NoMiso, NoSck, Spi, Spi1NoRemap, Spi2NoRemap},
};

use core::marker::PhantomData;

use mcp49xx::{Command, Mcp49xx, MODE_0};

use embedded_hal::spi::{Mode, Phase, Polarity};

pub const MODE: Mode = Mode {
    phase: Phase::CaptureOnSecondTransition,
    polarity: Polarity::IdleHigh,
};

use ws2812_spi as ws2812;

mod constants;
mod keyboard;
mod sequencer;
mod track;
mod led;

#[rtic::app(device = stm32f1xx_hal::pac, peripherals = true, dispatchers = [SPI1])]
mod app {
    use super::*;
    use crate::pac::SPI2;
    use mcp49xx::marker::{Buffered, Resolution12Bit, SingleChannel};
    use systick_monotonic::*;

    use track::*;
    use constants::*;

    use keyboard::{Keyboard, Keypad, Key};
    use sequencer::{reset_gate, tick, RecordingMode};
    use led::LedDriver;

    #[shared]
    struct Shared {
        current_track: usize,
        gate1: PB0<Output<PushPull>>,
        led_driver: LedDriver,
        tracks: [Track; TRACKS_COUNT],
        recording_cursor: usize, // when recording in CV mode, keep state of currently recorded step
        recording_mode: RecordingMode,
    }

    #[local]
    struct Local {
        duration: fugit::Duration<u64, 1, 100>,
        trigger_length: fugit::Duration<u64, 1, 100>,
        adc1: adc::Adc<pac::ADC1>,
        dac1: Mcp49xx<
            Pin<Output<PushPull>, CRH, 'B', 12>,
            Spi<
                SPI2,
                Spi2NoRemap,
                (
                    Pin<Alternate<PushPull>, CRH, 'B', 13>,
                    NoMiso,
                    Pin<Alternate<PushPull>, CRH, 'B', 15>,
                ),
                u8,
            >,
            Resolution12Bit,
            SingleChannel,
            Buffered,
        >,
        dac2: Mcp49xx<
            Pin<Output<PushPull>, CRH, 'B', 14>,
            Spi<
                SPI2,
                Spi2NoRemap,
                (
                    Pin<Alternate<PushPull>, CRH, 'B', 13>,
                    NoMiso,
                    Pin<Alternate<PushPull>, CRH, 'B', 15>,
                ),
                u8,
            >,
            Resolution12Bit,
            SingleChannel,
            Buffered,
        >,
        spi_dac: Spi<
            SPI2,
            Spi2NoRemap,
            (
                Pin<Alternate<PushPull>, CRH, 'B', 13>,
                NoMiso,
                Pin<Alternate<PushPull>, CRH, 'B', 15>,
            ),
            u8,
        >,
        keyboard: Keyboard,
    }

    #[monotonic(binds = SysTick, default = true)]
    type MonoTimer = Systick<100>; // 100 Hz / 10 ms granularity

    #[init]
    fn init(cx: init::Context) -> (Shared, Local, init::Monotonics) {
        rtt_init_print!();
        rprintln!("init");

        let mut flash = cx.device.FLASH.constrain();
        let rcc = cx.device.RCC.constrain();
        let mut afio = cx.device.AFIO.constrain();

        let clocks = rcc
            .cfgr
            .sysclk(72.MHz())
            .pclk1(24.MHz())
            .freeze(&mut flash.acr);

        let mut gpioa = cx.device.GPIOA.split();

        let mut adc1 = adc::Adc::adc1(cx.device.ADC1, clocks);

        let mut gpiob = cx.device.GPIOB.split();

        // init gate1
        let gate1 = gpiob.pb0.into_push_pull_output(&mut gpiob.crl);

        // LED matrix
        let pins_led = (
            NoSck,
            NoMiso,
            gpioa.pa7.into_alternate_push_pull(&mut gpioa.crl),
        );

        let spi_led = Spi::spi1(
            cx.device.SPI1,
            pins_led,
            &mut afio.mapr,
            MODE,
            3.MHz(),
            clocks,
        );

        let led_driver = LedDriver::new(spi_led);

        // DAC
        let pins_dac = (
            gpiob.pb13.into_alternate_push_pull(&mut gpiob.crh), // SCK
            NoMiso,
            gpiob.pb15.into_alternate_push_pull(&mut gpiob.crh), // MOSI
        );

        let mut spi_dac = Spi::spi2(cx.device.SPI2, pins_dac, MODE_0, 8.MHz(), clocks);

        let mut cs_dac1 = gpiob.pb12.into_push_pull_output(&mut gpiob.crh);
        let mut cs_dac2 = gpiob.pb14.into_push_pull_output(&mut gpiob.crh);
        cs_dac1.set_high();
        cs_dac2.set_high();

        let mut dac1 = Mcp49xx::new_mcp4921(cs_dac1);
        let mut dac2 = Mcp49xx::new_mcp4921(cs_dac2);
        let cmd = Command::default();
        dac1.send(&mut spi_dac, cmd).unwrap();
        dac2.send(&mut spi_dac, cmd).unwrap();

        // systick
        let systick = cx.core.SYST;
        let mut mono = Systick::new(systick, 72_000_000);

        let mut current_track = 0;
        let mut tracks = [track::Track::new(); TRACKS_COUNT];

        let d = (60.0 / BPM) * 1000.0 * 1000.0;
        let t = d / 2.0;
        rprintln!("BPM: {:?}, length: {:?}", d, t);

        let duration = systick_monotonic::ExtU64::micros(d as u64);
        let trigger_length = systick_monotonic::ExtU64::micros(t as u64);
        let recording_cursor = 0;
        let recording_mode = RecordingMode::Step;

        let keyboard = Keyboard::new(
            gpioa.pa0.into_pull_up_input(&mut gpioa.crl),
            gpioa.pa1.into_pull_up_input(&mut gpioa.crl),
            gpioa.pa2.into_pull_up_input(&mut gpioa.crl),
            gpioa.pa3.into_pull_up_input(&mut gpioa.crl),
            gpioa.pa4.into_pull_up_input(&mut gpioa.crl),
            gpioa.pa5.into_pull_up_input(&mut gpioa.crl),
            gpioa.pa8.into_open_drain_output(&mut gpioa.crh),
            gpioa.pa9.into_open_drain_output(&mut gpioa.crh),
            gpioa.pa10.into_open_drain_output(&mut gpioa.crh),
        );

        tick::spawn_after(duration, mono.now()).unwrap();
        inputs::spawn_after(systick_monotonic::ExtU64::micros(5));
        // cv_out::spawn_after(systick_monotonic::ExtU64::millis(100));

        (
            Shared {
                current_track,
                gate1,
                led_driver,
                tracks,
                recording_cursor,
                recording_mode,
            },
            Local {
                duration,
                keyboard,
                trigger_length,
                dac1,
                dac2,
                spi_dac,
                adc1,
            },
            init::Monotonics(mono),
        )
    }

    // #[task(local = [dac1, dac2, spi_dac])]
    // fn cv_out(cx: cv_out::Context) {
    //     let cmd = Command::default();
    //
    //     let mut rnd = StdRand::default();
    //     let val = rnd.next_range(0..4080);
    //     let val2 = rnd.next_range(0..4080);
    //
    //     rprintln!("Set position {:?} and {:?}", val, val2);
    //
    //     cx.local
    //         .dac1
    //         .send(cx.local.spi_dac, cmd.value(val))
    //         .unwrap();
    //     cx.local
    //         .dac2
    //         .send(cx.local.spi_dac, cmd.value(val2))
    //         .unwrap();
    //     cv_out::spawn_after(systick_monotonic::ExtU64::millis(500)).unwrap();
    // }

    #[task(local = [adc1, keyboard], shared = [tracks, led_driver,  current_track, recording_cursor, recording_mode])]
    fn inputs(cx: inputs::Context) {
        let (fn_key, shift_key, nav_key, note_key) = cx.local.keyboard.read();

        (cx.shared.tracks, cx.shared.current_track, cx.shared.recording_cursor, cx.shared.recording_mode).lock(|tracks, current_track, recording_cursor, recording_mode| {
            // switch track mode (CV vs Gate)
            if shift_key == Key::Shift && nav_key == Key::Forward {
                rprintln!("Pressed Shift+Forward");
                tracks[*current_track].toggle_mode();
                inputs::spawn_after(systick_monotonic::ExtU64::millis(KEYBOARD_KEY_PRESS_DELAY_MS));
                return;
            }

            // switch track
            let step = cx.local.keyboard.match_step(note_key);
            if fn_key == Key::Fn1 && step > -1 {
                rprintln!("Pressed Fn1+{:?}", step);
                *current_track = note_key as usize;
                inputs::spawn_after(systick_monotonic::ExtU64::millis(KEYBOARD_KEY_PRESS_DELAY_MS));
                return;
            }

            // randomize gates with probability based on the key pressed
            if fn_key == Key::Fn2 && step >= 0 && step <= 8 {
                rprintln!("Pressed Fn2+{}", step);
                tracks[*current_track].randomize(step as f64 / 8.0);
                inputs::spawn_after(systick_monotonic::ExtU64::millis(KEYBOARD_KEY_PRESS_DELAY_MS));
                return;
            }

            // switch recording mode
            if fn_key == Key::Fn1 && shift_key == Key::Shift {
                if *recording_mode == RecordingMode::Note {
                    *recording_mode = RecordingMode::Step
                } else {
                    *recording_mode = RecordingMode::Note
                }
                rprintln!("Switch recording mode {:?}", *recording_mode);
                inputs::spawn_after(systick_monotonic::ExtU64::millis(KEYBOARD_KEY_PRESS_DELAY_MS));
                return;
            }

            let note = cx.local.keyboard.match_note(note_key);
            if *recording_mode == RecordingMode::Note && note != Note::Unknown {
                rprintln!("Pressed note {:?}", note);
                tracks[*current_track].set_note(*recording_cursor, note);
                if *recording_cursor == tracks[*current_track].pattern.len() {
                    *recording_cursor = 0;
                } else {
                    *recording_cursor += 1;
                }
                inputs::spawn_after(systick_monotonic::ExtU64::millis(KEYBOARD_KEY_PRESS_DELAY_MS));
                return;
            }

            // toggle step
            if step >= 0 && step <= 8 {
                tracks[*current_track].toggle_step(step as usize);
                inputs::spawn_after(systick_monotonic::ExtU64::millis(KEYBOARD_KEY_PRESS_DELAY_MS));
                return;
            }
        });
        inputs::spawn_after(systick_monotonic::ExtU64::millis(KEYBOARD_REFRESH_MS));
    }

    // #[task(shared = [tracks, led_driver,  current_track, recording_cursor, recording_mode])]
    // fn led(cx: led::Context) {
    //
    // }

    extern "Rust" {
        #[task(local = [duration, trigger_length], shared = [tracks, led_driver, current_track, recording_cursor, recording_mode])]
        fn tick(cx: tick::Context, instant: fugit::TimerInstantU64<100>);

        #[task(shared = [gate1])]
        fn reset_gate(cx: reset_gate::Context);
    }
}
