#![allow(unused_imports)]
#![deny(unsafe_code)]
#![no_main]
#![no_std]

use panic_rtt_target as _;

use rtic::app;
use rtt_target::{rprintln, rtt_init_print};

use core::marker::PhantomData;
use embedded_hal::spi::{Mode, Phase, Polarity};
use stm32f1xx_hal::{
    adc,
    gpio::{
        gpioa::{PA0, PA1, PA10, PA2, PA3, PA4, PA5, PA6, PA7, PA8, PA9},
        gpiob::{PB0, PB12, PB13, PB14, PB15},
        Alternate, Analog, Floating, Input, OpenDrain, Output, Pin, PullUp, PushPull, CRH, CRL,
    },
    pac,
    prelude::*,
    spi::{NoMiso, NoSck, Spi, Spi1NoRemap, Spi2NoRemap},
};

use keypad::{keypad_new, keypad_struct, KeypadInput};
use mcp49xx::{Command, Mcp49xx, MODE_0};
use ws2812_spi as ws2812;

pub const SPI_MODE: Mode = Mode {
    phase: Phase::CaptureOnSecondTransition,
    polarity: Polarity::IdleHigh,
};

mod constants;
mod keyboard;
mod led;
mod sequencer;
mod track;

#[rtic::app(device = stm32f1xx_hal::pac, peripherals = true, dispatchers = [SPI1])]
mod app {
    use super::*;
    use crate::pac::SPI2;
    use mcp49xx::marker::{Buffered, Resolution12Bit, SingleChannel};
    use systick_monotonic::*;

    use constants::*;
    use track::*;

    use keyboard::{Keyboard, Keypad};
    use led::LedDriver;
    use sequencer::*;

    #[shared]
    struct Shared {
        current_track: usize,
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
        led_driver: LedDriver,
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
        tracks: [Track; TRACKS_COUNT],
    }

    #[local]
    struct Local {
        step_length: fugit::Duration<u64, 1, 100>,
        gate_length: fugit::Duration<u64, 1, 100>,
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
        let mut gpiob = cx.device.GPIOB.split();

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
            SPI_MODE,
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

        let current_track = 0;
        let tracks = [track::Track::new(); TRACKS_COUNT];

        // define step length
        let step_length_us = (60.0 / BPM) * 1000.0 * 1000.0;
        // define gate length
        let gate_length_us = step_length_us * GATE_LENGTH;
        rprintln!(
            "Step duration: {:?}us, gate on duration: {:?}us",
            step_length_us,
            gate_length_us
        );

        let step_length = systick_monotonic::ExtU64::micros(step_length_us as u64);
        let gate_length = systick_monotonic::ExtU64::micros(gate_length_us as u64);

        // setup keyboard using led matrix schema
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

        // start processes
        tick::spawn_after(step_length, mono.now()).unwrap();
        keyboard_ctrl::spawn_after(systick_monotonic::ExtU64::millis(KEYBOARD_REFRESH_MS)).unwrap();
        led_ctrl::spawn_after(systick_monotonic::ExtU64::millis(LED_REFRESH_MS)).unwrap();

        (
            Shared {
                current_track,
                dac1,
                dac2,
                led_driver,
                spi_dac,
                tracks,
            },
            Local {
                gate_length,
                keyboard,
                step_length,
            },
            init::Monotonics(mono),
        )
    }

    extern "Rust" {
        #[task(local = [keyboard], shared = [tracks, led_driver,  current_track])]
        fn keyboard_ctrl(cx: keyboard_ctrl::Context);

        #[task(priority = 1, local = [step_length, gate_length], shared = [tracks, current_track])]
        fn tick(cx: tick::Context, instant: fugit::TimerInstantU64<100>);

        #[task(shared = [tracks, led_driver, current_track])]
        fn led_ctrl(cx: led_ctrl::Context);

        #[task(shared = [tracks, dac1, dac2, spi_dac])]
        fn cv_ctrl(cx: cv_ctrl::Context);

        #[task(shared = [tracks, dac1, dac2, spi_dac])]
        fn gate_reset(cx: gate_reset::Context);
    }
}
