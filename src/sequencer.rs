use crate::app;
use embedded_hal::adc::OneShot;
use rtic::mutex_prelude::*;
use rtt_target::rprintln;
use smart_leds::RGB;
use systick_monotonic::*;

pub(crate) fn tick(mut cx: app::tick::Context, instant: fugit::TimerInstantU64<100>) {
    let offset = 8;

    (
        cx.shared.leds,
        cx.shared.tracks,
        cx.shared.current_track,
        cx.shared.gate1,
    )
        .lock(|leds, tracks, current_track, gate1| {
            // clear all lights (clock and gate and button state)
            for i in 0..17 {
                leds[i as usize].b = 0x00;
                leds[i as usize].r = 0x00;
                leds[i as usize].g = 0x00;
            }

            let track = &mut tracks[*current_track];

            // move forward
            track.tick();

            // button state
            for i in 0..track.pattern.len() {
                if track.pattern[i] == 255 {
                    leds[offset + i].b = 0x10;
                }
            }

            // set gate
            if track.pattern[track.cursor] == 255 {
                let mut color = match *current_track {
                    0 => RGB {
                        b: 0x05,
                        r: 0x00,
                        g: 0x00,
                    },
                    1 => RGB {
                        b: 0x05,
                        r: 0x10,
                        g: 0x00,
                    },
                    2 => RGB {
                        b: 0x05,
                        r: 0x00,
                        g: 0x10,
                    },
                    3 => RGB {
                        b: 0x10,
                        r: 0x00,
                        g: 0x10,
                    },
                    _ => RGB {
                        b: 0x00,
                        r: 0x00,
                        g: 0x00,
                    },
                };
                leds[offset + track.cursor] = color;
            }

            // set clock
            leds[offset + track.cursor].g = 0x10;

            // Gate example
            // if tracks[0].pattern[track.cursor] == 255 {
            //     gate1.set_high();
            // } else {
            //     gate1.set_low();
            // }
        });

    app::set_led::spawn().ok();
    app::reset_gate::spawn_at(instant + *cx.local.trigger_length).unwrap();

    let next_instant = instant + *cx.local.duration;
    rprintln!(
        "half {:?}, next: {:?}",
        instant + *cx.local.trigger_length,
        next_instant
    );
    app::tick::spawn_at(next_instant, next_instant).unwrap();
}

pub(crate) fn reset_gate(mut cx: app::reset_gate::Context) {
    cx.shared.gate1.lock(|gate1| {
        gate1.set_low();
    });
}
