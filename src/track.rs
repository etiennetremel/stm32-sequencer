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
    pub mode: Mode,
    seed: u64,
}

// Track can either be Gate or CV out
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Mode {
    Gate,
    CV,
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
    Unknown,
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
            _ => Note::Unknown,
        }
    }
}

// Step can be a note or gate
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Step {
    pub gate: bool,
    pub note: Note,
    pub velocity: u8,
}

impl Track {
    pub fn new()->Track {
        return Track{
            cursor: 0,
            divide: 0,
            seed: 0,
            playing: true,
            mode: Mode::Gate,
            pattern: [Step{gate: false, velocity: 255, note: Note::C}; STEPS_COUNT],
        }
    }

    pub fn set_mode(&mut self, mode: Mode) {
        self.mode = mode;
    }

    pub fn toggle_mode(&mut self) {
        if self.mode == Mode::Gate {
            self.mode = Mode::CV;
        } else {
            self.mode = Mode::Gate;
        }
    }

    pub fn toggle_step(&mut self, index: usize) {
        if self.pattern[index].gate {
            self.pattern[index].gate = false;
        } else {
            self.pattern[index].gate = true;
        }
    }

    pub fn set_note(&mut self, index: usize, note: Note) {
        self.pattern[index].note = note;
    }

    pub fn tick(&mut self) {
        if !self.playing {
            return;
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
    }

    // pub fn randomise_note(&mut self) {
    //     for i in 0..self.pattern.len() {
    //         self.pattern[i].note = rng.gen();
    //     }
    //     rprintln!("Randomized note pattern {:?}", self.pattern);
    //     return;
    // }

    pub fn randomize(&mut self, probability: f64) {
        // TODO: review seeding logic, executing seeding logic on every random
        // action is not really optimized
        let mut rng = SmallRng::seed_from_u64(self.seed);

        // set new seed with random u64
        self.seed = rng.next_u64();
        if self.mode == Mode::CV {
            for i in 0..self.pattern.len() {
                self.pattern[i].note = rng.gen();
            }
            rprintln!("Randomized note pattern {:?}", self.pattern);
            return;
        }

        for i in 0..self.pattern.len() {
            self.pattern[i].gate = rng.gen();
        }
        rprintln!("Randomized gate pattern {:?}", self.pattern);

        // Generate random notes when CV mode
        // if self.mode == Mode::CV {
        //     for i in 0..self.pattern.len() {
        //         self.pattern[i].note = rng.gen();
        //     }
        //     rprintln!("Randomized note pattern {:?}", self.pattern);
        //     return;
        // }
        //
        // // Generate random gates when CV gate
        // // randomly fill array with range 0 255
        // let mut pattern = [0u8; 8];
        // rng.fill_bytes(&mut pattern);
        // rprintln!("Randomized pattern {:?}", pattern);
        //
        // // define true/false based on probability
        // for i in 0..self.pattern.len() {
        //     if pattern[i] > (probability * 255.0) as u8 {
        //         self.pattern[i].gate = 255;
        //     } else {
        //         self.pattern[i].gate = 0;
        //     }
        // }
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
