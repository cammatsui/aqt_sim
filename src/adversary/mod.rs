//! This module contains all implementations of adversaries, which determine where Packets are
//! injected into the network.

use self::preset::PresetAdversary;
use self::{path_random::SDPathRandomAdversary, preset::InjectionConfig};
use crate::network::Network;
use crate::packet::Packet;
use serde::{Deserialize, Serialize};

pub mod path_random;
pub mod preset;

/// Enum to store all adversaries.
#[derive(Serialize, Deserialize, Clone)]
pub enum Adversary {
    SDPathRandom(SDPathRandomAdversary),
    Preset(PresetAdversary),
}

impl Adversary {
    /// Get the next packets, through `AdversaryTrait`
    pub fn get_next_packets(&mut self, network: &Network, rd: usize) -> Vec<Packet> {
        match self {
            Self::SDPathRandom(a) => a.get_next_packets(network, rd),
            Self::Preset(a) => a.get_next_packets(network, rd),
        }
    }

    pub fn new_sd_path_random() -> Self {
        Self::SDPathRandom(SDPathRandomAdversary::new())
    }

    pub fn sd_path_random_from_seed(seed: u64) -> Self {
        Self::SDPathRandom(SDPathRandomAdversary::from_seed(seed))
    }

    pub fn preset_from_injection_configs(to_inject: Vec<Vec<InjectionConfig>>) -> Self {
        Self::Preset(PresetAdversary::from_injection_configs(to_inject))
    }
}

/// Trait which all adversaries must implement.
pub trait AdversaryTrait {
    /// Create the packets to be injected.
    fn get_next_packets(&mut self, network: &Network, rd: usize) -> Vec<Packet>;
}
