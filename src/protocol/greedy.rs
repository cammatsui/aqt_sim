//! This module contains implementations of greedy protocols.

use super::{CAPACITY_KEY, GREEDY_FIFO_NAME, GREEDY_LIS_NAME, PROTOCOL_NAME_KEY};
use crate::config::{CfgErrorMsg, Configurable};
use crate::network::{Network, NodeID};
use crate::packet::Packet;
use crate::protocol::ProtocolTrait;
use serde_json::{Map, Number, Value};
use std::cmp::min;

/// The greedy FIFO protocol always forwards packets as many packets from a buffer as allowed by
/// the protocol's capacity.
#[derive(Clone)]
pub struct GreedyFIFO {
    capacity: usize,
}

impl GreedyFIFO {
    /// Get a new `GreedyFIFO` struct.
    pub fn new(capacity: usize) -> Self {
        GreedyFIFO { capacity }
    }
}

impl ProtocolTrait for GreedyFIFO {
    fn forward_packets(&mut self, network: &mut Network) -> Vec<Packet> {
        let mut absorbed = Vec::new();
        let mut packets_to_fwd = Vec::new();

        let eb_ids = network.get_edgebuffers();
        for (from_id, to_id) in eb_ids {
            let mut buffer_packets_to_fwd = self.get_buffer_packets_to_fwd(from_id, to_id, network);
            packets_to_fwd.append(&mut buffer_packets_to_fwd);
        }

        let num_to_fwd = packets_to_fwd.len();
        for _ in 0..num_to_fwd {
            let p = packets_to_fwd.remove(0);
            if !p.should_be_absorbed() {
                self.add_packet(p, network)
            } else {
                absorbed.push(p);
            }
        }
        absorbed
    }
}

impl GreedyFIFO {
    fn get_buffer_packets_to_fwd(
        &mut self,
        from_id: NodeID,
        to_id: NodeID,
        network: &mut Network,
    ) -> Vec<Packet> {
        let eb = network.get_edgebuffer_mut(from_id, to_id).unwrap();
        let num_to_fwd = min(self.capacity, eb.buffer.len());
        let mut packets_to_fwd = Vec::new();
        for _ in 0..num_to_fwd {
            // NOTE: We need to remove from the front to enforce FIFO.
            let mut packet_to_fwd = eb.buffer.remove(0);
            packet_to_fwd.increment_path_idx();
            packets_to_fwd.push(packet_to_fwd);
        }
        packets_to_fwd
    }
}

impl Configurable for GreedyFIFO {
    fn from_config(config: Value) -> Result<Self, CfgErrorMsg> {
        let map = config.as_object().unwrap();
        let capacity = match map.get(CAPACITY_KEY) {
            Some(Value::Number(num)) => Ok(num.as_u64().unwrap() as usize),
            _ => Err(String::from("No capacity provided.")),
        }?;
        Ok(Self { capacity })
    }

    fn to_config(&self) -> Value {
        let mut map: Map<String, Value> = Map::new();
        map.insert(
            PROTOCOL_NAME_KEY.to_string(),
            Value::String(GREEDY_FIFO_NAME.to_string()),
        );
        map.insert(
            CAPACITY_KEY.to_string(),
            Value::Number(Number::from(self.capacity)),
        );
        Value::Object(map)
    }
}

/// The greedy LIS protocol always forwards packets as many of the oldest packets from a buffer as
/// allowed by the protocol's capacity.
#[derive(Clone)]
pub struct GreedyLIS {
    capacity: usize,
}

impl GreedyLIS {
    /// Get a new `GreedyLIS` struct.
    pub fn new(capacity: usize) -> Self {
        GreedyLIS { capacity }
    }
}

impl ProtocolTrait for GreedyLIS {
    fn forward_packets(&mut self, network: &mut Network) -> Vec<Packet> {
        let mut absorbed = Vec::new();
        let mut packets_to_fwd = Vec::new();

        let eb_ids = network.get_edgebuffers();
        for (from_id, to_id) in eb_ids {
            let mut buffer_packets_to_fwd = self.get_buffer_packets_to_fwd(from_id, to_id, network);
            packets_to_fwd.append(&mut buffer_packets_to_fwd);
        }

        let num_to_fwd = packets_to_fwd.len();
        for _ in 0..num_to_fwd {
            let p = packets_to_fwd.remove(0);
            if !p.should_be_absorbed() {
                self.add_packet(p, network)
            } else {
                absorbed.push(p);
            }
        }
        absorbed
    }
}

impl GreedyLIS {
    fn get_buffer_packets_to_fwd(
        &mut self,
        from_id: NodeID,
        to_id: NodeID,
        network: &mut Network,
    ) -> Vec<Packet> {
        let eb = network.get_edgebuffer_mut(from_id, to_id).unwrap();
        let num_to_fwd = min(self.capacity, eb.buffer.len());
        let mut packets_to_fwd = Vec::new();

        for _ in 0..num_to_fwd {
            let mut min_injection_rd = usize::MAX;
            let mut min_injection_idx = 0;
            for i in 0..eb.buffer.len() {
                if eb.buffer[i].get_injection_rd() < min_injection_rd {
                    min_injection_idx = i;
                    min_injection_rd = eb.buffer[i].get_injection_rd();
                }
            }
            let mut packet_to_fwd = eb.buffer.remove(min_injection_idx);
            packet_to_fwd.increment_path_idx();
            packets_to_fwd.push(packet_to_fwd);
        }

        packets_to_fwd
    }
}

impl Configurable for GreedyLIS {
    fn from_config(config: Value) -> Result<Self, CfgErrorMsg> {
        let map = config.as_object().unwrap();
        let capacity = match map.get(CAPACITY_KEY) {
            Some(Value::Number(num)) => Ok(num.as_u64().unwrap() as usize),
            _ => Err(String::from("No capacity provided.")),
        }?;
        Ok(Self { capacity })
    }

    fn to_config(&self) -> Value {
        let mut map: Map<String, Value> = Map::new();
        map.insert(
            PROTOCOL_NAME_KEY.to_string(),
            Value::String(GREEDY_LIS_NAME.to_string()),
        );
        map.insert(
            CAPACITY_KEY.to_string(),
            Value::Number(Number::from(self.capacity)),
        );
        Value::Object(map)
    }
}
