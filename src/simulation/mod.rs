//! This module contains the `Simulation` struct, which holds the data for the simulation, the
//! `Recorder` trait and its implementations, which "records" snapshots of the simulation, and the
//! `Threshold` trait and its implementations, which determines when to stop the simulation.

use std::fs;
use std::io::prelude::*;
use serde::Serialize;
use crate::network::Network;
use crate::protocol::Protocol;
use crate::adversary::Adversary;
use crate::simulation::threshold::Threshold;
use crate::simulation::recorder::{ Recorder, DebugPrintRecorder };

pub mod threshold;
pub mod recorder;


/// Stores all data related to a run of a simulation, including the `Network`, `Protocol`, and
/// `Adversary`.
#[derive(Serialize)]
pub struct Simulation<P, A, T> where P: Protocol, A: Adversary, T: Threshold {
    network: Network,
    protocol: P,
    adversary: A,
    threshold: T,
    #[serde(skip_serializing)]
    recorders: Vec<Box<dyn Recorder>>,
}

impl<P, A, T> Simulation<P, A, T> where 
    P: Protocol + Serialize,
    A: Adversary + Serialize,
    T: Threshold + Serialize,
{
    const SIM_CONFIG_FILENAME: &'static str = "sim_config.json";
    
    /// Create a new `Simulation`. Use this to run non-debug sims.
    pub fn new(
        network: Network,
        protocol: P,
        adversary: A,
        threshold: T,
        recorders: Vec<Box<dyn Recorder>>,
        output_path: String,
    ) -> Self {
        let mut new_sim = Simulation { network, protocol, adversary, threshold, recorders };
        // TODO: save sim details to json with serde. For now just save network.
        new_sim.save_config(&output_path);
        for recorder in &mut new_sim.recorders {
            recorder.set_output_path(output_path.clone())
        }
        new_sim
    }

    /// Create a new `Simulation` with a `DebugPrintRecorder` pre-added.
    pub fn new_debug(
        network: Network,
        protocol: P,
        adversary: A,
        threshold: T,
    ) -> Self {
        let recorders: Vec<Box<dyn Recorder>> = vec![Box::new(DebugPrintRecorder::new())];
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
                recorder.record(rd, false, &self.network, None);
            }

            // Forward.
            let absorbed = self.protocol.forward_packets(&mut self.network);
            
            for recorder in &mut self.recorders {
                recorder.record(rd, true, &self.network, Some(&absorbed));
            }
            if self.threshold.check_termination(rd, &self.network) { break };
            rd += 1;
        }
        for recorder in &mut self.recorders { recorder.close() }
    }

    fn save_config(&self, output_path: &str) {
        let data = serde_json::to_string_pretty(&self).unwrap();
        fs::create_dir_all(output_path.clone())
            .expect("Failed to save simulation results.");
        let mut filepath = String::from(output_path);
        filepath.push_str(&format!("/{}", Self::SIM_CONFIG_FILENAME));
        
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create(true)
            .open(&filepath)
            .unwrap();

        if let Err(_) = writeln!(file, "{}", data) {
            eprintln!("Couldn't write to file {}", filepath);
        }

    }
}
