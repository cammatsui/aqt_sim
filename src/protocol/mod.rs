//! This module contains implementations of protocols, which handle how packets are forwarded and
//! how packets are added to the network.

use crate::packet::Packet;
use crate::network::Network;


pub mod greedy;
pub mod oed;


/// Interface for forwarding protocol behaviors.
// TODO: add check_graph_structure() to ensure that the graph we are using works with the given
// protocol.
pub trait Protocol {
    /// Create a new instance of this `Protocol`.
    fn new(capacity: usize) -> Self;

    /// Add a `Packet` to the network.
    fn add_packet(&mut self, p: Packet, network: &mut Network) {
        let eb = network.get_edgebuffer_mut(
            p.cur_node().unwrap(),
            p.next_node().unwrap()
        ).unwrap();
        // NOTE: We need to push to the back as some of the protocols depend on this.
        eb.buffer.push(p);
    }

    /// Forward all `Packet`s on the network.
    fn forward_packets(&mut self, network: &mut Network);

    /// Get the edge capacity for this protocol.
    fn get_capacity(&self) -> usize;
}
