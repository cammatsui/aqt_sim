use aqt_sim::simulation::Simulation;
use aqt_sim::simulation::SimulationConfig;
use std::env;
use std::fs;
use std::time::Instant;

fn main() {
    // usage: input config file ->
    //let network = presets::construct_path(NUM_BUFFERS);
    //let protocol = Protocol::new_oed_with_swap();
    //let adversary = Adversary::sd_path_random_from_seed(SEED);
    //let threshold = Threshold::timed_from_rds(NUM_RDS);

    //let recorders: Vec<Recorder> = vec![
    //    Recorder::new_debug_print(),
    //    Recorder::file_recorder_from_type(FileRecorderType::BufferLoadCSV),
    //    Recorder::file_recorder_from_type(FileRecorderType::AbsorptionCSV),
    //];
    //let mut simulation = Simulation::new(
    //    network,
    //    protocol,
    //    adversary,
    //    threshold,
    //    recorders,
    //    "/home/cammatsui/Dev/aqt_sim/output".to_string(),
    //);
    //simulation.run();
    let now = Instant::now();


    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("USAGE: aqt_sim <config_filepath>");
    } else {
        let json = fs::read_to_string(&args[1]).unwrap();
        let sim_configs = serde_json::from_str(&json).unwrap();
        run_sims(sim_configs);
    }


    let elapsed = now.elapsed();
    println!("Elapsed: {:.2?}", elapsed);
}

fn run_sims(sim_configs: Vec<SimulationConfig>) {
    for sim_config in sim_configs {
        let mut simulation = Simulation::from_config(sim_config);
        simulation.run();
    }
    /*
    use std::thread;

    let mut handles = Vec::new();
    for sim_config in sim_configs {
        handles.push(thread::spawn(move || {
            let mut simulation = Simulation::from_config(sim_config);
            simulation.run();
        }));
    }

    for handle in handles {
        handle.join().unwrap()
    }
    */

}
