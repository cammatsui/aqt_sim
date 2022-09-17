use std::fs;
use std::io::prelude::*;
use crate::network::Network;
use crate::packet::Packet;

// For CSV/file writing, how many lines to keep in memory before writing to disk.
const LINE_LIMIT: usize = 5000;


/// Trait implemented by all recorders.
pub trait Recorder {
    fn record(&mut self, rd: usize, prime: bool, network: &Network, absorbed: Option<&Vec<Packet>>);
    fn set_output_path(&mut self, output_path: String);
    fn close(&mut self);
}


/// Prints the network and any to the console.
pub struct DebugPrintRecorder;

impl DebugPrintRecorder {
    pub fn new() -> Self { DebugPrintRecorder}
}

impl Recorder for DebugPrintRecorder {
    fn record(
        &mut self,
        rd: usize,
        prime: bool,
        network: &Network,
        absorbed: Option<&Vec<Packet>>,
    ) {
        if prime { println!("{}':", rd) } else { println!("{}:", rd) };
        println!("{}", network);
        if let Some(absorbed_packets) = absorbed {
            if absorbed_packets.len() == 0 { return };
            println!("Absorbed Packets:");
            for packet in absorbed_packets {
                println!("{:?}", packet);
            }
        }
        println!();
    }

    fn close(&mut self) {
        println!("Simulation finished.");
    }

    fn set_output_path(&mut self, _output_path: String) { }
}


/// Types of file recorders.
#[derive(Clone, Copy)]
pub enum FileRecorderType {
    AbsorptionCSV,
    BufferLoadCSV,
}


/// Write some aspect of the simulation state to a file.
pub struct FileRecorder {
    recorder_type: FileRecorderType,
    lines: Vec<String>,
    // We require the output dir path to be set; optional so that Simulation::new() caller doesn't
    // have to construct and provide every individual file's output path.
    file_path: Option<String>,
}

impl FileRecorder {
    pub fn new(recorder_type: FileRecorderType) -> Self {
        FileRecorder {
            recorder_type,
            lines: vec![Self::type_to_header(recorder_type).to_string()],
            file_path: None,
        }
    }

    const fn type_to_filename(recorder_type: FileRecorderType) -> &'static str {
        match recorder_type {
            FileRecorderType::AbsorptionCSV => "absorption.csv",
            FileRecorderType::BufferLoadCSV => "buffer_load.csv",
        }
    }

    const fn type_to_header(recorder_type: FileRecorderType) -> &'static str {
        match recorder_type {
            FileRecorderType::AbsorptionCSV => "rd,packet_id,packet_injection_rd\n",
            FileRecorderType::BufferLoadCSV => "rd,prime,buffer_from,buffer_to,load\n",
        }
    }

    /// Write a line to the recorder.
    pub fn write(&mut self, line: String) {
        if self.lines.len() >= LINE_LIMIT {
            self.save();
            self.lines = Vec::new();
        }
        self.lines.push(line);
    }

    /// Save the lines to a file.
    pub fn save(&mut self) {
        let data = self.lines.concat();
        let file_path_unwrapped = self.file_path.as_ref()
            .expect("You must set an output path for each recorder.");

        let mut file = fs::OpenOptions::new()
            .write(true).create(true).append(true).open(&file_path_unwrapped)
            .expect(&format!("Failed to save simulation results to {}", file_path_unwrapped));

        if let Err(_) = writeln!(file, "{}", data) {
            eprintln!("Failed to save simulation results to {}", file_path_unwrapped);
        }
    }
}

impl Recorder for FileRecorder {
    fn close(&mut self) {
        self.save();
    }

    fn set_output_path(&mut self, dir_path: String) {
        fs::create_dir_all(dir_path.clone())
            .expect(&format!("Failed to save simulation results to {}", &dir_path));

        let mut file_path = String::from(dir_path);
        file_path.push('/');
        file_path.push_str(Self::type_to_filename(self.recorder_type));
        self.file_path = Some(file_path);
    }

    fn record(
        &mut self,
        rd: usize,
        prime: bool,
        network: &Network,
        absorbed: Option<&Vec<Packet>>,
    ) {
        match self.recorder_type {
            FileRecorderType::AbsorptionCSV => {
                if !prime { return };
                for packet in absorbed.unwrap() {
                    self.write(format!(
                        "{},{},{}\n",
                        rd,
                        packet.get_id(),
                        packet.get_injection_rd()
                    ));
                }
            },
            FileRecorderType::BufferLoadCSV => {
                let prime_flag = if prime { 1 } else { 0 };
                for (from_id, to_id) in network.get_edgebuffers() {
                    let load = network.get_edgebuffer(from_id, to_id).unwrap().buffer.len();
                    self.write(
                        format!("{},{},{},{},{}\n", rd, prime_flag, from_id, to_id, load),
                    );
                }
            }
        }
    }
}
