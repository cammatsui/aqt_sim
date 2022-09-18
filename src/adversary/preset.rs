//! This module contains the preset adversary, where you must specify the packets to be injected.

use crate::packet::{ Packet, PacketFactory, PacketPath };
use crate::network::Network;
use super::Adversary;


/// An adversary which gives a preset vec of packets per round.
pub struct PresetAdversary {
    rds: usize,
    to_inject: Vec<Vec<Packet>>,
}

impl PresetAdversary {
    /// Create a new `PresetAdversary` from a vector of injection configs (specifying packets).
    pub fn from_injection_configs(mut injections: Vec<Vec<InjectionConfig>>) -> Self {
        let mut factory = PacketFactory::new();
        let mut to_inject = Vec::new();
        let rds = injections.len();

        for rd in 0..rds {
            let mut rd_packets = Vec::new();
            let rd_injections = &mut injections[rd];
            let num_injections = rd_injections.len();
            for _ in 0..num_injections {
                let cfg = rd_injections.remove(0);
                rd_packets.push(factory.create_packet(cfg.path, rd, cfg.path_idx));
            }
            to_inject.push(rd_packets);
        }
        PresetAdversary { to_inject, rds }
    }

    /// Get the pre-defined number of rounds for this adversary.
    pub fn rds(&self) -> usize {
        self.rds
    }
}

impl Adversary for PresetAdversary {
    fn get_next_packets(&mut self, _network: &Network, _rd: usize) -> Vec<Packet> {
        self.to_inject.remove(0)
    }
}


/// Config to create a packet from.
pub struct InjectionConfig {
    path: PacketPath,
    path_idx: usize,
}

impl InjectionConfig {
    pub fn new(path: PacketPath, path_idx: usize) -> Self {
        InjectionConfig { path, path_idx }
    }
}
