use aqt_sim::protocol::oed::OEDWithSwap;
use aqt_sim::sim::Simulation;
use aqt_sim::network::presets;
use aqt_sim::protocol::Protocol;
use aqt_sim::protocol::greedy::GreedyFIFO;
use aqt_sim::adversary::Adversary;
use aqt_sim::adversary::path_random::SDPathRandomAdversary;

const NUM_BUFFERS: usize = 10;
const NUM_RDS: usize = 10;

fn main() {
    let network = presets::construct_path(NUM_BUFFERS);
    let protocol = OEDWithSwap::new(1);
    let adversary = SDPathRandomAdversary::new();
    let mut simulation = Simulation::new(network, protocol, adversary, true);
    simulation.run(NUM_RDS);
}
