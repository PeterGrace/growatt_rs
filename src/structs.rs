use std::str::FromStr;
use serde::{Deserialize, Deserializer};
use tokio_modbus::Address;

#[derive(Debug, Deserialize, Clone)]
pub struct GrowattModel(Config);
impl GrowattModel {
    pub fn get_points(&self) -> &Vec<Point> {
        &self.0.points
    }
    pub fn get_manufacturer(&self) -> &String {
        &self.0.manufacturer
    }
    pub fn get_model(&self) -> &String {
        &self.0.model
    }
    pub fn get_locator(&self, l_type: &str) -> &Locator {
        match l_type {
            "serial_number" => &self.0.serial_number,
            "firmware_version" => &self.0.firmware_version,
            _ => panic!("Unknown locator type")
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum LocatorType {
    String,
    Integer
}
#[derive(Debug, Deserialize, Clone)]
pub struct Locator {
    pub format: LocatorType,
    pub address: u16,
    pub length: usize,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub manufacturer: String,
    pub model: String,
    pub serial_number: Locator,
    pub firmware_version: Locator,
    pub points: Vec<Point>
}
#[derive(Debug, Deserialize, Clone)]
pub struct Point {
    #[serde(deserialize_with = "deserialize_address")]
    pub address: Address,
    pub name: String,
    pub scale_factor: i16,
    pub length: usize,
    pub uom: String,
    pub device_class: String,
    pub state_class: String,
    pub precision: usize
}
impl Point {
    pub fn name(&self) -> String {
        String::from(self.name.clone())
    }
}
fn deserialize_address<'de, D>(deserializer: D) -> Result<u16, D::Error>
where
D: Deserializer<'de>{
    let s = String::deserialize(deserializer)?;
    let n = s.trim_start_matches("0x").to_string();
    let i = u32::from_str_radix(&n, 16).map_err(serde::de::Error::custom)? as u16;
    Ok(i)
}