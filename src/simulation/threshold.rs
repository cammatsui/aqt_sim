//! This module contains the `Threshold` trait and its implementations, which determine when a
//! `Simulation` should stop running.

use crate::network::Network;
use serde::{Deserialize, Serialize};

/// Used to end a `Simulation`.
#[derive(Serialize, Deserialize, Clone)]
pub enum Threshold {
    Timed(TimedThreshold),
}

impl Threshold {
    pub fn timed_from_rds(max_rds: usize) -> Self{
        Self::Timed(TimedThreshold::new(max_rds))
    }

    /// Check whether the `Simulation` should terminate based on the round number and network
    /// state.
    pub fn check_termination(&mut self, rd: usize, network: &Network) -> bool {
        match self {
            Self::Timed(t) => t.check_termination(rd, network),
        }
    }
}

/// Trait which all `Threshold`s should implement.
pub trait ThresholdTrait {
    /// Check whether to terminate the simulation and update any internal state of the
    /// `Threshold.`.
    fn check_termination(&mut self, rd: usize, network: &Network) -> bool;
}

/// To end a `Simulation` after a specified number of rounds has elapsed.
#[derive(Serialize, Deserialize, Clone)]
pub struct TimedThreshold {
    max_rds: usize,
}

impl TimedThreshold {
    /// Create a new `TimedThreshold` with the given number of maximum rounds.
    fn new(max_rds: usize) -> Self {
        TimedThreshold { max_rds }
    }
}

impl ThresholdTrait for TimedThreshold {
    fn check_termination(&mut self, rd: usize, _network: &Network) -> bool {
        rd >= self.max_rds
    }
}
