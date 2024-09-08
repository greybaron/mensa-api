use crate::types::Mensa;
use std::{
    collections::BTreeMap,
    sync::{LazyLock, OnceLock},
};

pub static MENSEN_MAP: LazyLock<std::sync::RwLock<BTreeMap<u32, String>>> =
    LazyLock::new(|| std::sync::RwLock::new(BTreeMap::new()));
pub static MENSEN_MAP_INV: LazyLock<std::sync::RwLock<BTreeMap<String, u32>>> =
    LazyLock::new(|| std::sync::RwLock::new(BTreeMap::new()));

pub static OPENMENSA_ALL_MENSEN: OnceLock<Vec<Mensa>> = OnceLock::new();
pub static OPENMENSA_LIVE_MENSEN: OnceLock<Vec<Mensa>> = OnceLock::new();
