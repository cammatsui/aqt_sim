//! This module contains the `Threshold` trait and its implementations, which determine when a
//! `Simulation` should stop running.

use crate::network::Network;


/// Used to end a `Simulation`.
pub trait Threshold {
    /// Check whether to terminate the simulation and update any internal state of the
    /// `Threshold.`.
    fn check_termination(&mut self, rd: usize, network: &Network) -> bool;
}


/// To end a `Simulatoin` after a specified number of rounds has elapsed.
pub struct TimedThreshold {
    max_rds: usize,
}

impl TimedThreshold {
    /// Create a new `TimedThreshold` with the given number of maximum rounds.
    pub fn new(max_rds: usize) -> Self {
        TimedThreshold { max_rds }
    }
}

impl Threshold for TimedThreshold {
    fn check_termination(&mut self, rd: usize, _network: &Network) -> bool {
        rd >= self.max_rds
    }
}

