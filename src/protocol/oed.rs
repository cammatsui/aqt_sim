//! This module contains implementations of OED protocols.

use super::{OED_WITH_SWAP_NAME, PROTOCOL_NAME_KEY};
use crate::config::{CfgErrorMsg, Configurable};
use crate::network::{Network, NodeID};
use crate::packet::Packet;
use crate::protocol::{priority, ProtocolTrait};
use serde_json::{Map, Value};

/// In the OED With Swap protocol, we forward the oldest packet from buffer x if x and x+1 fulfill
/// the OED criterion or the oldest packet in x is older than the youngest in x+1, and send the
/// youngest packet in x backward if L(x-1) > 0, x-1 and x fail the OED criterion, and the youngest
/// packet in x is younger than the oldest in x-1.
#[derive(Clone)]
pub struct OEDWithSwap;

impl OEDWithSwap {
    pub fn new() -> Self {
        OEDWithSwap
    }
}

impl ProtocolTrait for OEDWithSwap {
    fn forward_packets(&mut self, network: &mut Network) -> Vec<Packet> {
        let mut absorbed = Vec::new();
        let mut to_fwd_and_bwd = self.get_packets_to_fwd_and_bwd(network);
        let num_to_fwd_and_bwd = to_fwd_and_bwd.len();
        for _ in 0..num_to_fwd_and_bwd {
            let p = to_fwd_and_bwd.remove(0);
            if !p.should_be_absorbed() {
                self.add_packet(p, network)
            } else {
                absorbed.push(p)
            };
        }
        absorbed
    }
}

impl OEDWithSwap {
    /// Get a vector of packets we need to move according to OED with swap.
    fn get_packets_to_fwd_and_bwd(&mut self, network: &mut Network) -> Vec<Packet> {
        let mut result = Vec::new();
        let forward_or_backward = self.get_should_forward_or_backward(network);
        let num_nodes = network.get_num_nodes();
        for from_id in 0..num_nodes - 1 {
            let to_id = from_id + 1;
            let eb = network.get_edgebuffer_mut(from_id, to_id).unwrap();
            let load = eb.buffer.len();
            if load == 0 {
                continue;
            }
            let (forward, backward) = forward_or_backward[from_id];
            if forward {
                let o_idx = self.highest_priority_idx(from_id, to_id, network).unwrap();
                let buffer = &mut network.get_edgebuffer_mut(from_id, to_id).unwrap().buffer;
                let mut p = buffer.remove(o_idx);
                p.increment_path_idx();
                result.push(p);
            }
            if backward {
                let y_idx = self.lowest_priority_idx(from_id, to_id, network).unwrap();
                let buffer = &mut network.get_edgebuffer_mut(from_id, to_id).unwrap().buffer;
                let mut p = buffer.remove(y_idx);
                p.decrement_path_idx();
                result.push(p);
            }
        }

        result
    }

    fn buffer_oldest_youngest_packets<'a>(
        &self,
        from_id: NodeID,
        to_id: NodeID,
        network: &'a Network,
    ) -> Option<(&'a Packet, &'a Packet)> {
        let eb = network.get_edgebuffer(from_id, to_id).unwrap();
        let load = eb.buffer.len();
        if load == 0 {
            return None;
        }

        let o_idx = self.highest_priority_idx(from_id, to_id, network).unwrap();
        let y_idx = self.lowest_priority_idx(from_id, to_id, network).unwrap();

        Some((&eb.buffer[o_idx], &eb.buffer[y_idx]))
    }

    /// Get the index of the highest priority packet (lexicographically, smallest injection rd 
    /// then smallest id) in the given buffer.
    fn highest_priority_idx(
        &self,
        from_id: NodeID,
        to_id: NodeID,
        network: &Network,
    ) -> Option<usize> {
        let eb = network.get_edgebuffer(from_id, to_id).unwrap();
        let load = eb.buffer.len();
        if load == 0 {
            return None;
        }

        let mut hipri_packet = &eb.buffer[0];
        let mut hipri_idx = 0;
        for i in 1..eb.buffer.len() {
            if priority::lis_higher_priority(&eb.buffer[i], hipri_packet) {
                hipri_packet = &eb.buffer[i];
                hipri_idx = i;
            }
        }
        Some(hipri_idx)
    }

    /// Get the index of the lowest priority packet (lexicographically, largest injection rd 
    /// then largest id) in the given buffer.
    fn lowest_priority_idx(
        &self,
        from_id: NodeID,
        to_id: NodeID,
        network: &Network,
    ) -> Option<usize> {
        let eb = network.get_edgebuffer(from_id, to_id).unwrap();
        let load = eb.buffer.len();
        if load == 0 {
            return None;
        }

        let mut lopri_packet = &eb.buffer[0];
        let mut lopri_idx = 0;
        for i in 1..eb.buffer.len() {
            if priority::lis_higher_priority(lopri_packet, &eb.buffer[i]) {
                lopri_packet = &eb.buffer[i];
                lopri_idx = i;
            }
        }
        Some(lopri_idx)
    }

    /// Get a vector of `elt = (bool, bool)` indexed by from-ID where `elt.0` is whether the buffer
    /// outgoing from the given from-ID should forward a packet, and `elt.1` is whether this
    /// buffer should send a packet backward.
    fn get_should_forward_or_backward(&self, network: &mut Network) -> Vec<(bool, bool)> {
        // Calculate OED criterion for each buffer.
        let mut oed_criterion = Vec::new();
        let num_nodes = network.get_num_nodes();
        for from_id in 0..num_nodes - 2 {
            let this_eb = network.get_edgebuffer(from_id, from_id + 1).unwrap();
            let this_load = this_eb.buffer.len();
            let next_eb = network.get_edgebuffer(from_id + 1, from_id + 2).unwrap();
            let next_load = next_eb.buffer.len();
            let oed = this_load > next_load || (this_load == next_load && this_load % 2 == 1);
            oed_criterion.push(oed);
        }
        let maybe_last_eb = network.get_edgebuffer(num_nodes - 2, num_nodes - 1);
        let last_nonempty = maybe_last_eb.unwrap().buffer.len() > 0;
        oed_criterion.push(last_nonempty);

        // Get max/min packet refs for each buffer.
        let mut oldest_youngest = Vec::new();
        for from_id in 0..num_nodes - 1 {
            let to_id = from_id + 1;
            oldest_youngest.push(self.buffer_oldest_youngest_packets(from_id, to_id, network));
        }

        // Use OED with Swapping protocol to determine whether each buffer should send a packet
        // forward and/or backward. For a tuple in result, the first idx is whether to forward, the
        // second is whether to send a packet backward.
        let mut result = Vec::new();
        for from_id in 0..num_nodes - 1 {
            let this_oldest_youngest = oldest_youngest[from_id];
            if this_oldest_youngest == None {
                result.push((false, false));
                continue;
            }
            let (this_oldest, this_youngest) = this_oldest_youngest.unwrap();

            let should_fwd;
            if from_id != num_nodes - 2 {
                let next_oldest_youngest = oldest_youngest[from_id + 1];
                should_fwd = oed_criterion[from_id]
                    || priority::lis_higher_priority(this_oldest, next_oldest_youngest.unwrap().1)
            } else {
                // Always forward for the last buffer since at this point we know the last buffer
                // is nonempty.
                should_fwd = true;
            }

            let mut should_bwd = false;
            if from_id != 0 {
                let prev_oldest_youngest = oldest_youngest[from_id - 1];
                should_bwd = prev_oldest_youngest != None
                    && (!oed_criterion[from_id - 1]
                        && priority::lis_higher_priority(
                            prev_oldest_youngest.unwrap().0,
                            this_youngest,
                        ));
            }

            result.push((should_fwd, should_bwd));
        }
        result
    }
}

impl Configurable for OEDWithSwap {
    fn from_config(_config: Value) -> Result<Self, CfgErrorMsg> {
        Ok(Self)
    }

    fn to_config(&self) -> Value {
        let mut map: Map<String, Value> = Map::new();
        map.insert(
            PROTOCOL_NAME_KEY.to_string(),
            Value::String(OED_WITH_SWAP_NAME.to_string()),
        );
        Value::Object(map)
    }
}

#[cfg(test)]
mod tests {
    use super::OEDWithSwap;
    use super::ProtocolTrait;
    use crate::network::presets::construct_path;
    use crate::network::Network;
    use crate::packet::{PacketFactory, PacketPath};

    const PATH_LEN: usize = 10;

    fn setup_network_and_packet_path() -> (Network, PacketPath) {
        (construct_path(PATH_LEN), (0..PATH_LEN).collect())
    }

    #[test]
    fn test_absorption() {
        let (mut network, packet_path) = setup_network_and_packet_path();
        let mut factory = PacketFactory::new();
        // 0
        // -  ==>  -
        // 9       9
        //
        let p1 = factory.create_packet(packet_path.clone(), 0, 8);
        let mut oed = OEDWithSwap::new();
        oed.add_packet(p1, &mut network);

        oed.forward_packets(&mut network);

        let b9 = &network.get_edgebuffer(8, 9).unwrap().buffer;
        assert_eq!(b9.len(), 0);
    }

    #[test]
    fn test_absorption_multiple_packets_in_last() {
        let (mut network, packet_path) = setup_network_and_packet_path();
        let mut factory = PacketFactory::new();
        // 1
        // 0       1
        // -  ==>  -
        // 9       9
        //
        let p1 = factory.create_packet(packet_path.clone(), 0, 8);
        let p2 = factory.create_packet(packet_path.clone(), 1, 8);
        let p2_c = p2.clone();
        let mut oed = OEDWithSwap::new();
        oed.add_packet(p1, &mut network);
        oed.add_packet(p2, &mut network);

        oed.forward_packets(&mut network);

        let b9 = &network.get_edgebuffer(8, 9).unwrap().buffer;
        assert!(b9.contains(&p2_c));
    }

    #[test]
    fn test_forward_oldest() {
        let (mut network, packet_path) = setup_network_and_packet_path();
        let mut factory = PacketFactory::new();
        // 1
        // 0         1 0
        // ---  ==>  ---
        // 0 1       0 1
        //
        let p1 = factory.create_packet(packet_path.clone(), 0, 0);
        let p2 = factory.create_packet(packet_path.clone(), 1, 0);

        let (p1_c, p2_c) = (p1.clone(), p2.clone());

        network.add_packet(p1, 0, 1);
        network.add_packet(p2, 0, 1);

        let mut oed = OEDWithSwap::new();
        oed.forward_packets(&mut network);

        let b1 = &network.get_edgebuffer(0, 1).unwrap().buffer;
        let b2 = &network.get_edgebuffer(1, 2).unwrap().buffer;

        assert!(b1.contains(&p2_c));
        assert!(b2.contains(&p1_c));
    }

    #[test]
    fn test_even_swap() {
        let (mut network, packet_path) = setup_network_and_packet_path();
        let mut factory = PacketFactory::new();
        // 3 2       3
        // 0 1       2 0 1
        // ----- ==> -----
        // 0 1 2     0 1 2
        let p1 = factory.create_packet(packet_path.clone(), 0, 0);
        let p2 = factory.create_packet(packet_path.clone(), 1, 1);
        let p3 = factory.create_packet(packet_path.clone(), 2, 1);
        let p4 = factory.create_packet(packet_path.clone(), 3, 0);

        let (p1_c, p2_c, p3_c, p4_c) = (p1.clone(), p2.clone(), p3.clone(), p4.clone());

        network.add_packet(p1, 0, 1);
        network.add_packet(p2, 1, 2);
        network.add_packet(p3, 1, 2);
        network.add_packet(p4, 0, 1);

        let mut oed = OEDWithSwap::new();
        oed.forward_packets(&mut network);

        let b1 = &network.get_edgebuffer(0, 1).unwrap().buffer;
        let b2 = &network.get_edgebuffer(1, 2).unwrap().buffer;
        let b3 = &network.get_edgebuffer(2, 3).unwrap().buffer;

        assert!(b1.contains(&p4_c));
        assert!(b1.contains(&p3_c));
        assert!(b2.contains(&p1_c));
        assert!(b3.contains(&p2_c));
    }

    #[test]
    fn test_odd_no_swap() {
        let (mut network, packet_path) = setup_network_and_packet_path();
        let mut factory = PacketFactory::new();
        // 4 5         0
        // 3 2       4 5
        // 0 1       3 2 1
        // ----- ==> -----
        // 0 1 2     0 1 2
        //
        let p1 = factory.create_packet(packet_path.clone(), 0, 0);
        let p2 = factory.create_packet(packet_path.clone(), 1, 1);
        let p3 = factory.create_packet(packet_path.clone(), 2, 1);
        let p4 = factory.create_packet(packet_path.clone(), 3, 0);
        let p5 = factory.create_packet(packet_path.clone(), 4, 0);
        let p6 = factory.create_packet(packet_path.clone(), 5, 1);

        let (p1_c, p2_c, p3_c, p4_c, p5_c, p6_c) = (
            p1.clone(),
            p2.clone(),
            p3.clone(),
            p4.clone(),
            p5.clone(),
            p6.clone(),
        );

        network.add_packet(p1, 0, 1);
        network.add_packet(p2, 1, 2);
        network.add_packet(p3, 1, 2);
        network.add_packet(p4, 0, 1);
        network.add_packet(p5, 0, 1);
        network.add_packet(p6, 1, 2);

        let mut oed = OEDWithSwap::new();
        oed.forward_packets(&mut network);

        let b1 = &network.get_edgebuffer(0, 1).unwrap().buffer;
        let b2 = &network.get_edgebuffer(1, 2).unwrap().buffer;
        let b3 = &network.get_edgebuffer(2, 3).unwrap().buffer;

        assert!(b1.contains(&p4_c));
        assert!(b1.contains(&p5_c));
        assert!(b2.contains(&p6_c));
        assert!(b2.contains(&p3_c));
        assert!(b2.contains(&p1_c));
        assert!(b3.contains(&p2_c));
    }

    #[test]
    fn test_odd_swap() {
        let (mut network, packet_path) = setup_network_and_packet_path();
        let mut factory = PacketFactory::new();
        //   0
        // 1 2       2 1 0
        // ----- ==> -----
        // 0 1 2     0 1 2
        //
        let p1 = factory.create_packet(packet_path.clone(), 0, 1);
        let p2 = factory.create_packet(packet_path.clone(), 1, 0);
        let p3 = factory.create_packet(packet_path.clone(), 2, 1);

        let (p1_c, p2_c, p3_c) = (p1.clone(), p2.clone(), p3.clone());

        network.add_packet(p1, 1, 2);
        network.add_packet(p2, 0, 1);
        network.add_packet(p3, 1, 2);

        let mut oed = OEDWithSwap::new();
        oed.forward_packets(&mut network);

        let b1 = &network.get_edgebuffer(0, 1).unwrap().buffer;
        let b2 = &network.get_edgebuffer(1, 2).unwrap().buffer;
        let b3 = &network.get_edgebuffer(2, 3).unwrap().buffer;

        assert!(b1.contains(&p3_c));
        assert!(b2.contains(&p2_c));
        assert!(b3.contains(&p1_c));
    }
}
