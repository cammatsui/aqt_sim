//! This module contains all implementations of adversaries, which determine where Packets are
//! injected into the network.

use serde::{ Serialize, Deserialize };
use crate::packet::Packet;
use crate::network::Network;
use self::path_random::SDPathRandomAdversary;
use self::preset::PresetAdversary;

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
}


/// Trait which all adversaries must implement.
pub trait AdversaryTrait {
    /// Create the packets to be injected.
    fn get_next_packets(&mut self, network: &Network, rd: usize) -> Vec<Packet>;
}
