//! This module contains the `Recorder` trait and its implementations, which takes "snapshots" of
//! the simulation/network and either prints them to the console or writes them to a file.

use std::fs::OpenOptions;
use std::io::prelude::*;
use crate::network::Network;


/// Trait implemented by all recorders.
pub trait Recorder {
    fn record(&mut self, rd: usize, prime_timestep: bool, network: &Network);
    fn close(&mut self);
}


/// Print the whole network to the console.
pub struct DebugPrintRecorder;

impl DebugPrintRecorder {
    pub fn new() -> Self {
        DebugPrintRecorder {}
    }
}

impl Recorder for DebugPrintRecorder {
    fn record(&mut self, rd: usize, prime_timestep: bool, network: &Network) {
        if prime_timestep { println!("{}':", rd) } else { println!("{}:", rd) };
        println!("{}", network);
    }

    fn close(&mut self) {
        println!("Simulation finished.");
    }
}


const DEFAULT_LOCAL_LINE_LIMIT: usize = 5000;
const BUFFER_LOAD_CSV_HEADER: &str = "rd,prime,buffer_from,buffer_to,load\n";

/// Records the load in each buffer.
pub struct BufferLoadRecorder {
    local_line_limit: usize, // How many lines to keep in memory before dumping record to file. 
    lines: Vec<String>,
    output_filepath: String,
}

impl Recorder for BufferLoadRecorder {
    fn record(&mut self, rd: usize, prime_timestep: bool, network: &Network) {
        // format: rd,prime (0/1), buffer_from, buffer_to, load (see BUFFER_LOAD_CSV_HEADER)
        let prime_flag = if prime_timestep { 1 } else { 0 };
        for (from_id, to_id) in network.get_edgebuffers() {
            let load = network.get_edgebuffer(from_id, to_id).unwrap().buffer.len();
            self.write(format!("{},{},{},{},{}\n", rd, prime_flag, from_id, to_id, load));
        }
    }

    fn close(&mut self) {
        self.save()
    }
}

impl BufferLoadRecorder {
    /// Get a new `BufferLoadRecorder` with the default line limit.
    pub fn new(output_filepath: String) -> Self {
        BufferLoadRecorder {
            local_line_limit: DEFAULT_LOCAL_LINE_LIMIT,
            lines: vec![BUFFER_LOAD_CSV_HEADER.to_string()],
            output_filepath,
        }
    }

    /// Get a new `BufferLoadRecorder` with the specified line limit.
    pub fn new_with_line_limit(output_filepath: String, local_line_limit: usize) -> Self {
        BufferLoadRecorder {
            local_line_limit,
            lines: vec![BUFFER_LOAD_CSV_HEADER.to_string()],
            output_filepath,
        }
    }

    fn write(&mut self, line: String) {
        if self.lines.len() >= self.local_line_limit {
            // Write to file
            self.save();

            self.lines = Vec::new();
        }
        self.lines.push(line);
    }

    fn save(&mut self) {
        let data = self.lines.concat();
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .append(true)
            .open(&self.output_filepath)
            .unwrap();

        if let Err(e) = writeln!(file, "{}", data) {
            eprintln!("Couldn't write to file: {}", e);
        }
    }
}
