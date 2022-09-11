use crate::network::Network;
use crate::protocol::Protocol;
use crate::adversary::Adversary;


/// Stores all data related to a run of a simulation, including the `Network`, `Protocol`, and
/// `Adversary`.
pub struct Simulation<P: Protocol, A: Adversary> {
    network: Network,
    protocol: P,
    adversary: A,
    debug: bool,
}

impl<P: Protocol, A: Adversary> Simulation<P, A> {
    /// Create a new `Simulation`.
    pub fn new(network: Network, protocol: P, adversary: A, debug: bool) -> Self {
        Simulation { network, protocol, adversary, debug }
    }

    /// Run the simulation for the given number of rounds.
    pub fn run(&mut self, rds: usize) {
        for rd in 0..rds {
            // Injection.
            let mut packets_to_inject = self.adversary.get_next_packets(&self.network, rd);
            let num_to_inject = packets_to_inject.len();
            for _ in 0..num_to_inject {
                let p = packets_to_inject.remove(0);
                self.protocol.add_packet(p, &mut self.network);
            }
            if self.debug { self.debug_print_network(false, rd); }

            // Forwarding.
            self.protocol.forward_packets(&mut self.network);
            if self.debug { self.debug_print_network(true, rd); }
        }
    }

    fn debug_print_network(&self, prime_timestep: bool, rd: usize) {
        if prime_timestep { println!("{}':", rd) } else { println!("{}:", rd) };
        println!("{}", self.network);
    }
}
