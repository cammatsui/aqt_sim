use aqt_sim::adversary::Adversary;
use aqt_sim::network::presets;
use aqt_sim::protocol::Protocol;
use aqt_sim::simulation::recorder::{FileRecorderType, Recorder};
use aqt_sim::simulation::threshold::Threshold;
use aqt_sim::simulation::Simulation;

const NUM_BUFFERS: usize = 10;
const NUM_RDS: usize = 10;
const SEED: u64 = 32;

fn main() {
    let network = presets::construct_path(NUM_BUFFERS);
    let protocol = Protocol::new_oed_with_swap();
    let adversary = Adversary::sd_path_random_from_seed(SEED);
    let threshold = Threshold::timed_from_rds(NUM_RDS);

    let recorders: Vec<Recorder> = vec![
        Recorder::new_debug_print(),
        Recorder::file_recorder_from_type(FileRecorderType::BufferLoadCSV),
        Recorder::file_recorder_from_type(FileRecorderType::AbsorptionCSV),
    ];
    let mut simulation = Simulation::new(
        network,
        protocol,
        adversary,
        threshold,
        recorders,
        "/home/cammatsui/Dev/aqt_sim/output".to_string(),
    );
    simulation.run();
}
