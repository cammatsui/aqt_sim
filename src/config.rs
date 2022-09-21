use serde_json::{Map, Value};

/// String containing a configuration error message.
pub type CfgErrorMsg = String;

/// Trait which structs which we can dump to/load from a json config.
pub trait Configurable {
    fn from_config(config: Value) -> Result<Self, CfgErrorMsg>
    where
        Self: Sized;
    fn to_config(&self) -> Value;
}

/// Configuration for a `Simulation` run.
pub struct SimConfig {
    pub graph_adjacency: Value,
    pub protocol_cfg: Value,
    pub adversary_cfg: Value,
    pub threshold_cfg: Value,
    pub recorder_cfgs: Value,
    pub output_path: String,
}

pub const ADJACENCY_KEY: &str = "graph_adjacency";
pub const PROTOCOL_KEY: &str = "protocol";
pub const ADVERSARY_KEY: &str = "adversary";
pub const THRESHOLD_KEY: &str = "threshold";
pub const RECORDERS_KEY: &str = "recorders";
pub const OUTPUT_PATH_KEY: &str = "output_path";

impl SimConfig {
    fn get_key(
        obj: &mut Map<String, Value>,
        key: &str,
        err_msg: &str,
    ) -> Result<Value, CfgErrorMsg> {
        match obj.remove(key) {
            Some(val) => Ok(val),
            None => Err(String::from(err_msg)),
        }
    }

    /// Get a new `SimConfig` from the given `serde_json::Value`.
    pub fn from_val(config: Value) -> Result<Self, CfgErrorMsg> {
        let mut obj = match config {
            Value::Object(map) => map,
            _ => return Err(String::from("Simulation config must be a json object.")),
        };

        let graph_adjacency = Self::get_key(&mut obj, ADJACENCY_KEY, "No graph adjacency found.")?;
        let protocol_cfg =
            Self::get_key(&mut obj, PROTOCOL_KEY, "No protocol configuration found.")?;
        let adversary_cfg = Self::get_key(&mut obj, ADVERSARY_KEY, "No adversary config found.")?;
        let threshold_cfg = Self::get_key(&mut obj, THRESHOLD_KEY, "No threshold config found.")?;
        let recorder_cfgs = Self::get_key(&mut obj, RECORDERS_KEY, "No recorder configs found.")?;
        let output_path = match obj.remove(OUTPUT_PATH_KEY) {
            Some(Value::String(path)) => Ok(path),
            _ => Err("No output path string found."),
        }?;

        Ok(Self {
            graph_adjacency,
            protocol_cfg,
            adversary_cfg,
            threshold_cfg,
            recorder_cfgs,
            output_path,
        })
    }

    /// Dump this `SimConfig` to a `serde_json::Value`.
    pub fn to_val(&self) -> Value {
        let mut map = Map::new();
        map.insert(ADJACENCY_KEY.to_string(), self.graph_adjacency.clone());
        map.insert(PROTOCOL_KEY.to_string(), self.protocol_cfg.clone());
        map.insert(ADVERSARY_KEY.to_string(), self.adversary_cfg.clone());
        map.insert(THRESHOLD_KEY.to_string(), self.threshold_cfg.clone());
        map.insert(RECORDERS_KEY.to_string(), self.recorder_cfgs.clone());
        map.insert(
            OUTPUT_PATH_KEY.to_string(),
            Value::String(self.output_path.clone()),
        );
        Value::Object(map)
    }
}

/// Config for the entire program.
pub struct Config {
    pub sim_configs: Vec<SimConfig>,
    pub parallel: bool,
}

const SIMS_KEY: &str = "simulations";
const PARALLEL_KEY: &str = "parallel";

impl Config {
    /// Parse a json string into a `Config`.
    pub fn from_string(data: String) -> Result<Self, CfgErrorMsg> {
        let parsed: Value = serde_json::from_str(&data).unwrap();
        // TODO: Correct error message here
        let mut map: Map<String, Value> = parsed.as_object().unwrap().clone();

        let parallel = match map.remove(PARALLEL_KEY) {
            Some(Value::Bool(parallel_bool)) => Ok(parallel_bool),
            _ => Err(String::from(
                "Must provide \"parallel\" boolean field in config.",
            )),
        }?;

        let sim_cfgs: Vec<SimConfig> = match map.get(SIMS_KEY) {
            Some(Value::Array(cfgs)) => Ok(cfgs
                .iter()
                .map(|x| SimConfig::from_val(x.clone()).unwrap())
                .collect()),
            _ => Err(String::from(
                "Must provide \"parallel\" boolean field in config.",
            )),
        }?;

        Ok(Self { sim_configs: sim_cfgs, parallel })
    }

    /// Dump this `Config` into a json string.
    pub fn to_string(&self) -> String {
        let mut map = Map::new();
        map.insert(PARALLEL_KEY.to_string(), Value::Bool(self.parallel));
        let mut sims_arr = Vec::new();
        for sim_cfg in &self.sim_configs {
            sims_arr.push(sim_cfg.to_val())
        }
        map.insert(SIMS_KEY.to_string(), Value::Array(sims_arr));
        let obj = Value::Object(map);
        serde_json::to_string(&obj).unwrap()
    }
}
