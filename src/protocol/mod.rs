//! This module contains implementations of protocols, which handle how packets are forwarded and
//! how packets are added to the network.

use self::greedy::GreedyFIFO;
use self::oed::OEDWithSwap;
use crate::network::Network;
use crate::packet::Packet;
use serde::{Deserialize, Serialize};

pub mod greedy;
pub mod oed;

/// Interface for forwarding protocol behaviors.
// TODO: add check_graph_structure() to ensure that the graph we are using works with the given
// protocol.
#[derive(Serialize, Deserialize, Clone)]
pub enum Protocol {
    OEDWithSwap(OEDWithSwap),
    GreedyFIFO(GreedyFIFO),
}

impl Protocol {
    pub fn add_packet(&mut self, p: Packet, network: &mut Network) {
        match self {
            Self::GreedyFIFO(protocol) => protocol.add_packet(p, network),
            Self::OEDWithSwap(protocol) => protocol.add_packet(p, network),
        }
    }

    pub fn forward_packets(&mut self, network: &mut Network) -> Vec<Packet> {
        match self {
            Self::GreedyFIFO(protocol) => protocol.forward_packets(network),
            Self::OEDWithSwap(protocol) => protocol.forward_packets(network),
        }
    }

    pub fn new_oed_with_swap() -> Self {
        Self::OEDWithSwap(OEDWithSwap::new())
    }

    pub fn new_greedy_fifo(capacity: usize) -> Self {
        Self::GreedyFIFO(GreedyFIFO::new(capacity))
    }
}

/// Trait which all `Protocol`s must implement.
pub trait ProtocolTrait {
    /// Add a `Packet` to the network.
    fn add_packet(&mut self, p: Packet, network: &mut Network) {
        let cur = p.cur_node().unwrap();
        let next = p.next_node().unwrap();
        let eb = network.get_edgebuffer_mut(cur, next).unwrap();
        eb.buffer.push(p);
    }

    /// Forward all `Packet`s on the network. Returns absorbed `Packet`s.
    fn forward_packets(&mut self, network: &mut Network) -> Vec<Packet>;
}
