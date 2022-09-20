//! This module contains stochastic adversaries which work on a path network.

use super::AdversaryTrait;
use crate::network::{Network, NodeID};
use crate::packet::{Packet, PacketFactory};
use rand::{Rng, SeedableRng};
use rand_pcg::Pcg32;
use serde::{Deserialize, Serialize};

/// A single-destination path random adversary, which injects one packet per round into a random
/// buffer on the path.
#[derive(Serialize, Deserialize, Clone)]
pub struct SDPathRandomAdversary {
    #[serde(skip)]
    factory: PacketFactory,
    seed: Option<u64>,
}

impl SDPathRandomAdversary {
    /// Get a new `SDPathRandomAdversary`
    pub fn new() -> Self {
        SDPathRandomAdversary {
            factory: PacketFactory::new(),
            seed: None,
        }
    }

    /// Get a new `SDPathRandomAdversary` with the given seed.
    pub fn from_seed(seed: u64) -> Self {
        SDPathRandomAdversary {
            factory: PacketFactory::new(),
            seed: Some(seed),
        }
    }
}

impl AdversaryTrait for SDPathRandomAdversary {
    fn get_next_packets(&mut self, network: &Network, rd: usize) -> Vec<Packet> {
        let dest_id: NodeID = network.get_num_nodes() - 1;
        let src_id = match self.seed {
            Some(seed) => {
                let mut rng = Pcg32::seed_from_u64(seed);
                rng.gen_range(0..dest_id - 1)
            }
            None => {
                let mut rng = rand::thread_rng();
                rng.gen_range(0..dest_id - 1)
            }
        };

        let p = self
            .factory
            .create_packet((0..dest_id + 1).collect(), rd, src_id);
        vec![p]
    }
}
