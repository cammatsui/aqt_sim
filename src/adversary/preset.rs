//! This module contains the preset adversary, where you must specify the packets to be injected.

use super::AdversaryTrait;
use crate::network::Network;
use crate::packet::{Packet, PacketFactory, PacketPath};
use serde::{Deserialize, Serialize};

/// An adversary which gives a preset vec of packets per round.
#[derive(Serialize, Deserialize, Clone)]
pub struct PresetAdversary {
    #[serde(skip)]
    factory: PacketFactory,
    rds: usize,
    to_inject: Vec<Vec<InjectionConfig>>,
}

impl PresetAdversary {
    /// Create a new `PresetAdversary` from a vector of injection configs (specifying packets).
    pub fn from_injection_configs(to_inject: Vec<Vec<InjectionConfig>>) -> Self {
        let rds = to_inject.len();
        PresetAdversary {
            to_inject,
            rds,
            factory: PacketFactory::new(),
        }
    }

    /// Get the pre-defined number of rounds for this adversary.
    pub fn rds(&self) -> usize {
        self.rds
    }
}

impl AdversaryTrait for PresetAdversary {
    fn get_next_packets(&mut self, _network: &Network, rd: usize) -> Vec<Packet> {
        let mut next_packets = Vec::new();
        let mut next_injections = self.to_inject.remove(0);
        let num_injections = next_injections.len();
        for _ in 0..num_injections {
            let next_injection = next_injections.remove(0);
            next_packets.push(self.factory.create_packet(
                next_injection.path,
                rd,
                next_injection.path_idx,
            ));
        }
        next_packets
    }
}

/// Config to create a packet from.
#[derive(Serialize, Deserialize, Clone)]
pub struct InjectionConfig {
    path: PacketPath,
    path_idx: usize,
}

impl InjectionConfig {
    /// Make a new `InjectionConfig`.
    pub fn new(path: PacketPath, path_idx: usize) -> Self {
        InjectionConfig { path, path_idx }
    }
}
