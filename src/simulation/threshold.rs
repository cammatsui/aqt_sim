//! This module contains the `Threshold` trait and its implementations, which determine when a
//! `Simulation` should stop running.

use crate::config::{CfgErrorMsg, Configurable};
use crate::network::Network;
use serde_json::{Map, Number, Value};

/// Used to end a `Simulation`.
#[derive(Clone)]
pub enum Threshold {
    Timed(TimedThreshold),
    TotalLoad(TotalLoadThreshold),
}

impl Threshold {
    /// Get a `TimedThreshold` with the given max number of rounds.
    pub fn timed_from_rds(max_rds: usize) -> Self {
        Self::Timed(TimedThreshold::new(max_rds))
    }

    /// Check whether the `Simulation` should terminate based on the round number and network
    /// state.
    pub fn check_termination(&mut self, rd: usize, network: &Network) -> bool {
        match self {
            Self::Timed(t) => t.check_termination(rd, network),
            Self::TotalLoad(t) => t.check_termination(rd, network),
        }
    }
}

const THRESHOLD_NAME_KEY: &str = "threshold_name";
const TIMED_THRESHOLD_NAME: &str = "timed";
const TOTAL_LOAD_THRESHOLD_NAME: &str = "total_load";

impl Configurable for Threshold {
    fn from_config(config: Value) -> Result<Self, CfgErrorMsg> {
        // TODO: error msgs
        let map: Map<String, Value> = config.as_object().unwrap().clone();
        let threshold_name = match map.get(THRESHOLD_NAME_KEY) {
            Some(Value::String(name)) => Ok(name),
            _ => Err(String::from("No threshold name found.")),
        }?;

        match &threshold_name[..] {
            TIMED_THRESHOLD_NAME => Ok(Self::Timed(TimedThreshold::from_config(config).unwrap())),
            TOTAL_LOAD_THRESHOLD_NAME => Ok(Self::TotalLoad(TotalLoadThreshold::from_config(config).unwrap())),
            _ => Err(String::from("No threshold name found.")),
        }
    }

    fn to_config(&self) -> Value {
        match self {
            Self::Timed(t) => t.to_config(),
            Self::TotalLoad(t) => t.to_config(),
        }
    }
}

/// Trait which all `Threshold`s should implement.
pub trait ThresholdTrait {
    /// Check whether to terminate the simulation and update any internal state of the
    /// `Threshold.`.
    fn check_termination(&mut self, rd: usize, network: &Network) -> bool;
}

/// To end a `Simulation` after a specified number of rounds has elapsed.
#[derive(Clone)]
pub struct TimedThreshold {
    max_rds: usize,
}

impl TimedThreshold {
    /// Create a new `TimedThreshold` with the given number of maximum rounds.
    fn new(max_rds: usize) -> Self {
        TimedThreshold { max_rds }
    }
}

impl ThresholdTrait for TimedThreshold {
    fn check_termination(&mut self, rd: usize, _network: &Network) -> bool {
        rd >= self.max_rds
    }
}

const MAX_RDS_KEY: &str = "max_rds";

impl Configurable for TimedThreshold {
    fn from_config(config: Value) -> Result<Self, CfgErrorMsg> {
        let map: Map<String, Value> = config.as_object().unwrap().clone();
        let max_rds = match map.get(MAX_RDS_KEY) {
            Some(Value::Number(num)) => Ok(num.as_u64().unwrap() as usize),
            _ => Err("No max rounds found."),
        }?;
        Ok(Self { max_rds })
    }

    fn to_config(&self) -> Value {
        let mut map = Map::new();
        map.insert(
            THRESHOLD_NAME_KEY.to_string(),
            Value::String(TIMED_THRESHOLD_NAME.to_string()),
        );
        map.insert(
            MAX_RDS_KEY.to_string(),
            Value::Number(Number::from(self.max_rds)),
        );

        Value::Object(map)
    }
}

/// To end a `Simulation` after a specified total load has been reached.
#[derive(Clone)]
pub struct TotalLoadThreshold {
    max_load: usize,
}

impl ThresholdTrait for TotalLoadThreshold {
    fn check_termination(&mut self, _rd: usize, network: &Network) -> bool {
        let mut total_load = 0;
        let eb_ids =  network.get_edgebuffers();
        for (from_id, to_id) in eb_ids {
            let eb = network.get_edgebuffer(from_id, to_id).unwrap();
            total_load += eb.buffer.len();
        }
        total_load >= self.max_load
    }
}

const MAX_LOAD_KEY: &str = "max_load";

impl Configurable for TotalLoadThreshold {
    fn from_config(config: Value) -> Result<Self, CfgErrorMsg> {
        let map: Map<String, Value> = config.as_object().unwrap().clone();
        let max_load = match map.get(MAX_LOAD_KEY) {
            Some(Value::Number(num)) => Ok(num.as_u64().unwrap() as usize),
            _ => Err("No max load found."),
        }?;
        Ok(Self { max_load })
    }

    fn to_config(&self) -> Value {
        let mut map = Map::new();
        map.insert(
            THRESHOLD_NAME_KEY.to_string(),
            Value::String(TOTAL_LOAD_THRESHOLD_NAME.to_string()),
        );
        map.insert(
            MAX_LOAD_KEY.to_string(),
            Value::Number(Number::from(self.max_load)),
        );

        Value::Object(map)
    }
}
