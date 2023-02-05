use rand::distributions::{Distribution, Standard};
use rand::rngs::SmallRng;
use rand::RngCore;
use rand::{Rng, SeedableRng};
use rtt_target::rprintln;

use crate::constants::*;

// Track can either be Gate or CV out
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum TrackMode {
    GATE,
    CV,
}

// Gate state
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Gate {
    ON,
    OFF,
}

// Note name
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Note {
    A,
    Ab,
    B,
    Bb,
    C,
    D,
    Db,
    E,
    Eb,
    F,
    G,
    Gb,
}

// Step can be a note or gate
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Step {
    pub gate: Gate,
    pub note: Note,
    pub octave: i8,
    pub velocity: u8,
}

#[derive(Copy, Clone, Debug)]
pub struct Track {
    cursor: usize,
    divide: u8,
    length: usize,
    pattern: [Step; STEPS_COUNT],
    play: bool,
    mode: TrackMode,
    seed: u64,
}

impl Default for Track {
    fn default() -> Self {
        Self::new()
    }
}

impl Track {
    pub fn new() -> Track {
        Track {
            cursor: 0,
            divide: 0,
            length: STEPS_COUNT,
            seed: 0,
            play: true,
            mode: TrackMode::GATE,
            pattern: [Step {
                gate: Gate::OFF,
                velocity: 255,
                octave: 0,
                note: Note::C,
            }; STEPS_COUNT],
        }
    }

    pub fn get_track_length(&mut self) -> usize {
        self.length
    }

    pub fn get_gate(&mut self, index: usize) -> Gate {
        self.pattern[index].gate
    }

    pub fn get_current_gate(&mut self) -> Gate {
        self.pattern[self.cursor].gate
    }

    pub fn get_note(&mut self, index: usize) -> Note {
        self.pattern[index].note
    }

    pub fn get_current_note(&mut self) -> Note {
        self.pattern[self.cursor].note
    }

    pub fn get_cursor(&mut self) -> usize {
        self.cursor
    }

    pub fn get_mode(&mut self) -> TrackMode {
        self.mode
    }

    pub fn set_mode(&mut self, mode: TrackMode) -> &mut Self {
        if mode == TrackMode::CV {
            self.stop();
        }
        self.mode = mode;
        self
    }

    pub fn toggle_mode(&mut self) -> &mut Self {
        if self.mode == TrackMode::GATE {
            self.set_mode(TrackMode::CV);
        } else {
            self.set_mode(TrackMode::GATE);
        }
        self
    }

    pub fn record_note(&mut self, note: Note) -> &mut Self {
        self.pattern[self.cursor].note = note;
        if self.cursor < self.get_track_length() - 1 {
            self.cursor += 1;
        } else {
            self.cursor = 0;
            self.play();
        }
        self
    }

    pub fn set_gate(&mut self, index: usize, state: Gate) -> &mut Self {
        self.pattern[index].gate = state;
        self
    }

    pub fn toggle_step(&mut self, index: usize) -> &mut Self {
        if self.pattern[index].gate == Gate::ON {
            self.pattern[index].gate = Gate::OFF;
        } else {
            self.pattern[index].gate = Gate::ON;
        }
        self
    }

    pub fn set_note(&mut self, index: usize, note: Note) -> &mut Self {
        self.pattern[index].note = note;
        self
    }

    pub fn tick(&mut self) -> &mut Self {
        if !self.play {
            return self;
        }
        if self.cursor < self.get_track_length() - 1 {
            self.cursor += 1;
        } else {
            self.reset();
        }
        self
    }

    pub fn randomize(&mut self, _probability: f64) -> &mut Self {
        // TODO: review seeding logic, executing seeding logic on every random
        // action is not really optimized
        let mut rng = SmallRng::seed_from_u64(self.seed);

        // set new seed with random u64
        self.seed = rng.next_u64();

        // TODO: check how to implement probability
        for i in 0..self.get_track_length() {
            self.pattern[i].note = rng.gen();
            self.pattern[i].gate = rng.gen();
        }
        rprintln!("Randomized pattern {:?}", self.pattern);
        self
    }

    pub fn clear(&mut self) -> &mut Self {
        for i in 0..self.get_track_length() {
            self.pattern[i].gate = Gate::OFF;
            self.pattern[i].note = Note::C;
        }
        self
    }

    pub fn reset(&mut self) -> &mut Self {
        self.cursor = 0;
        self
    }

    pub fn play(&mut self) -> &mut Self {
        self.play = true;
        self
    }

    pub fn toggle_play(&mut self) -> &mut Self {
        if self.is_playing() {
            self.stop();
        } else {
            self.play();
        }
        self
    }

    pub fn stop(&mut self) -> &mut Self {
        self.play = false;
        self.reset();
        self
    }

    pub fn toggle_pause(&mut self) -> &mut Self {
        if self.is_playing() {
            self.play = false;
        } else {
            self.play = true;
        }
        self
    }

    pub fn is_playing(&mut self) -> bool {
        self.play
    }
}

impl Distribution<Gate> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Gate {
        if rng.gen_bool(0.5) {
            Gate::ON
        } else {
            Gate::OFF
        }
    }
}

impl Distribution<Note> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Note {
        match rng.gen_range(0..=11) {
            0 => Note::A,
            1 => Note::Ab,
            2 => Note::B,
            3 => Note::Bb,
            4 => Note::C,
            5 => Note::D,
            6 => Note::Db,
            7 => Note::E,
            8 => Note::Eb,
            9 => Note::F,
            10 => Note::G,
            11 => Note::Gb,
            _ => todo!(),
        }
    }
}
