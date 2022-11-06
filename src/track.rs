use rand::RngCore;
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};
use rtt_target::rprintln;

use crate::constants::*;

#[derive(Copy, Clone, Debug)]
pub struct Track {
    pub cursor: usize,
    pub divide: u32,
    pub pattern: [bool; 8],
    pub playing: bool,
    seed: u64,
}

impl Track {
    pub fn new()->Track {
        return Track{
            cursor: 0,
            divide: 0,
            seed: 0,
            playing: true,
            pattern: [false; STEPS_COUNT],
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
        // TODO: review seeding logic?
        let mut rng = SmallRng::seed_from_u64(self.seed);

        // set new seed with random u64
        self.seed = rng.next_u64();

        // randomly fill array with range 0 255
        let mut arr = [0u8; 8];
        rng.fill_bytes(&mut arr);

        // define true/false based on probability
        for i in 0..self.pattern.len() {
            self.pattern[i] = arr[i] > (probability * 255.0) as u8;
        }
        rprintln!("Randomized {:?}", self.pattern);
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
