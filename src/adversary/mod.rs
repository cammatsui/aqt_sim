//! This module contains all implementations of adversaries, which determine where Packets are
//! injected into the network.

use self::path_random::{SDPathRandomAdversary, SDPathRandomBurstyAdversary};
use crate::config::{CfgErrorMsg, Configurable};
use crate::network::Network;
use crate::packet::Packet;
use serde_json::Value;

pub mod path_random;

/// Enum to store all adversaries.
pub enum Adversary {
    SDPathRandom(SDPathRandomAdversary),
    SDPathRandomBursty(SDPathRandomBurstyAdversary),
}

impl Adversary {
    /// Get the next packets, through `AdversaryTrait`
    pub fn get_next_packets(&mut self, network: &Network, rd: usize) -> Vec<Packet> {
        match self {
            Self::SDPathRandom(a) => a.get_next_packets(network, rd),
            Self::SDPathRandomBursty(a) => a.get_next_packets(network, rd),
        }
    }
}

const ADVERSARY_NAME_KEY: &str = "adversary_name";
const SD_PATH_RANDOM_NAME: &str = "sd_path_random";
const SD_PATH_RANDOM_BURSTY_NAME: &str = "sd_path_random_bursty";

impl Configurable for Adversary {
    fn from_config(config: Value) -> Result<Self, CfgErrorMsg> {
        let name = match config.get(ADVERSARY_NAME_KEY) {
            Some(Value::String(name)) => Ok(name),
            _ => Err("No adversary name found."),
        }?;

        match &name[..] {
            SD_PATH_RANDOM_NAME => Ok(Adversary::SDPathRandom(
                SDPathRandomAdversary::from_config(config.clone()).unwrap(),
            )),
            SD_PATH_RANDOM_BURSTY_NAME => Ok(Adversary::SDPathRandomBursty(
                SDPathRandomBurstyAdversary::from_config(config.clone()).unwrap(),
            )),
            _ => Err(format!("No adversary with name {}", name)),
        }
    }

    fn to_config(&self) -> Value {
        match self {
            Self::SDPathRandom(a) => a.to_config(),
            Self::SDPathRandomBursty(a) => a.to_config(),
        }
    }
}

/// Trait which all adversaries must implement.
pub trait AdversaryTrait {
    /// Create the packets to be injected.
    fn get_next_packets(&mut self, network: &Network, rd: usize) -> Vec<Packet>;
}
