use std::str::FromStr;
use serde::{Deserialize, Deserializer};
use tokio_modbus::Address;

#[derive(Debug, Deserialize, Clone)]
pub struct GrowattModel(Vec<Point>);
impl GrowattModel {
    pub fn get_points(&self) -> &Vec<Point> {
        &self.0
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct Point {
    #[serde(deserialize_with = "deserialize_address")]
    pub address: Address,
    pub name: String,
    pub scale_factor: i16,
    pub length: usize,
    pub uom: String,
}

fn deserialize_address<'de, D>(deserializer: D) -> Result<u16, D::Error>
where
D: Deserializer<'de>{
    let s = String::deserialize(deserializer)?;
    let n = s.trim_start_matches("0x").to_string();
    let i = u32::from_str_radix(&n, 16).map_err(serde::de::Error::custom)? as u16;
    Ok(i)
}