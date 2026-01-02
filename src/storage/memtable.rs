use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Observation {
    pub station_id: String,
    pub time: String,
    pub temp: Option<f64>,
    pub humidity: Option<f64>,
    pub pressure: Option<f64>,
    pub wind_speed: Option<f64>,
    pub wind_dir: Option<u16>,
}

#[derive(Debug, Default)]
pub struct MemTable {
    // keyed by station_id -> vector of observations
    pub buffer: HashMap<String, Vec<Observation>>,
}

impl MemTable {
    pub fn new() -> Self {
        Self { buffer: HashMap::new() }
    }

    pub fn insert(&mut self, obs: Observation) {
        self.buffer.entry(obs.station_id.clone()).or_default().push(obs);
    }

    // Placeholder for flush logic
    pub fn flush(&mut self) {
        self.buffer.clear();
    }
}
