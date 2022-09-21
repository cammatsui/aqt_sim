use aqt_sim::config::Config;
use aqt_sim::simulation::Simulation;
use std::env;
use std::fs;
use std::thread;
use std::time::Instant;

fn main() {
    let now = Instant::now();

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("USAGE: aqt_sim <config_filepath>");
    } else {
        let json = fs::read_to_string(&args[1]).unwrap();
        let config = Config::from_string(json).unwrap();
        run_sims(config);
    }

    let elapsed = now.elapsed();
    println!("Elapsed: {:.2?}", elapsed);
}

fn run_sims(config: Config) {
    if config.parallel {
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
    } else {
        for sim_config in config.sim_configs {
            let mut simulation = Simulation::from_config(sim_config);
            simulation.run();
        }
    }
}
