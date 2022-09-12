//! This module contains the `Simulation` struct, which holds the data for the simulation, the
//! `Recorder` trait and its implementations, which "records" snapshots of the simulation, and the
//! `Threshold` trait and its implementations, which determines when to stop the simulation.

use crate::network::Network;
use crate::protocol::Protocol;
use crate::adversary::Adversary;
use crate::simulation::threshold::Threshold;
use crate::simulation::recorder::Recorder;

pub mod threshold;
pub mod recorder;


/// Stores all data related to a run of a simulation, including the `Network`, `Protocol`, and
/// `Adversary`.
pub struct Simulation<P, A, T> where P: Protocol, A: Adversary, T: Threshold {
//pub struct Simulation<P: Protocol, A: Adversary,T: Threshold> {
    network: Network,
    protocol: P,
    adversary: A,
    threshold: T,
    recorders: Vec<Box<dyn Recorder>>,
}

//impl<P: Protocol, A: Adversary, T: Threshold> Simulation<P, A, T> {
impl<P, A, T> Simulation<P, A, T> where 
    P: Protocol,
    A: Adversary,
    T: Threshold,
{
    /// Create a new `Simulation`.
    pub fn new(
        network: Network,
        protocol: P,
        adversary: A,
        threshold: T,
        recorders: Vec<Box<dyn Recorder>>,
    ) -> Self {
        Simulation { network, protocol, adversary, threshold, recorders }
    }

    /// Run the simulation for the given number of rounds.
    pub fn run(&mut self) {
        let mut rd = 1;
        loop {
            // Inject.
            let mut packets_to_inject = self.adversary.get_next_packets(&self.network, rd);
            let num_to_inject = packets_to_inject.len();
            for _ in 0..num_to_inject {
                let p = packets_to_inject.remove(0);
                self.protocol.add_packet(p, &mut self.network);
            }

            for recorder in &mut self.recorders {
                recorder.record(rd, false, &self.network);
            }

            // Forward.
            self.protocol.forward_packets(&mut self.network);
            
            for recorder in &mut self.recorders {
                recorder.record(rd, true, &self.network);
            }
            if self.threshold.check_termination(rd, &self.network) { break };
            rd += 1;
        }
        for recorder in &mut self.recorders { recorder.close() }
    }
}
