//! This module contains all implementations of adversaries, which determine where Packets are
//! injected into the network.

use crate::packet::Packet;
use crate::network::Network;

pub mod path_random;


/// Trait which all adversaries must implement.
pub trait Adversary {
    /// Get a new `Adversary`.
    fn new() -> Self;

    /// Create the packets to be injected.
    fn get_next_packets(&mut self, network: &Network, rd: usize) -> Vec<Packet>;
}
