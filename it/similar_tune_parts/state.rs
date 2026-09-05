//! The run's accumulating state, a leaf both the driver (mod.rs) and the
//! writer (output.rs) import — neither depends on the other, so the
//! module graph stays acyclic (check axis 6 charges every module on a cycle).
use super::config::Config;
use super::data;
use super::metrics;
use super::roles;
use std::{collections::BTreeMap, path::PathBuf};

#[derive(Default, serde::Serialize)]
pub struct Cost {
    pub pool_us: u128,
    pub retrieval_us: u128,
    pub novel: BTreeMap<String, usize>,
    pub stale: BTreeMap<String, usize>,
}

pub struct Run {
    pub configs: Vec<Config>,
    pub outcomes: Vec<Vec<metrics::Outcome>>,
    pub costs: Vec<Cost>,
    pub coverage: Vec<data::Coverage>,
    pub measured: Vec<serde_json::Value>,
    pub pairs: Vec<roles::Pair>,
    pub directory: PathBuf,
}
