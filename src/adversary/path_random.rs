//! This module contains stochastic adversaries which work on a path network.

use super::{AdversaryTrait, ADVERSARY_NAME_KEY, SD_PATH_RANDOM_NAME, SD_PATH_RANDOM_BURSTY_NAME};
use crate::config::{CfgErrorMsg, Configurable};
use crate::network::{Network, NodeID};
use crate::packet::{Packet, PacketFactory};
use crate::simulation::random::SimRng;
use serde_json::{Map, Number, Value};

/// A single-destination path random adversary, which injects one packet per round into a random
/// buffer on the path. Here, rho=1 and sigma=0.
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

/// A single-destination path "bursty" random adversary, with base rate rho=1, and sigma set by
/// the constructor.
pub struct SDPathRandomBurstyAdversary {
    factory: PacketFactory,
    rng: SimRng,
    seed: Option<u64>,
    sigma: usize,
    xi: usize,
}

impl SDPathRandomBurstyAdversary {
    /// Get a new `SDPathRandomBurstyAdversary`
    pub fn new(sigma: usize) -> Self {
        SDPathRandomBurstyAdversary {
            factory: PacketFactory::new(),
            rng: SimRng::new(),
            seed: None,
            sigma,
            xi: 0,
        }
    }

    /// Get a new `SDPathRandomAdversary` from the given seed.
    pub fn from_seed(seed: u64, sigma: usize) -> Self {
        SDPathRandomBurstyAdversary {
            factory: PacketFactory::new(),
            rng: SimRng::from_seed(seed),
            seed: Some(seed),
            sigma,
            xi: 0,
        }
    }
}

impl AdversaryTrait for SDPathRandomBurstyAdversary {
    fn get_next_packets(&mut self, network: &Network, rd: usize) -> Vec<Packet> {
        // Possible numbers of packets to inject are 0..(sigma-xi+1). Choose uniformly from these
        // options.
        let num_to_inject = self.rng.rand_int(self.sigma - self.xi + 2);
        let mut next_packets = Vec::new();
        for _ in 0..num_to_inject {
            let dest_id: NodeID = network.get_num_nodes() - 1;
            let src_id = self.rng.rand_int(dest_id - 1);
            next_packets.push(self.factory.create_packet((0..dest_id + 1).collect(), rd, src_id));
        }
        // Update xi.
        if self.xi + num_to_inject == 0 {
            self.xi = 0
        } else {
            self.xi = self.xi + num_to_inject - 1;
        }
        next_packets
    }
}

const SIGMA_NAME_KEY: &str = "sigma";

impl Configurable for SDPathRandomBurstyAdversary {
    fn from_config(config: Value) -> Result<Self, CfgErrorMsg> {
        let map = config.as_object().unwrap();
        let seed = match map.get(SEED_NAME_KEY) {
            Some(Value::Number(seed)) => Some(seed.as_u64().unwrap()),
            _ => None,
        };

        let sigma = match map.get(SIGMA_NAME_KEY) {
            Some(Value::Number(num)) => Ok(num.as_u64().unwrap() as usize),
            _ => Err(String::from("No sigma value provided.")),
        }?;

        match seed {
            Some(seed) => Ok(Self::from_seed(seed, sigma)),
            None => Ok(Self::new(sigma)),
        }
    }

    fn to_config(&self) -> Value {
        let mut map = Map::new();
        map.insert(
            ADVERSARY_NAME_KEY.to_string(),
            Value::String(SD_PATH_RANDOM_BURSTY_NAME.to_string()),
        );
        map.insert(
            SIGMA_NAME_KEY.to_string(),
            Value::Number(Number::from(self.sigma)),
        );
        if let Some(seed) = self.seed {
            map.insert(SEED_NAME_KEY.to_string(), Value::Number(Number::from(seed)));
        }
        Value::Object(map)
    }
}
