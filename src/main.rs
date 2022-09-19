use aqt_sim::protocol::oed::OEDWithSwap;
use aqt_sim::simulation::Simulation;
use aqt_sim::network::presets;
use aqt_sim::protocol::Protocol;
use aqt_sim::adversary::Adversary;
use aqt_sim::simulation::threshold::{ Threshold, TimedThreshold };
use aqt_sim::simulation::recorder::{ Recorder, DebugPrintRecorder, FileRecorder, FileRecorderType };
use aqt_sim::adversary::path_random::SDPathRandomAdversary;

const NUM_BUFFERS: usize = 10;
const NUM_RDS: usize = 10;

fn main() {
    let network = presets::construct_path(NUM_BUFFERS);
    let protocol = Protocol::OEDWithSwap(OEDWithSwap::new());
    let adversary = Adversary::SDPathRandom(SDPathRandomAdversary::new());
    let threshold = Threshold::Timed(TimedThreshold::new(NUM_RDS));

    let recorders: Vec<Recorder> = vec![
        Recorder::DebugPrint(DebugPrintRecorder::new()),
        Recorder::File(FileRecorder::new(FileRecorderType::BufferLoadCSV)),
        Recorder::File(FileRecorder::new(FileRecorderType::AbsorptionCSV)),
    ];
    let mut simulation = Simulation::new(network, protocol, adversary, threshold, recorders, "/home/cammatsui/Dev/aqt_sim/output".to_string());
    simulation.run();
}


