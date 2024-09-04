use std::{collections::BTreeMap, sync::OnceLock};

use crate::types::Mensa;

pub static MENSEN_MAP: OnceLock<BTreeMap<u32, String>> = OnceLock::new();
pub static MENSEN_MAP_INV: OnceLock<BTreeMap<String, u32>> = OnceLock::new();

pub static OPENMENSA_ALL_MENSEN: OnceLock<Vec<Mensa>> = OnceLock::new();
pub static OPENMENSA_LIVE_MENSEN: OnceLock<Vec<Mensa>> = OnceLock::new();
