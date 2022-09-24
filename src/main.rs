use aqt_sim::config::Config;
use aqt_sim::simulation::Simulation;
use std::env;
use std::fs;
use std::thread;
use std::time::Instant;

const USAGE_MSG: &str = "USAGE: aqt_sim <config_filepath>";

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("{}", USAGE_MSG);
    } else {
        let now = Instant::now();
        let json = fs::read_to_string(&args[1]).unwrap();
        let config = Config::from_string(json).unwrap();
        if config.parallel {
            run_parallel(config)
        } else {
            run_sequential(config)
        }
        let elapsed = now.elapsed();
        println!("Elapsed: {:.2?}", elapsed);
    }
}

fn run_parallel(config: Config) {
    let mut handles = Vec::new();
    for sim_config in config.sim_configs {
        handles.push(thread::spawn(move || {
            let mut simulation = Simulation::from_config(sim_config);
            simulation.run();
        }));
    }

    for handle in handles {
        handle.join().unwrap()
    }
}

fn run_sequential(config: Config) {
    for sim_config in config.sim_configs {
        let mut simulation = Simulation::from_config(sim_config);
        simulation.run();
    }
}
