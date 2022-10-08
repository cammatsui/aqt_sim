//! This module contains functions for comparing packet priorities according to different criteria.
use crate::protocol::Packet;
use std::cmp::Ordering;

/// Returns whether `p` has higher priority than `q` under LIS.
pub fn lis_higher_priority(p: &Packet, q: &Packet) -> bool {
    match p.injection_rd().cmp(&q.injection_rd()) {
        Ordering::Less => true,
        Ordering::Equal => p.id() < q.id(),
        Ordering::Greater => false,
    }
}
