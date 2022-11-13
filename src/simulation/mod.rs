//! This module contains the `Simulation` struct, which holds the data for the simulation, the
//! `Recorder` trait and its implementations, which "records" snapshots of the simulation, and the
//! `Threshold` trait and its implementations, which determines when to stop the simulation.

use crate::adversary::Adversary;
use crate::config;
use crate::config::{Configurable, SimConfig};
use crate::network::Network;
use crate::protocol::Protocol;
use crate::simulation::recorder::Recorder;
use crate::simulation::threshold::Threshold;
use serde_json::{Map, Value};
use std::fs;
use std::io::prelude::*;

pub mod random;
pub mod recorder;
pub mod threshold;

/// Stores all data related to a run of a simulation, including the `Network`, `Protocol`, and
/// `Adversary`.
pub struct Simulation {
    network: Network,
    protocol: Protocol,
    adversary: Adversary,
    threshold: Threshold,
    recorders: Vec<Recorder>,
    output_path: String,
}

const SIM_CONFIG_FILENAME: &str = "sim_config.json";

impl Simulation {
    /// Create a new `Simulation`. Use this to run non-debug sims.
    pub fn new(
        network: Network,
        protocol: Protocol,
        adversary: Adversary,
        threshold: Threshold,
        recorders: Vec<Recorder>,
        output_path: String,
    ) -> Self {
        let mut new_sim = Simulation {
            network,
            protocol,
            adversary,
            threshold,
            recorders,
            output_path: output_path.clone(),
        };
        new_sim.save_config(&output_path);
        for recorder in &mut new_sim.recorders {
            recorder.set_output_path(output_path.clone())
        }
        new_sim
    }

    /// Create a new `Simulation` from the provided `SimConfig`.
    pub fn from_config(cfg: SimConfig) -> Self {
        let recorders = cfg
            .recorder_cfgs
            .as_array()
            .unwrap()
            .iter()
            .map(|c| Recorder::from_config(c.clone()).unwrap())
            .collect();

        Simulation::new(
            Network::from_config(cfg.graph_adjacency).unwrap(),
            Protocol::from_config(cfg.protocol_cfg).unwrap(),
            Adversary::from_config(cfg.adversary_cfg).unwrap(),
            Threshold::from_config(cfg.threshold_cfg).unwrap(),
            recorders,
            cfg.output_path,
        )
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

            if self.threshold.check_termination(rd, &self.network) {
                break;
            }

            // Forward.
            let absorbed = self.protocol.forward_packets(&mut self.network);

            for recorder in &mut self.recorders {
                recorder.record(rd, true, &self.network, Some(&absorbed));
            }

            if self.threshold.check_termination(rd, &self.network) {
                break;
            }
            rd += 1;
        }
        for recorder in &mut self.recorders {
            recorder.close()
        }
    }

    fn to_config_str(&self) -> String {
        let mut map = Map::new();
        map.insert(config::ADJACENCY_KEY.to_string(), self.network.to_config());
        map.insert(config::PROTOCOL_KEY.to_string(), self.protocol.to_config());
        map.insert(
            config::ADVERSARY_KEY.to_string(),
            self.adversary.to_config(),
        );
        map.insert(
            config::THRESHOLD_KEY.to_string(),
            self.threshold.to_config(),
        );
        let recorder_cfgs = self.recorders.iter().map(|r| r.to_config()).collect();
        map.insert(
            config::RECORDERS_KEY.to_string(),
            Value::Array(recorder_cfgs),
        );
        map.insert(
            config::OUTPUT_PATH_KEY.to_string(),
            Value::String(self.output_path.clone()),
        );
        serde_json::to_string_pretty(&Value::Object(map)).unwrap()
    }

    fn save_config(&self, output_path: &str) {
        let data = self.to_config_str();
        fs::create_dir_all(output_path).unwrap();
        let file_path = format!("{}/{}", output_path, SIM_CONFIG_FILENAME);

        let mut file = fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&file_path)
            .expect(&format!(
                "Failed to save simulation config to {}",
                file_path
            ));

        if let Err(_) = writeln!(file, "{}", data) {
            eprintln!("Failed to save simulation config to {}", file_path);
        }
    }
}
