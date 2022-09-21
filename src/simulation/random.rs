use rand::{rngs::ThreadRng, Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

pub struct SimRng {
    seeded_rng: Option<ChaCha8Rng>,
    unseeded_rng: Option<ThreadRng>,
}

impl SimRng {
    pub fn new() -> Self {
        SimRng {
            seeded_rng: None,
            unseeded_rng: Some(rand::thread_rng()),
        }
    }

    pub fn from_seed(seed: u64) -> Self {
        SimRng {
            seeded_rng: Some(ChaCha8Rng::seed_from_u64(seed)),
            unseeded_rng: None,
        }
    }

    pub fn rand_int(&mut self, max: usize) -> usize {
        if let Some(rng) = &mut self.seeded_rng {
            return rng.gen_range(0..max);
        }
        if let Some(rng) = &mut self.unseeded_rng {
            return rng.gen_range(0..max);
        }
        panic!("No rng for this config");
    }
}

impl Default for SimRng {
    fn default() -> Self {
        Self::new()
    }
}
