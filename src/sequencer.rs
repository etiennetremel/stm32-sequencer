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

// keyboard key detection controller
pub(crate) fn keyboard_ctrl(cx: app::keyboard_ctrl::Context) {
    cx.local.keyboard.read();

    (cx.shared.tracks, cx.shared.current_track).lock(|tracks, current_track| {
        let track = &mut tracks[*current_track];
        let mut delay = KEYBOARD_KEY_PRESS_DELAY_MS;

        match (
            cx.local.keyboard.key_event.function,
            cx.local.keyboard.key_event.modifier,
            cx.local.keyboard.key_event.nav,
            cx.local.keyboard.key_event.code,
        ) {
            // switch recording mode
            (Some(FunctionKey::FN1), Some(ModifierKey::SHIFT), None, None) => {
                track.toggle_mode();
                rprintln!(
                    "Pressed Fn1+Shift, switched track mode to {:?}",
                    track.get_mode()
                );
            }

            // select next track
            (None, Some(ModifierKey::SHIFT), Some(NavKey::FORWARD), None) => {
                rprintln!("Pressed Shift+Forward, select next track");
                *current_track = if *current_track < TRACKS_COUNT - 1 {
                    *current_track + 1
                } else {
                    0
                };
            }

            // play current track
            (Some(FunctionKey::FN1), None, Some(NavKey::FORWARD), None) => {
                rprintln!("Pressed Fn1+Forward, toggle play/stop");
                track.toggle_play();
            }

            // pause current track
            (Some(FunctionKey::FN1), None, Some(NavKey::BACK), None) => {
                rprintln!("Pressed Fn1+Back, toggle play/pause");
                track.toggle_pause();
            }

            // clear pattern for current track
            (Some(FunctionKey::FN2), Some(ModifierKey::SHIFT), None, None) => {
                rprintln!("Pressed Fn2+Shift, clear pattern");
                track.clear();
            }

            // select previous track
            (None, Some(ModifierKey::SHIFT), Some(NavKey::BACK), None) => {
                rprintln!("Pressed Shift+Back, select previous track");
                *current_track = if *current_track > 0 {
                    *current_track - 1
                } else {
                    TRACKS_COUNT - 1
                };
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
                            track.randomize(step as f64 / 8.0);
                        }
                    }
                }
            }

            // handle key cv/gate
            (None, None, None, Some(CodeKey)) => {
                if let Some(code) = cx.local.keyboard.key_event.code {
                    match track.get_mode() {
                        TrackMode::CV => {
                            // handle cv recording mode, for one key pressed
                            // advance one step forward
                            if let Some(note) = cx.local.keyboard.match_note(code) {
                                rprintln!("Pressed note {:?}", note);
                                track.record_note(note);
                            }
                        }
                        TrackMode::GATE => {
                            // handle gate recording mode, toggle gate on/off
                            rprintln!("Pressed CODE {:?}", code);
                            if let Some(step) = cx.local.keyboard.match_step(code) {
                                rprintln!(
                                    "Got Step {}, track length {:?}",
                                    step,
                                    track.get_track_length()
                                );
                                if step < track.get_track_length() {
                                    rprintln!("Pressed Step {}", step);
                                    track.toggle_step(step);
                                }
                            }
                        }
                    }
                }
            }

            (_, _, _, _) => {
                delay = KEYBOARD_REFRESH_MS;
            }
        }
        app::keyboard_ctrl::spawn_after(systick_monotonic::ExtU64::millis(delay)).unwrap();
    });
}

// led_ctrl handle led display
pub(crate) fn led_ctrl(cx: app::led_ctrl::Context) {
    (
        cx.shared.led_driver,
        cx.shared.tracks,
        cx.shared.current_track,
    )
        .lock(|led_driver, tracks, current_track| {
            let track = &mut tracks[*current_track];
            match track.get_mode() {
                TrackMode::CV => cv_recording(led_driver, track, *current_track),
                TrackMode::GATE => gate_recording(led_driver, track, *current_track),
            }
        });

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

// cv_recording define led lighting when note recording mode is on
fn cv_recording(led_driver: &mut LedDriver, track: &mut Track, current_track: usize) {
    led_driver.clear();

    // button state
    if track.is_playing() {
        led_driver.set_note(track.get_current_note());
    } else {
        for step in 0..track.get_cursor() {
            led_driver.set_active_note(step, track.get_note(step));
        }
        led_driver.set_recording_cursor(track.get_cursor());
    }

    led_driver
        .set_active_track(current_track)
        .set_track_mode(track.get_mode())
        .set_clock(track.get_cursor())
        .write();
}

// gate_recording define led lighting when step recording mode is on
fn gate_recording(led_driver: &mut LedDriver, track: &mut Track, current_track: usize) {
    led_driver.clear();

    // display current active gate
    for step in 0..track.get_track_length() {
        if track.get_gate(step) == Gate::ON {
            led_driver.set_gate_on(step);
        }
    }

    led_driver
        .set_active_track(current_track)
        .set_track_mode(track.get_mode())
        .set_clock(track.get_cursor())
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
            dac1.send(
                spi_dac,
                cmd.value(match tracks[0].get_mode() {
                    TrackMode::GATE => {
                        if tracks[0].get_current_gate() == Gate::ON {
                            4080
                        } else {
                            0
                        }
                    }
                    TrackMode::CV => match_note_to_cv(tracks[0].get_current_note()),
                }),
            )
            .unwrap();

            dac2.send(
                spi_dac,
                cmd.value(match tracks[1].get_mode() {
                    TrackMode::GATE => {
                        if tracks[1].get_current_gate() == Gate::ON {
                            4080
                        } else {
                            0
                        }
                    }
                    TrackMode::CV => match_note_to_cv(tracks[1].get_current_note()),
                }),
            )
            .unwrap();
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
            if tracks[0].get_mode() == TrackMode::GATE {
                dac1.send(spi_dac, cmd.value(0)).unwrap();
            }

            if tracks[1].get_mode() == TrackMode::GATE {
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
