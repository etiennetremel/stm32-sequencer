use embedded_hal::adc::OneShot;
use mcp49xx::Command;
use rtic::mutex_prelude::*;
use rtt_target::rprintln;
use smart_leds::RGB;
use systick_monotonic::*;

use crate::app;
use crate::constants::*;
use crate::keyboard::*;
use crate::led::*;
use crate::track::*;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum RecordingMode {
    Step,
    Note,
}

// keyboard key detection controller
pub(crate) fn keyboard_ctrl(cx: app::keyboard_ctrl::Context) {
    cx.local.keyboard.read();

    (
        cx.shared.tracks,
        cx.shared.current_track,
        cx.shared.recording_cursor,
        cx.shared.recording_mode,
    )
        .lock(|tracks, current_track, recording_cursor, recording_mode| {
            match (
                cx.local.keyboard.key_event.function,
                cx.local.keyboard.key_event.modifier,
                cx.local.keyboard.key_event.nav,
                cx.local.keyboard.key_event.code,
            ) {
                // switch recording mode
                (Some(FunctionKey::FN1), Some(ModifierKey::SHIFT), None, None) => {
                    if *recording_mode == RecordingMode::Note {
                        rprintln!("Pressed Fn1+Shift, switched to STEP RECORDING");
                        *recording_mode = RecordingMode::Step
                    } else {
                        rprintln!("Pressed Fn1+Shift, switched to NOTE RECORDING");
                        tracks[*current_track].cursor = 0;
                        *recording_mode = RecordingMode::Note
                    }
                    app::keyboard_ctrl::spawn_after(systick_monotonic::ExtU64::millis(
                        KEYBOARD_KEY_PRESS_DELAY_MS,
                    ))
                    .unwrap();
                }

                // switch track mode (CV vs Gate)
                (Some(FunctionKey::FN1), None, Some(NavKey::FORWARD), None) => {
                    rprintln!("Pressed Fn1+Forward, toggle current track mode");
                    tracks[*current_track].toggle_mode();
                    app::keyboard_ctrl::spawn_after(systick_monotonic::ExtU64::millis(
                        KEYBOARD_KEY_PRESS_DELAY_MS,
                    ))
                    .unwrap();
                }

                // select next track
                (None, Some(ModifierKey::SHIFT), Some(NavKey::FORWARD), None) => {
                    rprintln!("Pressed Shift+Forward, select next track");
                    *current_track = if *current_track < TRACKS_COUNT - 1 {
                        *current_track + 1
                    } else {
                        0
                    };

                    app::keyboard_ctrl::spawn_after(systick_monotonic::ExtU64::millis(
                        KEYBOARD_KEY_PRESS_DELAY_MS,
                    ))
                    .unwrap();
                }

                // clear pattern for current track
                (Some(FunctionKey::FN2), Some(ModifierKey::SHIFT), None, None) => {
                    rprintln!("Pressed Fn2+Shift, clear pattern");
                    tracks[*current_track].clear();
                    app::keyboard_ctrl::spawn_after(systick_monotonic::ExtU64::millis(
                        KEYBOARD_KEY_PRESS_DELAY_MS,
                    ))
                    .unwrap();
                }

                // select previous track
                (None, Some(ModifierKey::SHIFT), Some(NavKey::BACK), None) => {
                    rprintln!("Pressed Shift+Back, select previous track");
                    *current_track = if *current_track > 0 {
                        *current_track - 1
                    } else {
                        TRACKS_COUNT - 1
                    };

                    app::keyboard_ctrl::spawn_after(systick_monotonic::ExtU64::millis(
                        KEYBOARD_KEY_PRESS_DELAY_MS,
                    ))
                    .unwrap();
                }

                // switch track
                (Some(FunctionKey::FN1), None, None, Some(CodeKey)) => {
                    if let Some(code) = cx.local.keyboard.key_event.code {
                        if let Some(step) = cx.local.keyboard.match_step(code) {
                            if cx.local.keyboard.key_event.function == Some(FunctionKey::FN1)
                                && step < TRACKS_COUNT
                            {
                                rprintln!("Pressed Fn1+{:?}, switch track", step);
                                *current_track = step;
                                app::keyboard_ctrl::spawn_after(systick_monotonic::ExtU64::millis(
                                    KEYBOARD_KEY_PRESS_DELAY_MS,
                                ))
                                .unwrap();
                            }
                        }
                    }
                }

                // randomize gates with probability based on the key pressed
                (Some(FunctionKey::FN2), None, None, Some(CodeKey)) => {
                    if let Some(code) = cx.local.keyboard.key_event.code {
                        if let Some(step) = cx.local.keyboard.match_step(code) {
                            if step <= 8 {
                                rprintln!("Pressed Fn2+{}, randomize pattern", step);
                                tracks[*current_track].randomize(step as f64 / 8.0);
                                app::keyboard_ctrl::spawn_after(systick_monotonic::ExtU64::millis(
                                    KEYBOARD_KEY_PRESS_DELAY_MS,
                                ))
                                .unwrap();
                            }
                        }
                    }
                }

                (None, None, None, Some(CodeKey)) => {
                    if let Some(code) = cx.local.keyboard.key_event.code {
                        if *recording_mode == RecordingMode::Note {
                            // write notes
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
                                app::keyboard_ctrl::spawn_after(systick_monotonic::ExtU64::millis(
                                    KEYBOARD_KEY_PRESS_DELAY_MS,
                                ))
                                .unwrap();
                            }
                        } else {
                            // toggle step
                            if let Some(step) = cx.local.keyboard.match_step(code) {
                                if step < tracks[*current_track].pattern.len() {
                                    rprintln!("Pressed Step {}", step);
                                    tracks[*current_track].toggle_step(step);
                                    app::keyboard_ctrl::spawn_after(
                                        systick_monotonic::ExtU64::millis(
                                            KEYBOARD_KEY_PRESS_DELAY_MS,
                                        ),
                                    )
                                    .unwrap();
                                }
                            }
                        }
                    }
                }

                (_, _, _, _) => {
                    app::keyboard_ctrl::spawn_after(systick_monotonic::ExtU64::millis(
                        KEYBOARD_REFRESH_MS,
                    ))
                    .unwrap();
                }
            }
        });
}

pub(crate) fn led_ctrl(cx: app::led_ctrl::Context) {
    (
        cx.shared.led_driver,
        cx.shared.tracks,
        cx.shared.current_track,
        cx.shared.recording_cursor,
        cx.shared.recording_mode,
    )
        .lock(
            |led_driver, tracks, current_track, recording_cursor, recording_mode| {
                let track = tracks[*current_track];
                if *recording_mode == RecordingMode::Note {
                    note_recording(led_driver, track, *current_track, *recording_cursor);
                } else {
                    step_recording(led_driver, track, *current_track);
                }
            },
        );

    app::led_ctrl::spawn_after(systick_monotonic::ExtU64::millis(LED_REFRESH_MS)).unwrap();
}

// tick move play cursor ahead by 1 step on each track
pub(crate) fn tick(cx: app::tick::Context, instant: fugit::TimerInstantU64<100>) {
    (cx.shared.tracks, cx.shared.current_track).lock(|tracks, current_track| {
        // move 1 step forward on each tracks
        for i in 0..TRACKS_COUNT {
            tracks[i].tick();
        }
        rprintln!("CURRENT TRACK [{:?}]", *current_track,);
    });

    app::cv_ctrl::spawn().unwrap();
    app::gate_reset::spawn_at(instant + *cx.local.gate_length).unwrap();

    // call next tick
    let next_instant = instant + *cx.local.step_length;
    app::tick::spawn_at(next_instant, next_instant).unwrap();
}

// define led lighting when in note recording mode
fn note_recording(
    led_driver: &mut LedDriver,
    track: Track,
    current_track: usize,
    recording_cursor: usize,
) {
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

// define led lighting when in step recording mode
fn step_recording(led_driver: &mut LedDriver, track: Track, current_track: usize) {
    led_driver.clear();

    // button state
    for step in 0..track.pattern.len() {
        // display current active gate
        if track.pattern[step].gate == Gate::ON {
            led_driver.set_gate_on(step);
        }
        // display note being played
        if track.mode == TrackMode::CV && track.cursor == step {
            led_driver.set_note(track.pattern[step].note);
        }
    }

    led_driver
        .set_active_track(current_track)
        .set_track_mode(track.mode)
        .set_clock(track.cursor)
        .write();
}

// write Gate/CV value for each DAC
pub(crate) fn cv_ctrl(cx: app::cv_ctrl::Context) {
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
                dac1.send(
                    spi_dac,
                    cmd.value(if step_track0.gate == Gate::ON {
                        4080
                    } else {
                        0
                    }),
                )
                .unwrap();
            } else {
                dac1.send(spi_dac, cmd.value(match_note_to_cv(step_track0.note)))
                    .unwrap();
            }

            let step_track1 = tracks[1].pattern[tracks[1].cursor];

            if tracks[1].mode == TrackMode::GATE {
                dac2.send(
                    spi_dac,
                    cmd.value(if step_track1.gate == Gate::ON {
                        4080
                    } else {
                        0
                    }),
                )
                .unwrap();
            } else {
                dac2.send(spi_dac, cmd.value(match_note_to_cv(step_track1.note)))
                    .unwrap();
            }
        });
}

// reset all gate after trigger
pub(crate) fn gate_reset(cx: app::gate_reset::Context) {
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
fn match_note_to_cv(note: Note) -> u16 {
    // TODO: adjust with correct CV value
    match note {
        Note::C => 0,
        Note::D => 400,
        Note::E => 800,
        Note::F => 1200,
        Note::G => 1600,
        Note::A => 2400,
        Note::B => 2800,
        Note::Db => 3200,
        Note::Eb => 3600,
        Note::Gb => 4080,
        Note::Ab => 4080,
        Note::Bb => 4080,
    }
}
