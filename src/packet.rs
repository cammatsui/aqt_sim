//! This module contains structs and types related to packets in the AQT model, including the
//! Packet struct itself as well as a PacketFactory and PacketPath (which is just a vector of
//! NodeIDs determining the route the Packet should follow).

use crate::network::NodeID;
use std::fmt;

/// The `Packet` struct represents a packet in AQT. It includes:
/// - An id, which is unique,
/// - A `PacketPath` for the packet to follow in the network,
/// - An index into the packet's path so we know where the packet currently is, and
/// - The packet's injection round.
///
/// We enforce the ID uniqueness by *only* allowing packets to be created via the `PacketFactory`
/// struct.
#[derive(Clone)]
pub struct Packet {
    id: usize,
    path: PacketPath,
    path_idx: usize,
    injection_rd: usize,
}

impl Packet {
    /// Get this `Packet`'s id.
    pub fn get_id(&self) -> usize {
        self.id
    }

    /// Increment the index into the `PacketPath`. We need to keep this in sync with the packet's
    /// state in the `Network` so we can quickly forward packets.
    pub fn increment_path_idx(&mut self) {
        if self.is_absorbed() {
            panic!("Packet has already been absorbed.");
        }
        self.path_idx += 1;
    }

    /// Decrement the index into the `PacketPath`. We need to keep this in sync with the packet's
    /// state in the `Network` so we can quickly forward packets.
    pub fn decrement_path_idx(&mut self) {
        if self.path_idx == 0 {
            panic!("Packet is already at the beginning of its path.");
        }
        self.path_idx -= 1;
    }

    /// Check whether this packet is absorbed.
    pub fn is_absorbed(&self) -> bool {
        self.path_idx == self.path.len()
    }

    /// Check whether this packet should be absorbed the next time it is forwarded.
    pub fn should_be_absorbed(&self) -> bool {
        self.path_idx == self.path.len() - 1
    }

    /// Get the injection round of this `Packet`.
    pub fn get_injection_rd(&self) -> usize {
        self.injection_rd
    }

    /// Get the id of the current `Node` that this packet occupies. Returns `None` if the packet
    /// has been absorbed.
    pub fn cur_node(&self) -> Option<NodeID> {
        match self.path.get(self.path_idx) {
            Some(next_id) => Some(*next_id),
            None => None,
        }
    }

    /// Get the id of the next `Node` that this packet will occupy if forwarded in its path.
    /// Returns `None` if the packet has been absorbed or is about to be absorbed.
    pub fn next_node(&self) -> Option<NodeID> {
        match self.path.get(self.path_idx + 1) {
            Some(next_id) => Some(*next_id),
            None => None,
        }
    }

    /// Get the number of steps that this packet needs to travel in the network in order to be
    /// absorbed, including the absorption step.
    pub fn dist_to_go(&self) -> usize {
        self.path.len() - self.path_idx + 1
    }

    /// Get the current index into the `PacketPath`.
    pub fn get_path_idx(&self) -> usize {
        self.path_idx
    }

    /// Get a reference to this packet's path.
    pub fn get_path(&self) -> &PacketPath {
        &self.path
    }

    /// Get a mutable reference to this packet's path.
    pub fn get_path_mut(&mut self) -> &mut PacketPath {
        &mut self.path
    }
}

impl fmt::Debug for Packet {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Packet")
            .field("id", &self.id)
            .field("cur_node", &self.cur_node())
            .field("injection_rd", &self.injection_rd)
            .finish()
    }
}

impl PartialEq for Packet {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

/// This struct allows for the creation of `Packet`s with unique ids. We thus require all `Packet`s
/// to be created through a `PacketFactory`.
#[derive(Default, Clone)]
pub struct PacketFactory {
    cur_id: usize,
}

impl PacketFactory {
    /// Create a new `PacketFactory`.
    pub fn new() -> Self {
        PacketFactory { cur_id: 0 }
    }

    /// Create a new `Packet`.
    pub fn create_packet(
        &mut self,
        path: PacketPath,
        injection_rd: usize,
        path_idx: usize,
    ) -> Packet {
        let p = Packet {
            id: self.cur_id,
            path,
            path_idx,
            injection_rd,
        };
        self.cur_id += 1;
        p
    }
}

/// The path of `Node`s that a `Packet` will take through a `Network`.
pub type PacketPath = Vec<NodeID>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic]
    fn test_iter_through_path() {
        let mut packet_factory = PacketFactory::new();
        let mut p = packet_factory.create_packet(vec![1, 4, 9, 16], 0, 0);
        assert_eq!(p.get_path_idx(), 0);
        assert_eq!(p.cur_node().unwrap(), 1);
        assert_eq!(p.next_node().unwrap(), 4);
        assert_eq!(p.dist_to_go(), 4);

        p.increment_path_idx();

        assert_eq!(p.get_path_idx(), 1);
        assert_eq!(p.cur_node().unwrap(), 4);
        assert_eq!(p.next_node().unwrap(), 9);
        assert_eq!(p.dist_to_go(), 3);

        p.increment_path_idx();
        p.increment_path_idx();

        assert_eq!(p.dist_to_go(), 1);
        assert_eq!(p.next_node(), None);
        p.increment_path_idx();
        assert_eq!(p.dist_to_go(), 0);
        assert_eq!(p.cur_node(), None);
        assert_eq!(p.next_node(), None);
        assert!(p.is_absorbed());

        // Should panic here; we don't want to allow iteration if the packet is already absorbed.
        p.increment_path_idx();
    }
}
