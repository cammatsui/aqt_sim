//! This module contains implementations of protocols, which handle how packets are forwarded and
//! how packets are added to the network.

use serde::{ Serialize, Deserialize };
use crate::packet::Packet;
use crate::network::Network;
use self::oed::OEDWithSwap;
use self::greedy::GreedyFIFO;

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

}


/// Trait which all `Protocol`s must implement.
pub trait ProtocolTrait {
    /// Add a `Packet` to the network.
    fn add_packet(&mut self, p: Packet, network: &mut Network) {
        let eb = network.get_edgebuffer_mut(
            p.cur_node().unwrap(),
            p.next_node().unwrap()
        ).unwrap();
        // NOTE: We need to push to the back as some of the protocols depend on this.
        eb.buffer.push(p);
    }

    /// Forward all `Packet`s on the network. Returns absorbed `Packet`s.
    fn forward_packets(&mut self, network: &mut Network) -> Vec<Packet>;
}
