use crate::config::{CfgErrorMsg, Configurable};
use crate::network::Network;
use crate::packet::Packet;
use serde_json::{Map, Value};
use std::fs;
use std::io::prelude::*;

// For CSV/file writing, how many lines to keep in memory before writing to disk.
const LINE_LIMIT: usize = 5000;

/// Enum for all `Recorder`s.
#[derive(Clone)]
pub enum Recorder {
    DebugPrint(DebugPrintRecorder),
    File(FileRecorder),
}

impl Recorder {
    /// Get a new `FileRecorder` from the given recorder type.
    pub fn file_recorder_from_type(recorder_type: FileRecorderType) -> Self {
        Recorder::File(FileRecorder::new(recorder_type))
    }

    /// Get a new `DebugPrintRecorder`.
    pub fn new_debug_print() -> Self {
        Recorder::DebugPrint(DebugPrintRecorder::new())
    }

    /// Record the state of the `Simulation` via the `RecorderTrait`.
    pub fn record(
        &mut self,
        rd: usize,
        prime: bool,
        network: &Network,
        absorbed: Option<&Vec<Packet>>,
    ) {
        match self {
            Self::DebugPrint(rec) => rec.record(rd, prime, network, absorbed),
            Self::File(rec) => rec.record(rd, prime, network, absorbed),
        }
    }

    /// Set the output path for this `Recorder` via the `RecorderTrait`.
    pub fn set_output_path(&mut self, output_path: String) {
        match self {
            Self::DebugPrint(rec) => rec.set_output_path(output_path),
            Self::File(rec) => rec.set_output_path(output_path),
        }
    }

    /// Close this `Recorder` via the `RecorderTrait`.
    pub fn close(&mut self) {
        match self {
            Self::DebugPrint(rec) => rec.close(),
            Self::File(rec) => rec.close(),
        }
    }
}

const RECORDER_NAME_KEY: &str = "recorder_name";
const DEBUG_PRINT_NAME: &str = "debug_print";
const BUFFER_LOAD_NAME: &str = "buffer_load";
const ABSORPTION_NAME: &str = "absorption";
const SMOOTHED_CONFIG_LIS_NAME: &str = "smoothed_config_lis";

impl Configurable for Recorder {
    fn from_config(config: Value) -> Result<Self, CfgErrorMsg> {
        // TODO: correct error msg
        let map: Map<String, Value> = config.as_object().unwrap().clone();
        let recorder_name = match map.get(RECORDER_NAME_KEY) {
            Some(Value::String(name)) => Ok(name),
            _ => Err(String::from("No protocol name found.")),
        }?;

        match &recorder_name[..] {
            DEBUG_PRINT_NAME => Ok(Self::DebugPrint(DebugPrintRecorder::new())),
            BUFFER_LOAD_NAME => Ok(Self::File(FileRecorder::new(
                FileRecorderType::BufferLoadCSV,
            ))),
            ABSORPTION_NAME => Ok(Self::File(FileRecorder::new(
                FileRecorderType::AbsorptionCSV,
            ))),
            SMOOTHED_CONFIG_LIS_NAME => Ok(Self::File(FileRecorder::new(
                FileRecorderType::SmoothedConfigLISCSV,
            ))),
            _ => Err(format!("No recorder with name {}.", recorder_name)),
        }
    }

    fn to_config(&self) -> Value {
        let mut map: Map<String, Value> = Map::new();
        let key = RECORDER_NAME_KEY.to_string();
        let val = match self {
            Self::DebugPrint(_) => DEBUG_PRINT_NAME.to_string(),
            Self::File(r) => match r.recorder_type {
                FileRecorderType::BufferLoadCSV => BUFFER_LOAD_NAME.to_string(),
                FileRecorderType::AbsorptionCSV => ABSORPTION_NAME.to_string(),
                FileRecorderType::SmoothedConfigLISCSV => SMOOTHED_CONFIG_LIS_NAME.to_string(),
            },
        };
        map.insert(key, Value::String(val));
        Value::Object(map)
    }
}

/// Trait implemented by all recorders.
pub trait RecorderTrait {
    fn record(&mut self, rd: usize, prime: bool, network: &Network, absorbed: Option<&Vec<Packet>>);
    fn set_output_path(&mut self, output_path: String);
    fn close(&mut self);
}

/// Prints the network and any to the console.
#[derive(Clone)]
pub struct DebugPrintRecorder;

impl DebugPrintRecorder {
    fn new() -> Self {
        DebugPrintRecorder
    }
}

impl RecorderTrait for DebugPrintRecorder {
    fn record(
        &mut self,
        rd: usize,
        prime: bool,
        network: &Network,
        absorbed: Option<&Vec<Packet>>,
    ) {
        if prime {
            println!("{}':", rd)
        } else {
            println!("{}:", rd)
        }
        println!("{}", network);
        if let Some(absorbed_packets) = absorbed {
            if absorbed_packets.len() == 0 {
                return;
            }
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

    fn set_output_path(&mut self, _output_path: String) {}
}

/// Types of file recorders.
#[derive(Clone, Copy)]
pub enum FileRecorderType {
    AbsorptionCSV,
    BufferLoadCSV,
    SmoothedConfigLISCSV,
}

/// Write some aspect of the simulation state to a file.
#[derive(Clone)]
pub struct FileRecorder {
    recorder_type: FileRecorderType,
    lines: Vec<String>,
    // We require the output dir path to be set; optional so that Simulation::new() caller doesn't
    // have to construct and provide every individual file's output path.
    file_path: Option<String>,
}

impl FileRecorder {
    /// Get a new `FileRecorder` of the given type.
    fn new(recorder_type: FileRecorderType) -> Self {
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
            FileRecorderType::SmoothedConfigLISCSV => "smoothed_config_lis.csv",
        }
    }

    const fn type_to_header(recorder_type: FileRecorderType) -> &'static str {
        match recorder_type {
            FileRecorderType::AbsorptionCSV => "rd,packet_id,packet_injection_rd\n",
            FileRecorderType::BufferLoadCSV => "rd,prime,buffer_from,buffer_to,load\n",
            FileRecorderType::SmoothedConfigLISCSV => {
                "rd,prime,buffer_from,buffer_to,packet_id,injection_rd\n"
            }
        }
    }

    /// Write a line to the recorder.
    fn write(&mut self, line: String) {
        if self.lines.len() >= LINE_LIMIT {
            self.save();
            self.lines = Vec::new();
        }
        self.lines.push(line);
    }

    /// Save the lines to a file.
    fn save(&mut self) {
        let data = self.lines.concat();
        let file_path_unwrapped = self
            .file_path
            .as_ref()
            .expect("You must set an output path for each recorder.");

        let mut file = fs::OpenOptions::new()
            .write(true)
            .create(true)
            .append(true)
            .open(&file_path_unwrapped)
            .expect(&format!(
                "Failed to save simulation results to {}",
                file_path_unwrapped
            ));

        if let Err(_) = writeln!(file, "{}", data) {
            eprintln!(
                "Failed to save simulation results to {}",
                file_path_unwrapped
            );
        }
    }
}

impl RecorderTrait for FileRecorder {
    fn close(&mut self) {
        self.save();
    }

    fn set_output_path(&mut self, dir_path: String) {
        fs::create_dir_all(dir_path.clone()).expect(&format!(
            "Failed to save simulation results to {}",
            &dir_path
        ));

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
                if !prime {
                    return;
                }
                for packet in absorbed.unwrap() {
                    self.write(format!(
                        "{},{},{}\n",
                        rd,
                        packet.id(),
                        packet.injection_rd()
                    ));
                }
            }
            FileRecorderType::BufferLoadCSV => {
                let prime_flag = if prime { 1 } else { 0 };
                for (from_id, to_id) in network.get_edgebuffers() {
                    let load = network.get_edgebuffer(from_id, to_id).unwrap().buffer.len();
                    self.write(format!(
                        "{},{},{},{},{}\n",
                        rd, prime_flag, from_id, to_id, load
                    ));
                }
            }
            FileRecorderType::SmoothedConfigLISCSV => {
                self.write_smoothed_config_lis_lines(rd, prime, network);
            }
        }
    }
}

impl FileRecorder {
    fn write_smoothed_config_lis_lines(&mut self, rd: usize, prime: bool, network: &Network) {
        let prime_flag = if prime { 1 } else { 0 };
        let edgebuffers_ids = network.get_edgebuffers();
        let mut smoothing_queue: Vec<&Packet> = Vec::new();
        for eb_ids in edgebuffers_ids.into_iter().rev() {
            let buffer = &network.get_edgebuffer(eb_ids.0, eb_ids.1).unwrap().buffer;
            for packet in buffer {
                smoothing_queue.push(packet);
            }

            match Self::pop_oldest_packet(&mut smoothing_queue) {
                None => self.write(format!(
                    "{},{},{},{},{},{}\n",
                    rd, prime_flag, eb_ids.0, eb_ids.1, -1, -1
                )),
                Some(oldest) => self.write(format!(
                    "{},{},{},{},{},{}\n",
                    rd,
                    prime_flag,
                    eb_ids.0,
                    eb_ids.1,
                    oldest.id(),
                    oldest.injection_rd(),
                )),
            }
        }
        // "Negative buffers" for packets remaining in the smoothing queue.
        let mut negative_buffer_to: i64 = 0;
        while let Some(oldest) = Self::pop_oldest_packet(&mut smoothing_queue) {
            self.write(format!(
                "{},{},{},{},{},{}\n",
                rd,
                prime_flag,
                negative_buffer_to,
                negative_buffer_to - 1,
                oldest.id(),
                oldest.injection_rd(),
            ));
            negative_buffer_to -= 1;
        }
    }

    fn pop_oldest_packet<'a>(queue: &'a mut Vec<&Packet>) -> Option<&'a Packet> {
        if queue.len() == 0 {
            return None;
        }
        let mut min_injection_rd = usize::MAX;
        let mut min_injection_idx = 0;
        for i in 0..queue.len() {
            let p = queue[i];
            if p.injection_rd() < min_injection_rd {
                min_injection_rd = p.injection_rd();
                min_injection_idx = i;
            }
        }
        Some(queue.remove(min_injection_idx))
    }
}
