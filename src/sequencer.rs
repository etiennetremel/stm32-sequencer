use crate::app;
use embedded_hal::adc::OneShot;
use rtic::mutex_prelude::*;
use rtt_target::rprintln;
use smart_leds::RGB;
use systick_monotonic::*;

use crate::track::{Note, Track, Mode};
use crate::constants::*;
use crate::led::*;
use crate::keyboard::Key;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum RecordingMode {
    Step,
    Note,
}

pub(crate) fn tick(mut cx: app::tick::Context, instant: fugit::TimerInstantU64<100>) {
    (
        cx.shared.led_driver,
        cx.shared.tracks,
        cx.shared.current_track,
        cx.shared.recording_cursor,
        cx.shared.recording_mode,
    )
        .lock(|led_driver, tracks, current_track, recording_cursor, recording_mode| {
            // move 1 step forward on each tracks
            for i in 0..TRACKS_COUNT {
                tracks[i].tick();
            }

            let mut track = tracks[*current_track];
            if *recording_mode == RecordingMode::Note {
                cv_recording(led_driver, track, *current_track, *recording_cursor);
            } else {
                gate_recording(led_driver, track, *current_track);
            }
            rprintln!("TRACK1: {:?}", tracks[0]);
            // rprintln!("LED: {:?}", led_driver.leds);
        });

    app::reset_gate::spawn_at(instant + *cx.local.trigger_length).unwrap();

    let next_instant = instant + *cx.local.duration;
    // rprintln!(
    //     "half {:?}, next: {:?}",
    //     instant + *cx.local.trigger_length,
    //     next_instant
    // );
    app::tick::spawn_at(next_instant, next_instant).unwrap();
}

fn cv_recording(led_driver: &mut LedDriver, track: Track, current_track: usize, recording_cursor: usize) {
    led_driver.clear();

    // button state
    for step in 0..recording_cursor {
        led_driver.set_note(step, track.pattern[current_track].note);
    }

    // leds[match_note_to_led(track.pattern[track.cursor].note)] = RGB {b: 0x10, r: 0x10, g: 0x10};

    led_driver
        .set_active_track(current_track)
        .set_track_mode(track.mode)
        .set_clock(track.cursor);
}

fn gate_recording(led_driver: &mut LedDriver, track: Track, current_track: usize) {
    led_driver.clear();

    // button state
    for step in 0..track.pattern.len() {
        if track.pattern[step].gate {
            led_driver.set_gate_on(step);
        }
    }

    // leds[match_note_to_led(track.pattern[track.cursor].note)] = RGB {b: 0x10, r: 0x10, g: 0x10};

    led_driver
        .set_active_track(current_track)
        .set_track_mode(track.mode)
        .set_clock(track.cursor);
}

pub(crate) fn reset_gate(mut cx: app::reset_gate::Context) {
    cx.shared.gate1.lock(|gate1| {
        gate1.set_low();
    });
}
