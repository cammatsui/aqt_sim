//! This module contains stochastic adversaries which work on a path network.

use super::{AdversaryTrait, ADVERSARY_NAME_KEY, SD_PATH_RANDOM_NAME};
use crate::config::{CfgErrorMsg, Configurable};
use crate::network::{Network, NodeID};
use crate::packet::{Packet, PacketFactory};
use crate::simulation::random::SimRng;
use serde_json::{Map, Number, Value};

/// A single-destination path random adversary, which injects one packet per round into a random
/// buffer on the path.
pub struct SDPathRandomAdversary {
    factory: PacketFactory,
    rng: SimRng,
    seed: Option<u64>,
}

impl SDPathRandomAdversary {
    /// Get a new `SDPathRandomAdversary`
    pub fn new() -> Self {
        SDPathRandomAdversary {
            factory: PacketFactory::new(),
            rng: SimRng::new(),
            seed: None,
        }
    }

    /// Get a new `SDPathRandomAdversary` from the given seed.
    pub fn from_seed(seed: u64) -> Self {
        SDPathRandomAdversary {
            factory: PacketFactory::new(),
            rng: SimRng::from_seed(seed),
            seed: Some(seed),
        }
    }
}

impl AdversaryTrait for SDPathRandomAdversary {
    fn get_next_packets(&mut self, network: &Network, rd: usize) -> Vec<Packet> {
        let dest_id: NodeID = network.get_num_nodes() - 1;
        let src_id = self.rng.rand_int(dest_id - 1);

        vec![self
            .factory
            .create_packet((0..dest_id + 1).collect(), rd, src_id)]
    }
}

const SEED_NAME_KEY: &str = "seed";

impl Configurable for SDPathRandomAdversary {
    fn from_config(config: Value) -> Result<Self, CfgErrorMsg> {
        let map = config.as_object().unwrap();
        match map.get(SEED_NAME_KEY) {
            Some(Value::Number(num)) => {
                let seed = num.as_u64().unwrap();
                Ok(Self::from_seed(seed))
            }
            _ => Ok(Self::new()),
        }
    }

    fn to_config(&self) -> Value {
        let mut map = Map::new();
        map.insert(
            ADVERSARY_NAME_KEY.to_string(),
            Value::String(SD_PATH_RANDOM_NAME.to_string()),
        );
        if let Some(seed) = self.seed {
            map.insert(SEED_NAME_KEY.to_string(), Value::Number(Number::from(seed)));
        }
        Value::Object(map)
    }
}
