use rand::RngCore;
use rand::distributions::{Distribution, Standard};
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};
use rtt_target::rprintln;

use crate::constants::*;

#[derive(Copy, Clone, Debug)]
pub struct Track {
    pub cursor: usize,
    pub divide: u8,
    pub pattern: [Step; STEPS_COUNT],
    pub playing: bool,
    pub mode: TrackMode,
    seed: u64,
}

// Track can either be Gate or CV out
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum TrackMode {
    GATE,
    CV,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Gate {
    ON,
    OFF,
}

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

impl Track {
    pub fn new()->Track {
        return Track{
            cursor: 0,
            divide: 0,
            seed: 0,
            playing: true,
            mode: TrackMode::GATE,
            pattern: [Step{gate: Gate::OFF, velocity: 255, octave: 0, note: Note::C}; STEPS_COUNT],
        }
    }

    pub fn set_mode(&mut self, mode: TrackMode) -> &mut Self {
        self.mode = mode;
        return self;
    }

    pub fn toggle_mode(&mut self) -> &mut Self {
        if self.mode == TrackMode::GATE {
            self.mode = TrackMode::CV;
        } else {
            self.mode = TrackMode::GATE;
        }
        return self;
    }

    pub fn set_gate(&mut self, index: usize, state: Gate) -> &mut Self {
        self.pattern[index].gate = state;
        return self;
    }

    pub fn toggle_step(&mut self, index: usize) -> &mut Self {
        if self.pattern[index].gate == Gate::ON {
            self.pattern[index].gate = Gate::OFF;
        } else {
            self.pattern[index].gate = Gate::ON;
        }
        return self;
    }

    pub fn set_note(&mut self, index: usize, note: Note) -> &mut Self {
        self.pattern[index].note = note;
        return self;
    }

    pub fn tick(&mut self) -> &mut Self {
        if !self.playing {
            return self;
        }
        self.cursor += 1;
        if self.cursor >= self.pattern.len() {
            self.reset();
        }
        // rprintln!(
        //     "{} [{:?}:{}]",
        //     self.cursor,
        //     self.pattern[self.cursor as usize],
        //     self.pattern.len()
        // );
        return self;
    }

    // pub fn randomise_note(&mut self) {
    //     for i in 0..self.pattern.len() {
    //         self.pattern[i].note = rng.gen();
    //     }
    //     rprintln!("Randomized note pattern {:?}", self.pattern);
    //     return;
    // }

    pub fn randomize(&mut self, probability: f64) -> &mut Self {
        // TODO: review seeding logic, executing seeding logic on every random
        // action is not really optimized
        let mut rng = SmallRng::seed_from_u64(self.seed);

        // set new seed with random u64
        self.seed = rng.next_u64();

        // TODO: check how to implement probability
        for i in 0..self.pattern.len() {
            self.pattern[i].note = rng.gen();
            self.pattern[i].gate = rng.gen();
        }
        rprintln!("Randomized pattern {:?}", self.pattern);
        return self;
    }

    pub fn clear(&mut self) -> &mut Self {
        for i in 0..self.pattern.len() {
            self.pattern[i].gate = Gate::OFF;
            self.pattern[i].note = Note::C;
        }
        return self;
    }

    fn reset(&mut self) {
        self.cursor = 0;
    }

    fn play(&mut self) {
        self.playing = true;
    }

    fn stop(&mut self) {
        self.pause();
        self.reset();
    }

    fn pause(&mut self) {
        self.playing = false;
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
        match rng.gen_range(0..=10) {
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
