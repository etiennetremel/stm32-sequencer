use rand::RngCore;
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};
use rtt_target::rprintln;

use crate::constants::*;

#[derive(Copy, Clone, Debug)]
pub struct Track {
    pub cursor: usize,
    pub divide: u32,
    pub pattern: [u8; 8],
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

impl Track {
    pub fn new()->Track {
        return Track{
            cursor: 0,
            divide: 0,
            seed: 0,
            playing: true,
            mode: Mode::Gate,
            pattern: [0; STEPS_COUNT],
        }
    }

    pub fn toggle_step(&mut self, index: usize) {
        self.pattern[index] = !self.pattern[index];
    }

    pub fn tick(&mut self) {
        if !self.playing {
            return;
        }
        self.cursor += 1;
        if self.cursor >= self.pattern.len() {
            self.reset();
        }
        rprintln!(
            "{} [{:?}:{}]",
            self.cursor,
            self.pattern[self.cursor as usize],
            self.pattern.len()
        );
    }

    pub fn randomize(&mut self, probability: f64) {
        // TODO: review seeding logic, executing seeding logic on every random
        // action is not really optimized
        let mut rng = SmallRng::seed_from_u64(self.seed);

        // set new seed with random u64
        self.seed = rng.next_u64();

        // randomly fill array with range 0 255
        let mut pattern = [0u8; 8];
        rng.fill_bytes(&mut pattern);
        rprintln!("Randomized pattern {:?}", pattern);

        if self.mode == Mode::CV {
            self.pattern = pattern.into();
            return;
        }

        // define true/false based on probability
        for i in 0..self.pattern.len() {
            if pattern[i] > (probability * 255.0) as u8 {
                self.pattern[i] = 255;
            } else {
                self.pattern[i] = 0;
            }
        }
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
