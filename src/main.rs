use aqt_sim::protocol::oed::OEDWithSwap;
use aqt_sim::simulation::Simulation;
use aqt_sim::network::presets;
use aqt_sim::protocol::Protocol;
use aqt_sim::adversary::Adversary;
use aqt_sim::simulation::threshold::TimedThreshold;
use aqt_sim::simulation::recorder::{ Recorder, DebugPrintRecorder, FileRecorder, FileRecorderType };
use aqt_sim::adversary::path_random::SDPathRandomAdversary;

const NUM_BUFFERS: usize = 10;
const NUM_RDS: usize = 10;

fn main() {
    let network = presets::construct_path(NUM_BUFFERS);
    let protocol = OEDWithSwap::new(1);
    let adversary = SDPathRandomAdversary::new();
    let threshold = TimedThreshold::new(NUM_RDS);
    let recorders: Vec<Box<dyn Recorder>> = vec![
        Box::new(DebugPrintRecorder::new()),
        Box::new(FileRecorder::new(FileRecorderType::BufferLoadCSV)),
        Box::new(FileRecorder::new(FileRecorderType::AbsorptionCSV)),
    ];
    let mut simulation = Simulation::new(network, protocol, adversary, threshold, recorders, "/home/cammatsui/Dev/aqt_sim/output".to_string());
    simulation.run();
}
