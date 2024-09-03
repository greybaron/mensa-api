use std::{collections::BTreeMap, sync::OnceLock};

pub static MENSEN_MAP: OnceLock<BTreeMap<u8, String>> = OnceLock::new();
pub static MENSEN_MAP_INV: OnceLock<BTreeMap<String, u8>> = OnceLock::new();
