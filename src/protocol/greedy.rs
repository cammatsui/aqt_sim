//! This module contains implementations of greedy protocols.

use serde::Serialize;
use crate::network::{ NodeID, Network };
use crate::packet::Packet;
use crate::protocol::Protocol;
use std::cmp::min;


/// The greedy FIFO protocol always forwards packets as many packets from a buffer as allowed by
/// the protocol's capacity.
#[derive(Serialize)]
pub struct GreedyFIFO {
    capacity: usize,
    protocol_name: String,
}

impl Protocol for GreedyFIFO {
    fn new(capacity: usize) -> Self {
        GreedyFIFO { capacity, protocol_name: String::from("GreedyFIFO") }
    }

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

    fn get_capacity(&self) -> usize {
        self.capacity
    }
}

impl GreedyFIFO {
    fn get_buffer_packets_to_fwd(
        &mut self,
        from_id: NodeID,
        to_id: NodeID,
        network: &mut Network
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
