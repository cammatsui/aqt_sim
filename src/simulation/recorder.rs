//! This module contains the `Recorder` trait and its implementations, which takes "snapshots" of
//! the simulation/network and either prints them to the console or writes them to a file.

use std::fs;
use std::io::prelude::*;
use crate::network::Network;
use crate::packet::Packet;

const DEFAULT_LINE_LIMIT: usize = 5000;


/// Trait implemented by all recorders.
pub trait Recorder {
    fn record(&mut self, rd: usize, prime: bool, network: &Network, absorbed: Option<&Vec<Packet>>);
    fn set_output_path(&mut self, output_path: String);
    fn close(&mut self);
}


/// Trait implemented by all recorders which save to a file.
pub trait SaveRecorder {
    /// If the line limit has been reached, saves all of the recorder's lines to the file at the
    /// file path. Otherwise, just appends the line to lines.
    fn write(&mut self, line: String, filename: &str) {
        if self.get_lines().len() >= self.get_line_limit() {
            // Write to file
            self.save(filename);
            self.reset_lines();
        }
        self.push_line(line);
    }

    /// Append the `SaveRecorder`'s lines to the file at the recorder's output path.
    fn save(&mut self, filename: &str) {
        let data = self.get_lines().concat();

        let output_path_unwrapped = self.get_output_path()
            .expect("You must set an output path for each recorder.");
        let mut path = String::from(output_path_unwrapped);

        fs::create_dir_all(path.clone())
            .expect("Failed to save simulation results.");
        path.push('/');
        path.push_str(filename);

        let mut file = fs::OpenOptions::new()
            .write(true)
            .create(true)
            .append(true)
            .open(&path)
            .unwrap();

        if let Err(e) = writeln!(file, "{}", data) {
            eprintln!("Couldn't write to file: {}", e);
        }
    }

    /// Get the line vector.
    fn get_lines(&self) -> &Vec<String>;

    /// Push a line to the line vector.
    fn push_line(&mut self, line: String);

    /// Reset the line vector to a new empty vector.
    fn reset_lines(&mut self);

    /// Get the line limit, the number of lines to keep in memory before writing to disk.
    fn get_line_limit(&self) -> usize;

    // Get the output path which we save the results to.
    fn get_output_path(&self) -> Option<&String>;
}


/// Prints the whole network to the console each timestep.
pub struct DebugPrintRecorder;

impl DebugPrintRecorder {
    /// Get a new `DebugPrintRecorder`.
    pub fn new() -> Self {
        DebugPrintRecorder {}
    }
}

impl Recorder for DebugPrintRecorder {
    /// Does nothing since this `Recorder` does not save anything.
    fn set_output_path(&mut self, _output_path: String) { }

    fn record(
        &mut self,
        rd: usize,
        prime: bool,
        network: &Network,
        _absorbed: Option<&Vec<Packet>>,
    ) {
        if prime { println!("{}':", rd) } else { println!("{}:", rd) };
        println!("{}", network);
    }

    fn close(&mut self) {
        println!("Simulation finished.");
    }
}


const BUFFER_LOAD_CSV_HEADER: &str = "rd,prime,buffer_from,buffer_to,load\n";
/// Records the load in each buffer and saves to a csv.
pub struct BufferLoadRecorder {
    line_limit: usize, // How many lines to keep in memory before dumping record to file. 
    lines: Vec<String>,
    output_path: Option<String>,
    filename: &'static str,
}

impl Recorder for BufferLoadRecorder {
    fn set_output_path(&mut self, output_path: String) {
       self.output_path = Some(output_path); 
    }

    fn record(
        &mut self,
        rd: usize,
        prime: bool,
        network: &Network,
        _absorbed: Option<&Vec<Packet>>,
    ) {
        // format: rd,prime (0/1), buffer_from, buffer_to, load (see BUFFER_LOAD_CSV_HEADER)
        let prime_flag = if prime { 1 } else { 0 };
        for (from_id, to_id) in network.get_edgebuffers() {
            let load = network.get_edgebuffer(from_id, to_id).unwrap().buffer.len();
            self.write(
                format!("{},{},{},{},{}\n", rd, prime_flag, from_id, to_id, load),
                self.filename
            );
        }
    }

    fn close(&mut self) {
        self.save(self.filename);
    }
}

impl SaveRecorder for BufferLoadRecorder {
    fn get_lines(&self) -> &Vec<String> {
        &self.lines
    }

    fn push_line(&mut self, line: String) {
        self.lines.push(line);
    }

    fn reset_lines(&mut self) {
        self.lines = Vec::new();
    }

    fn get_line_limit(&self) -> usize {
        self.line_limit
    }

    fn get_output_path(&self) -> Option<&String> {
        self.output_path.as_ref()
    }
}

impl BufferLoadRecorder {
    /// Get a new `BufferLoadRecorder` with the default line limit.
    pub fn new() -> Self {
        Self::new_with_line_limit(DEFAULT_LINE_LIMIT)
    }

    /// Get a new `BufferLoadRecorder` with the specified line limit.
    pub fn new_with_line_limit(local_line_limit: usize) -> Self {
        BufferLoadRecorder {
            line_limit: local_line_limit,
            lines: vec![BUFFER_LOAD_CSV_HEADER.to_string()],
            output_path: None,
            filename: "buffer_load.csv",
        }
    }
}


const ABSORPTION_CSV_HEADER: &str = "rd,packet_id,packet_injection_rd\n";
/// Records round number, injection round, packet ID of each absorbed `Packet`.
pub struct AbsorptionRecorder {
    line_limit: usize, // How many lines to keep in memory before dumping record to file. 
    lines: Vec<String>,
    output_path: Option<String>,
    filename: &'static str,
}

impl Recorder for AbsorptionRecorder {
    fn record(
        &mut self,
        rd: usize,
        prime: bool,
        _network: &Network,
        absorbed: Option<&Vec<Packet>>,
    ) {
        if !prime { return };
        for packet in absorbed.unwrap() {
            self.write(
                format!("{},{},{}\n", rd, packet.get_id(), packet.get_injection_rd()),
                self.filename
            );
        }
    }

    fn set_output_path(&mut self, output_path: String) {
        self.output_path = Some(output_path);
    }

    fn close(&mut self) {
        self.save(self.filename);
    }
}

impl SaveRecorder for AbsorptionRecorder {
    fn get_lines(&self) -> &Vec<String> {
        &self.lines
    }

    fn push_line(&mut self, line: String) {
        self.lines.push(line);
    }

    fn reset_lines(&mut self) {
        self.lines = Vec::new();
    }

    fn get_line_limit(&self) -> usize {
        self.line_limit
    }

    fn get_output_path(&self) -> Option<&String> {
        self.output_path.as_ref()
    }
}

impl AbsorptionRecorder {
    /// Get a new `AbsorptionLoadRecorder` with the default line limit.
    pub fn new() -> Self {
        Self::new_with_line_limit(DEFAULT_LINE_LIMIT)
    }

    /// Get a new `AbsorptionLoadRecorder` with the specified line limit.
    pub fn new_with_line_limit(local_line_limit: usize) -> Self {
        AbsorptionRecorder {
            line_limit: local_line_limit,
            lines: vec![ABSORPTION_CSV_HEADER.to_string()],
            output_path: None,
            filename: "absorption.csv",
        }
    }
}
