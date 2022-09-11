//! This module contains types related to the underlying graph data structure and buffers. All of
//! these types are wrapped by the `Network` struct. Access to and modification of the network is
//! done via `NodeID`s, indices into the network's `Node` vector, where nodes are referenced by 
//! IDs and `EdgeBuffers` are referenced by pairs of from- and to-IDs.

use hashbrown::HashMap;
use crate::packet::Packet;
use std::fmt;


/// The `Network` struct wraps the underlying graph data structure and manages the buffers of
/// packets. The struct offers the following interface:
///
/// Construction
/// - Create a new network:
///     `Network::new()`,
/// - Add a new `Node` to the network:
///     `let node_id = network.add_node()`,
/// - Add an new `EdgeBuffer` to the network:
///     `network.add_edgebuffer(from_id, to_id, capacity)`.
///
/// Access
/// - Get vector of neighbor IDs of a node:
///     `network.get_neighbors(node_id)``,
/// - Get a vector of the graph's nodes' IDs:
///     `network.get_nodes()`,
/// - Get a vector of the graph's edgebuffers' ID pairs:
///     `network.get_edgebuffers()`.
///
/// Buffer Access/Modification
/// - Add a given `Packet` into an `EdgeBuffer` from the given edgebuffer ID pair:
///     `network.add_packet(packet, from_id, to_id)`,
/// - Get an immutable reference to a `Buffer` from the given edgebuffer ID pair:
///     `network.get_buffer(from_id, to_id)`,
/// - Get a mutable reference to a `Buffer`` from the given edgebuffer ID pair:
///     `network.get_buffer_mut(from_id, to_id)`,
/// - Get and take a `Buffer` and replace it with a new empty `Buffer`:
///     `network.take_buffer(from_id, to_id)`.
pub struct Network {
    nodes: Vec<Node>,
}

impl Network {
    /// Get a new empty `Network`.
    pub fn new() -> Self {
        Network { nodes: Vec::new() }
    }

    /// Add a new `Node` to the network.
    pub fn add_node(&mut self) -> NodeID {
        let node_id = self.nodes.len();
        self.nodes.push(Node::new());
        node_id
    }

    /// Add a new empty `EdgeBuffer` between two nodes. Panics if one of the given IDs is invalid
    /// for this network or if there is already an edgebuffer between these two nodes.
    pub fn add_edgebuffer(&mut self, from_id: NodeID, to_id: NodeID) {
        self.check_node_id(to_id);
        self.check_node_id(from_id);

        let from_node: &mut Node = &mut self.nodes[from_id];
        if from_node.edgebuffer_map.contains_key(&to_id) {
            panic!("There is already an EdgeBuffer between nodes {} and {}", from_id, to_id);
        }

        from_node.edgebuffer_map.insert(to_id, EdgeBuffer::new());
    }

    /// Get a vector of the given node's neighbors' node ids.
    pub fn get_neighbors(&self, node_id: NodeID) -> Vec<NodeID> {
        self.check_node_id(node_id);
        let node = &self.nodes[node_id];
        let mut result = Vec::new();
        for neighbor_id in node.edgebuffer_map.keys() {
            result.push(neighbor_id.clone());
        }
        result
    }

    /// Get a vector of all nodes' node ids in sorted order.
    pub fn get_nodes(&self) -> Vec<NodeID> {
        (0..self.nodes.len()).collect()
    }

    /// Get the number of nodes in this network.
    pub fn get_num_nodes(&self) -> usize {
        self.nodes.len()
    }

    /// Get a vector of all edgebuffer ID pairs, sorted by the first `NodeID` in the pair.
    pub fn get_edgebuffers(&self) -> Vec<(NodeID, NodeID)> {
        let mut result = Vec::new();
        for from_id in 0..self.nodes.len() {
            for to_id in self.get_neighbors(from_id) {
                result.push((from_id, to_id))
            } 
        }
        result
    }

    /// Add the given `Packet` to the specified `Buffer`. Returns `None` if there is no 
    /// `EdgeBuffer` corresponding to the given from- and to-IDs.
    pub fn add_packet(&mut self, p: Packet, from_id: NodeID, to_id: NodeID) {
        match self.get_edgebuffer_mut(from_id, to_id) {
            Some(eb) => eb.buffer.push(p),
            None => panic!("No EdgeBuffer between Nodes {} and {}.", from_id, to_id),
        }
    }

    /// Get an immutable reference to the specified `Buffer`. Returns `None` if there is no 
    /// `EdgeBuffer` corresponding to the given from- and to-IDs.
    pub fn get_edgebuffer(&self, from_id: NodeID, to_id: NodeID) -> Option<&EdgeBuffer> {
        self.check_node_id(from_id);
        self.check_node_id(to_id);
        match self.nodes[from_id].edgebuffer_map.get(&to_id) {
            Some(eb) => Some(&eb),
            None => None,
        }
    }

    /// Get an mutable reference to the specified `Buffer`. Returns `None` if there is no 
    /// `EdgeBuffer` corresponding to the given from- and to-NodeIDs.
    pub fn get_edgebuffer_mut(
        &mut self,
        from_id: NodeID,
        to_id: NodeID
    ) -> Option<&mut EdgeBuffer> {
        self.check_node_id(from_id);
        self.check_node_id(to_id);
        match self.nodes[from_id].edgebuffer_map.get_mut(&to_id) {
            Some(eb) => Some(eb),
            None => None,
        }
    }

    /// Get (and take ownership of) the specified `Buffer`. Returns `None` if there is no 
    /// `EdgeBuffer` corresponding to the given from- and to-IDs.
    pub fn take_buffer(&mut self, from_id: NodeID, to_id: NodeID) -> Option<Buffer> {
        self.check_node_id(from_id);
        self.check_node_id(to_id);
        match self.nodes[from_id].edgebuffer_map.get_mut(&to_id) {
            Some(eb) => {
                let mut buffer = Vec::new();
                std::mem::swap(&mut buffer, &mut eb.buffer);
                Some(buffer)
            }
            None => None,
        }
    }

    fn check_node_id(&self, node_id: NodeID) {
        if node_id >= self.nodes.len() {
            panic!("No Node with ID {} in this network.", node_id);
        }
    }
}

impl fmt::Display for Network {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut result = String::new();
        let edgebuffer_ids = self.get_edgebuffers();
        for (from_id, to_id) in edgebuffer_ids {
            let buffer = &self.get_edgebuffer(from_id, to_id).unwrap()
                .buffer;
            result.push_str(&format!("{}, {}: {:?}\n", from_id, to_id, buffer));
        }
        write!(f, "{}", result)
    }
}


pub struct Node {
    pub edgebuffer_map: HashMap<NodeID, EdgeBuffer>,
}

impl Node {
    pub fn new() -> Self {
        Node { edgebuffer_map: HashMap::new() }
    }
}


/// An `EdgeBuffer` represents an edge in the graph with an associated `Buffer` (just a vector of
/// `Packet`s).
pub struct EdgeBuffer {
    pub buffer: Buffer,
}

impl EdgeBuffer {
    /// Get a new empty `EdgeBuffer`.
    pub fn new() -> Self {
        EdgeBuffer { buffer: Vec::new() }
    }
}


/// A `NodeID` uniquely specifies a `Node` in the network. These IDs are also used, in pairs, to 
/// uniquely specify `EdgeBuffer`s in the network..
pub type NodeID = usize;

/// Just a vector of `Packet`s.
pub type Buffer = Vec<Packet>;


pub mod presets {
    //! This module contains functions to create preset network structures.
    use super::Network;

    /// Construct a path network with the given number of buffers.
    pub fn construct_path(num_buffers: usize) -> Network {
        let mut network = Network::new();
        for _ in 0..num_buffers+1 {
            network.add_node();
        }

        for buff_id in 0..num_buffers {
            network.add_edgebuffer(buff_id, buff_id+1);
        }

        network
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::packet::PacketFactory;

    fn setup_test_graph() -> Network {
        let mut network = Network::new();

        let a_id = network.add_node();
        let b_id = network.add_node();
        let c_id = network.add_node();
        let d_id = network.add_node();

        network.add_edgebuffer(a_id, b_id);
        network.add_edgebuffer(a_id, c_id);
        network.add_edgebuffer(c_id, b_id);
        network.add_edgebuffer(b_id, c_id);
        network.add_edgebuffer(a_id, d_id);
        network.add_edgebuffer(b_id, d_id);

        network
    }

    #[test]
    fn test_get_neighbors() {
        let network = setup_test_graph();

        let (a_id, b_id, c_id, d_id) = (0, 1, 2, 3);

        let a_neighbors = network.get_neighbors(a_id);
        let b_neighbors = network.get_neighbors(b_id);
        let c_neighbors = network.get_neighbors(c_id);
        let d_neighbors = network.get_neighbors(d_id);

        let expect_a_neighbors = vec![b_id, c_id, d_id];
        let expect_b_neighbors = vec![c_id, d_id];
        let expect_c_neighbors = vec![b_id];
        let expect_d_neighbors: Vec<NodeID> = vec![];
        
        assert!(a_neighbors.into_iter().all(|neighbor| expect_a_neighbors.contains(&neighbor)));
        assert!(b_neighbors.into_iter().all(|neighbor| expect_b_neighbors.contains(&neighbor)));
        assert!(c_neighbors.into_iter().all(|neighbor| expect_c_neighbors.contains(&neighbor)));
        assert!(d_neighbors.into_iter().all(|neighbor| expect_d_neighbors.contains(&neighbor)));
    }


    #[test]
    #[should_panic]
    fn test_add_edgebuffer_panic() {
        let mut network = setup_test_graph();
        network.add_edgebuffer(0, 10);
    }

    #[test]
    fn test_get_nodes() {
        let network = setup_test_graph();
        let node_ids = network.get_nodes();
        let expect_node_ids = vec![0, 1, 2, 3];
        assert!(node_ids.into_iter().all(|node_id| expect_node_ids.contains(&node_id)));
    }

    #[test]
    fn test_get_edgebuffers() {
        let network = setup_test_graph();
        let (a_id, b_id, c_id, d_id) = (0, 1, 2, 3);
        let eb_ids = network.get_edgebuffers();
        let expect_eb_ids = vec![
            (a_id, b_id),
            (a_id, c_id),
            (c_id, b_id),
            (b_id, c_id),
            (a_id, d_id),
            (b_id, d_id),
        ];
        assert!(eb_ids.into_iter().all(|eb_id_pair| expect_eb_ids.contains(&eb_id_pair)))
    }

    #[test]
    fn test_add_packet_and_get_edgebuffer() {
        let mut network = setup_test_graph();
        let (a_id, b_id) = (0, 1);
        let mut factory = PacketFactory::new();
        let p = factory.create_packet(Vec::new(), 0, 0);
        let p2 = p.clone();

        { 
            let eb = network.get_edgebuffer(a_id, b_id).unwrap();
            assert!(!eb.buffer.contains(&p2));
        }

        network.add_packet(p, a_id, b_id);
        let eb = network.get_edgebuffer(a_id, b_id).unwrap();
        assert!(eb.buffer.contains(&p2));
    }

    #[test]
    fn test_add_packet_and_take_buffer() {
        let mut network = setup_test_graph();
        let (b_id, d_id) = (1, 3);
        let mut factory = PacketFactory::new();
        let p = factory.create_packet(Vec::new(), 0, 0);
        let p2 = p.clone();
        network.add_packet(p, b_id, d_id);
        
        let buff = network.take_buffer(b_id, d_id).unwrap();
        assert!(buff.contains(&p2));

        let new_eb = network.get_edgebuffer(b_id, d_id).unwrap();
        assert!(new_eb.buffer.len() == 0);
    }
}
