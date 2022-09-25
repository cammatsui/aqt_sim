# Adversarial Queueing Theory (AQT) Simulator

This program runs simulations of the adversarial queueing theory (AQT) model, first introduced by
[Borodin et al](https://www.cs.cornell.edu/home/kleinber/stoc96-aqt.pdf) in 2001, and was created
for my thesis at Amherst College with Professor Will Rosenbaum in the Computer Science Department.
The currently supported adversaries and protocols focus on path networks.

## Usage

Call either `cargo run <path_to_config_json>` or `./aqt_sim <path_to_config_json>`, depending on
whether you have built the program or not. See `config.json` in the root directory of this repo
for an example. The config allows you to specify:
1. Whether to run the given simulations in parallel, and
2. The simulations (each specified by a network structure, adversary, protocol, threshold, output
path, and set of recorders) to run.

Running the program will create a new directory at the specified output path with the simulation's
configuration (`sim_config.json`), and `csv` files for each recorder which saves to a file.

The config format also allows for comments with `//`, but not inline comments.

## Supported Adversaries
- Single destination path random adversary: `"sd_path_random"` in `config.json`: randomly injects
one packet per round on a single destination path network.
- Single destination path random bursty adversary: `"sd_path_random_bursty"` in `config.json`:
a random (1, `sigma`) adversary. Keeps track of `xi` and injects a random number of packets 
(between 0 and `sigma-xi+1`) with random sources.

## Supported Protocols
- Odd-even-downhill with swap: `"oed_swap"` in `config.json`,
- Greedy FIFO: `"greedy_fifo"` in `config.json`,
- Greedy LIS (longest-in-system): `"greedy_lis"` in `config.json`.

## Supported Recorders
- Debug print: `"debug_print"` in `config.json`. Prints each buffer's packet set at each
timestep.
- Buffer load: `"buffer_load"` in `config.json`. Saves the load of each buffer at each timestep
to `output_path/buffer_load.csv`.
- Absorption: `"absorption"` in `config.json`. Records each absorbed packet with the round number
of the absorption, the absorbed packet's id, and the absorbed packet's injection round to
`output_path/absorption.csv`.
- Smoothed configuration LIS recorder: `"smoothed_config_lis"` in `config.json`. Recorders the 
smoothed configuration of the network at each timestep.

## Supported Thresholds
- Timed: `"timed"` in `config.json`. Stops the simulation after the specified number of rounds.

