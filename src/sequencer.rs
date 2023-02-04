use crate::app;
use embedded_hal::adc::OneShot;
use rtic::mutex_prelude::*;
use rtt_target::rprintln;
use smart_leds::RGB;
use systick_monotonic::*;
use mcp49xx::Command;

use crate::track::*;
use crate::constants::*;
use crate::led::*;
use crate::keyboard::*;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum RecordingMode {
    Step,
    Note,
}

pub(crate) fn keyboard_ctrl(mut cx: app::keyboard_ctrl::Context) {
    cx.local.keyboard.read();

    (
        cx.shared.tracks,
        cx.shared.current_track,
        cx.shared.recording_cursor,
        cx.shared.recording_mode
    ).lock(|tracks, current_track, recording_cursor, recording_mode| {
        // switch recording mode
        if cx.local.keyboard.key_event.function == Some(FunctionKey::FN1) && cx.local.keyboard.key_event.modifier == Some(ModifierKey::SHIFT) {
            if *recording_mode == RecordingMode::Note {
                rprintln!("Pressed Fn1+Shift, switched to STEP RECORDING");
                *recording_mode = RecordingMode::Step
            } else {
                rprintln!("Pressed Fn1+Shift, switched to NOTE RECORDING");
                tracks[*current_track].cursor = 0;
                *recording_mode = RecordingMode::Note
            }
            app::keyboard_ctrl::spawn_after(systick_monotonic::ExtU64::millis(KEYBOARD_KEY_PRESS_DELAY_MS));
            return;
        }

        // switch track mode (CV vs Gate)
        if cx.local.keyboard.key_event.function == Some(FunctionKey::FN1) && cx.local.keyboard.key_event.nav == Some(NavKey::FORWARD) {
            rprintln!("Pressed Fn1+Forward, toggle current track mode");
            tracks[*current_track].toggle_mode();
            app::keyboard_ctrl::spawn_after(systick_monotonic::ExtU64::millis(KEYBOARD_KEY_PRESS_DELAY_MS));
            return;
        }

        // select next track
        if cx.local.keyboard.key_event.modifier == Some(ModifierKey::SHIFT) && cx.local.keyboard.key_event.nav == Some(NavKey::FORWARD) {
            rprintln!("Pressed Shift+Forward, select next track");
            *current_track = if *current_track < TRACKS_COUNT-1 {
                *current_track+1
            } else {
                0
            };

            app::keyboard_ctrl::spawn_after(systick_monotonic::ExtU64::millis(KEYBOARD_KEY_PRESS_DELAY_MS));
            return;
        }

        // clear pattern for current track
        if cx.local.keyboard.key_event.function == Some(FunctionKey::FN2) && cx.local.keyboard.key_event.modifier == Some(ModifierKey::SHIFT) {
            rprintln!("Pressed Fn2+Shift, clear pattern");
            tracks[*current_track].clear();
            app::keyboard_ctrl::spawn_after(systick_monotonic::ExtU64::millis(KEYBOARD_KEY_PRESS_DELAY_MS));
            return;
        }

        // select previous track
        if cx.local.keyboard.key_event.modifier == Some(ModifierKey::SHIFT) && cx.local.keyboard.key_event.nav == Some(NavKey::BACK) {
            rprintln!("Pressed Shift+Back, select previous track");
            *current_track = if *current_track > 0 {
                *current_track-1
            } else {
                TRACKS_COUNT-1
            };

            app::keyboard_ctrl::spawn_after(systick_monotonic::ExtU64::millis(KEYBOARD_KEY_PRESS_DELAY_MS));
            return;
        }

        if let Some(code) = cx.local.keyboard.key_event.code {
            if *recording_mode == RecordingMode::Note {
                if let Some(note) = cx.local.keyboard.match_note(code) {
                    rprintln!("Pressed note {:?}", note);
                    tracks[*current_track]
                        .set_note(*recording_cursor, note)
                        .set_gate(*recording_cursor, Gate::ON);
                    if *recording_cursor == tracks[*current_track].pattern.len() {
                        *recording_cursor = 0;
                    } else {
                        *recording_cursor += 1;
                    }
                    app::keyboard_ctrl::spawn_after(systick_monotonic::ExtU64::millis(KEYBOARD_KEY_PRESS_DELAY_MS));
                    return;
                }
            } else {
                if let Some(step) = cx.local.keyboard.match_step(code) {
                    // switch track
                    if cx.local.keyboard.key_event.function == Some(FunctionKey::FN1) && step < TRACKS_COUNT {
                        rprintln!("Pressed Fn1+{:?}, switch track", step);
                        *current_track = step;
                        app::keyboard_ctrl::spawn_after(systick_monotonic::ExtU64::millis(KEYBOARD_KEY_PRESS_DELAY_MS));
                        return;
                    }

                    // randomize gates with probability based on the key pressed
                    if cx.local.keyboard.key_event.function == Some(FunctionKey::FN2) && step <= 8 {
                        rprintln!("Pressed Fn2+{}, randomize pattern", step);
                        tracks[*current_track].randomize(step as f64 / 8.0);
                        app::keyboard_ctrl::spawn_after(systick_monotonic::ExtU64::millis(KEYBOARD_KEY_PRESS_DELAY_MS));
                        return;
                    }

                    // toggle step
                    if step < tracks[*current_track].pattern.len() {
                        rprintln!("Pressed Step {}", step);
                        tracks[*current_track].toggle_step(step);
                        app::keyboard_ctrl::spawn_after(systick_monotonic::ExtU64::millis(KEYBOARD_KEY_PRESS_DELAY_MS));
                        return;
                    }
                }
            }
        }
    });
    app::keyboard_ctrl::spawn_after(systick_monotonic::ExtU64::millis(KEYBOARD_REFRESH_MS));
}

pub(crate) fn led_ctrl(mut cx: app::led_ctrl::Context) {
    (
        cx.shared.led_driver,
        cx.shared.tracks,
        cx.shared.current_track,
        cx.shared.recording_cursor,
        cx.shared.recording_mode,
    )
        .lock(|led_driver, tracks, current_track, recording_cursor, recording_mode| {
            let mut track = tracks[*current_track];
            if *recording_mode == RecordingMode::Note {
                note_recording(led_driver, track, *current_track, *recording_cursor);
            } else {
                step_recording(led_driver, track, *current_track);
            }
        });

    app::led_ctrl::spawn_after(systick_monotonic::ExtU64::millis(LED_REFRESH_MS));
}

pub(crate) fn tick(mut cx: app::tick::Context, instant: fugit::TimerInstantU64<100>) {
    (
        cx.shared.tracks,
        cx.shared.current_track,
    )
        .lock(|tracks, current_track| {
            // move 1 step forward on each tracks
            for i in 0..TRACKS_COUNT {
                tracks[i].tick();
            }
            rprintln!(
                "CURRENT TRACK [{:?}]",
                *current_track,
            );

            // rprintln!(
            //     "CURRENT TRACK [{:?}] | {:?} | {:?} | {:?} | {:?} | {:?} | {:?} | {:?} | {:?}",
            //     *current_track,
            //     tracks[0].pattern[tracks[0].cursor],
            //     tracks[1].pattern[tracks[1].cursor],
            //     tracks[2].pattern[tracks[2].cursor],
            //     tracks[3].pattern[tracks[3].cursor],
            //     tracks[4].pattern[tracks[4].cursor],
            //     tracks[5].pattern[tracks[5].cursor],
            //     tracks[6].pattern[tracks[6].cursor],
            //     tracks[7].pattern[tracks[7].cursor],
            // );
        });

    app::cv_ctrl::spawn().unwrap();
    app::gate_reset::spawn_at(instant + *cx.local.gate_length).unwrap();

    let next_instant = instant + *cx.local.step_length;
    // rprintln!(
    //     "half {:?}, next: {:?}",
    //     instant + *cx.local.trigger_length,
    //     next_instant
    // );
    app::tick::spawn_at(next_instant, next_instant).unwrap();
}

fn note_recording(led_driver: &mut LedDriver, track: Track, current_track: usize, recording_cursor: usize) {
    led_driver.clear();

    // button state
    for step in 0..recording_cursor {
        led_driver.set_recording_position(step, track.pattern[current_track].note);
    }

    led_driver
        .set_active_track(current_track)
        .set_track_mode(track.mode)
        .set_clock(track.cursor)
        .write();
}

fn step_recording(led_driver: &mut LedDriver, track: Track, current_track: usize) {
    led_driver.clear();

    // button state
    for step in 0..track.pattern.len() {
        if track.pattern[step].gate == Gate::ON {
            led_driver.set_gate_on(step);
            if track.mode == TrackMode::CV {
                led_driver.set_note(step, track.pattern[step].note);
            }
        }
    }

    led_driver
        .set_active_track(current_track)
        .set_track_mode(track.mode)
        .set_clock(track.cursor)
        .write();
}

pub(crate) fn cv_ctrl(mut cx:app::cv_ctrl::Context) {
    let cmd = Command::default();

    (
        cx.shared.dac1,
        cx.shared.dac2,
        cx.shared.spi_dac,
        cx.shared.tracks,
    )
        .lock(|dac1, dac2, spi_dac, tracks| {
            let step_track0 = tracks[0].pattern[tracks[0].cursor];

            if tracks[0].mode == TrackMode::GATE {
                dac1.send(spi_dac, cmd.value(if step_track0.gate == Gate::ON {
                    4080
                } else {
                    0
                })).unwrap();
            } else {
                dac1.send(spi_dac, cmd.value(match_note_to_cv(step_track0.note))).unwrap();
            }

            let step_track1 = tracks[1].pattern[tracks[1].cursor];

            if tracks[1].mode == TrackMode::GATE {
                dac2.send(spi_dac, cmd.value(if step_track1.gate == Gate::ON {
                    4080
                } else {
                    0
                })).unwrap();
            } else {
                dac2.send(spi_dac, cmd.value(match_note_to_cv(step_track1.note))).unwrap();
            }


            // rprintln!(
            //     "CURRENT TRACK [{:?}] | {:?} | {:?} | {:?} | {:?} | {:?} | {:?} | {:?} | {:?}",
            //     *current_track,
            //     tracks[0].pattern[tracks[0].cursor],
            //     tracks[1].pattern[tracks[1].cursor],
        });

    // rprintln!("Set position {:?} and {:?}", val, val2);
    //
    // cx.local
    //     .dac1
    //     .send(cx.local.spi_dac, cmd.value(val))
    //     .unwrap();
    // cx.local
    //     .dac2
    //     .send(cx.local.spi_dac, cmd.value(val2))
    //     .unwrap();
    // app::cv_ctrl::spawn_after(systick_monotonic::ExtU64::millis(500)).unwrap();
}

pub(crate) fn gate_reset(mut cx: app::gate_reset::Context) {
    let cmd = Command::default();

    (
        cx.shared.dac1,
        cx.shared.dac2,
        cx.shared.spi_dac,
        cx.shared.tracks,
    )
        .lock(|dac1, dac2, spi_dac, tracks| {
            if tracks[0].mode == TrackMode::GATE {
                dac1.send(spi_dac, cmd.value(0)).unwrap();
            }

            if tracks[1].mode == TrackMode::GATE {
                dac2.send(spi_dac, cmd.value(0)).unwrap();
            }
        });
}

// return led position based on a given note
fn match_note_to_cv(note: Note)-> u16 {
    match (note) {
        Note::C=> 0,
        Note::D=> 400,
        Note::E=> 800,
        Note::F=> 1200,
        Note::G=> 1600,
        Note::A=> 2400,
        Note::B=> 2800,
        Note::Db=> 3200,
        Note::Eb=> 3600,
        Note::Gb=> 4080,
        Note::Ab=> 4080,
        Note::Bb=> 4080,
    }
}
