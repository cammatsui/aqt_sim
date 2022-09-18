//! This module contains stochastic adversaries which work on a path network.

use rand::Rng;
use crate::packet::{ Packet, PacketFactory };
use crate::network::{ Network, NodeID };
use super::Adversary;


/// A single-destination path random adversary, which injects one packet per round into a random 
/// buffer on the path.
pub struct SDPathRandomAdversary {
    factory: PacketFactory,
}

impl SDPathRandomAdversary {
    pub fn new() -> Self {
        SDPathRandomAdversary {
            factory: PacketFactory::new(),
        }
    }
}

impl Adversary for SDPathRandomAdversary {
    fn get_next_packets(&mut self, network: &Network, rd: usize) -> Vec<Packet> {
        let dest_id: NodeID = network.get_num_nodes()-1;
        let mut rng = rand::thread_rng();
        let src_id: NodeID = rng.gen_range(0..dest_id-1);
        let p = self.factory.create_packet((0..dest_id+1).collect(), rd, src_id);
        vec![p]
    }
}
