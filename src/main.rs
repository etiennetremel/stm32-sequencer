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

use core::convert::Infallible;
use keypad::{keypad_new, keypad_struct, KeypadInput};

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

// use tinyrand::{StdRand, Wyrand};

use ws2812_spi as ws2812;

mod constants;
mod inputs;
mod outputs;
mod sequencer;
mod track;

#[rtic::app(device = stm32f1xx_hal::pac, peripherals = true, dispatchers = [SPI1])]
mod app {
    use super::*;
    use crate::pac::SPI2;
    use crate::ws2812::Ws2812;
    use mcp49xx::marker::{Buffered, Resolution12Bit, SingleChannel};
    use smart_leds::{SmartLedsWrite, RGB};
    use systick_monotonic::*;
    // use tinyrand::RandRange;

    use track::*;
    use constants::*;

    #[shared]
    struct Shared {
        leds: [RGB<u8>; 18],
        tracks: [Track; TRACKS_COUNT],
        current_track: usize,
        gate1: PB0<Output<PushPull>>,
    }

    #[local]
    struct Local {
        keypad: Keypad,
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
        ws: Ws2812<
            Spi<
                stm32f1xx_hal::pac::SPI1,
                Spi1NoRemap,
                (NoSck, NoMiso, Pin<Alternate<PushPull>, CRL, 'A', 7>),
                u8,
            >,
        >,
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

        let kp = keypad_new!(Keypad {
            rows: (
                gpioa.pa0.into_pull_up_input(&mut gpioa.crl),
                gpioa.pa1.into_pull_up_input(&mut gpioa.crl),
                gpioa.pa2.into_pull_up_input(&mut gpioa.crl),
                gpioa.pa3.into_pull_up_input(&mut gpioa.crl),
                gpioa.pa4.into_pull_up_input(&mut gpioa.crl),
                gpioa.pa5.into_pull_up_input(&mut gpioa.crl),
            ),
            columns: (
                gpioa.pa8.into_open_drain_output(&mut gpioa.crh),
                gpioa.pa9.into_open_drain_output(&mut gpioa.crh),
                gpioa.pa10.into_open_drain_output(&mut gpioa.crh),
            ),
        });

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

        let mut leds = [RGB::default(); NUM_LEDS];
        let mut ws = Ws2812::new(spi_led);

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

        // reset leds
        ws.write(leds.iter().cloned()).unwrap();

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

        tick::spawn_after(duration, mono.now()).unwrap();
        keyboard::spawn_after(systick_monotonic::ExtU64::micros(5));
        // cv_out::spawn_after(systick_monotonic::ExtU64::millis(100));

        (
            Shared {
                leds,
                tracks,
                current_track,
                gate1,
            },
            Local {
                duration,
                trigger_length,
                ws,
                dac1,
                dac2,
                spi_dac,
                adc1,
                keypad: kp,
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

    use crate::inputs::keyboard;
    use crate::outputs::set_led;
    use crate::sequencer::{reset_gate, tick};

    extern "Rust" {
        #[task(local = [adc1, keypad], shared = [leds, tracks, current_track])]
        fn keyboard(cx: keyboard::Context);

        #[task(local = [ws], shared = [leds])]
        fn set_led(cx: set_led::Context);

        #[task(local = [duration, trigger_length], shared = [leds, tracks, current_track, gate1])]
        fn tick(cx: tick::Context, instant: fugit::TimerInstantU64<100>);

        #[task(shared = [leds, gate1])]
        fn reset_gate(cx: reset_gate::Context);
    }
}
