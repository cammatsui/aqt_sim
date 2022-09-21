//! This module contains implementations of protocols, which handle how packets are forwarded and
//! how packets are added to the network.

use self::greedy::{GreedyFIFO, GreedyLIS};
use self::oed::OEDWithSwap;
use crate::config::{CfgErrorMsg, Configurable};
use crate::network::Network;
use crate::packet::Packet;
use serde_json::{Map, Value};

pub mod greedy;
pub mod oed;

/// Interface for forwarding protocol behaviors.
// TODO: add check_graph_structure() to ensure that the graph we are using works with the given
// protocol.
#[derive(Clone)]
pub enum Protocol {
    OEDWithSwap(OEDWithSwap),
    GreedyFIFO(GreedyFIFO),
    GreedyLIS(GreedyLIS),
}

impl Protocol {
    /// Get a new `OEDWithSwap` protocol.
    pub fn new_oed_with_swap() -> Self {
        Self::OEDWithSwap(OEDWithSwap::new())
    }

    /// Get a new `GreedyFIFO` protocol.
    pub fn new_greedy_fifo(capacity: usize) -> Self {
        Self::GreedyFIFO(GreedyFIFO::new(capacity))
    }

    /// Add a packet to the given `Network` via `ProtocolTrait`.
    pub fn add_packet(&mut self, p: Packet, network: &mut Network) {
        match self {
            Self::GreedyFIFO(protocol) => protocol.add_packet(p, network),
            Self::OEDWithSwap(protocol) => protocol.add_packet(p, network),
            Self::GreedyLIS(protocol) => protocol.add_packet(p, network),
        }
    }

    /// Forward packets on the given `Network` via `ProtocolTrait`.
    pub fn forward_packets(&mut self, network: &mut Network) -> Vec<Packet> {
        match self {
            Self::OEDWithSwap(protocol) => protocol.forward_packets(network),
            Self::GreedyFIFO(protocol) => protocol.forward_packets(network),
            Self::GreedyLIS(protocol) => protocol.forward_packets(network),
        }
    }
}

const PROTOCOL_NAME_KEY: &str = "protocol_name";
const OED_WITH_SWAP_NAME: &str = "oed_swap";
const GREEDY_FIFO_NAME: &str = "greedy_fifo";
const GREEDY_LIS_NAME: &str = "greedy_lis";
const CAPACITY_KEY: &str = "capacity";

impl Configurable for Protocol {
    fn from_config(config: Value) -> Result<Self, CfgErrorMsg> {
        // TODO: correct error msg
        let map: Map<String, Value> = config.as_object().unwrap().clone();
        let protocol_name = match map.get(PROTOCOL_NAME_KEY) {
            Some(Value::String(name)) => Ok(name),
            _ => Err(String::from("No protocol name found.")),
        }?;

        match &protocol_name[..] {
            OED_WITH_SWAP_NAME => Ok(Self::OEDWithSwap(OEDWithSwap::from_config(config).unwrap())),
            GREEDY_FIFO_NAME => Ok(Self::GreedyFIFO(GreedyFIFO::from_config(config).unwrap())),
            GREEDY_LIS_NAME => Ok(Self::GreedyLIS(GreedyLIS::from_config(config).unwrap())),
            _ => Err(format!("No protocol with name {}.", protocol_name)),
        }
    }

    fn to_config(&self) -> Value {
        match self {
            Self::OEDWithSwap(p) => p.to_config(),
            Self::GreedyLIS(p) => p.to_config(),
            Self::GreedyFIFO(p) => p.to_config(),
        }
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
