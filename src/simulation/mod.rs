//! This module contains the `Simulation` struct, which holds the data for the simulation, the
//! `Recorder` trait and its implementations, which "records" snapshots of the simulation, and the
//! `Threshold` trait and its implementations, which determines when to stop the simulation.

use std::fs;
use std::io::prelude::*;
use serde::{ Serialize, Deserialize };
use crate::network::Network;
use crate::protocol::Protocol;
use crate::adversary::Adversary;
use crate::simulation::threshold::Threshold;
use crate::simulation::recorder::{ Recorder, DebugPrintRecorder };

pub mod threshold;
pub mod recorder;


/// Stores all data related to a run of a simulation, including the `Network`, `Protocol`, and
/// `Adversary`.
pub struct Simulation {
    network: Network,
    protocol: Protocol,
    adversary: Adversary,
    threshold: Threshold,
    recorders: Vec<Recorder>,
}

impl Simulation {
    /// Get a new `Simulation` from the given (deserialized) `SimulationConfig`.
    pub fn from_config(config: SimulationConfig) -> Self {
        Simulation::new(
            Network::from_graph_structure(config.adjacency),
            config.protocol,
            config.adversary,
            config.threshold,
            config.recorders,
            config.output_path,
        )
    }

    /// Write this `Simulation` to a `SimulationConfig` for serialization.
    pub fn to_config(&self, output_path: String) -> SimulationConfig {
        SimulationConfig {
            adjacency: self.network.graph_structure(),
            protocol: self.protocol.clone(),
            adversary: self.adversary.clone(),
            threshold: self.threshold.clone(),
            recorders: self.recorders.clone(),
            output_path,
        }
    }
    
    /// Create a new `Simulation`. Use this to run non-debug sims.
    pub fn new(
        network: Network,
        protocol: Protocol,
        adversary: Adversary,
        threshold: Threshold,
        recorders: Vec<Recorder>,
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
        protocol: Protocol,
        adversary: Adversary,
        threshold: Threshold,
    ) -> Self {
        let recorders: Vec<Recorder> = vec![Recorder::DebugPrint(DebugPrintRecorder::new())];
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
        let config = self.to_config(String::from(output_path));
        config.save_to_file(output_path);
    }
}


/// Represents a configuration for a `Simulation` struct.
#[derive(Serialize, Deserialize)]
pub struct SimulationConfig {
    adjacency: Vec<Vec<usize>>,
    protocol: Protocol,
    adversary: Adversary,
    threshold: Threshold,
    #[serde(skip_serializing)]
    recorders: Vec<Recorder>,
    output_path: String,
}

impl SimulationConfig {
    const SIM_CONFIG_FILENAME: &'static str = "sim_config.json";

    /// Save this config to the given file.
    pub fn save_to_file(&self, output_path: &str) {
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

    /// Read a vector of `SimulationConfig`s from a file.
    pub fn read_configs_from_file(&self, file_path: &str) -> Vec<SimulationConfig> {
        let json = fs::read_to_string(file_path).unwrap();
        serde_json::from_str(&json).unwrap()
    }
}
