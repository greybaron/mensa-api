use crate::types::Canteen;
use std::{
    collections::BTreeMap,
    sync::{LazyLock, OnceLock},
};

pub static CANTEEN_MAP: LazyLock<std::sync::RwLock<BTreeMap<u32, String>>> =
    LazyLock::new(|| std::sync::RwLock::new(BTreeMap::new()));
pub static CANTEEN_MAP_INV: LazyLock<std::sync::RwLock<BTreeMap<String, u32>>> =
    LazyLock::new(|| std::sync::RwLock::new(BTreeMap::new()));

pub static OPENMENSA_ALL_CANTEENS: OnceLock<Vec<Canteen>> = OnceLock::new();
pub static OPENMENSA_LIVE_CANTEENS: OnceLock<Vec<Canteen>> = OnceLock::new();
