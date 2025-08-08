use crate::structs::{GrowattModel, Point};
use chrono::DateTime;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct DeviceInfo {
    pub identifiers: Vec<String>,
    pub manufacturer: String,
    pub name: String,
    pub model: String,
    pub sw_version: String,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ValueType {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Array(Vec<String>),
    Pad,
}

#[derive(Deserialize, Serialize, Debug, Default, Clone)]
#[serde(untagged)]
pub enum PayloadValueType {
    Float(f64),
    Int(i64),
    String(String),
    Boolean(bool),
    #[default]
    None,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Payload {
    Config(HAConfigPayload),
    CurrentState(StatePayload),
    #[default]
    None,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EntityCategory {
    Config,
    #[default]
    Diagnostic,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct HAConfigPayload {
    pub name: String,
    pub device: DeviceInfo,
    pub unique_id: String,
    pub entity_id: String,
    pub state_topic: String,
    pub expires_after: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entity_category: Option<EntityCategory>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command_topic: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payload_on: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payload_off: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state_class: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device_class: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "unit_of_measurement")]
    pub native_uom: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value_template: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suggested_display_precision: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assumed_state: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attribution: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub available: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entity_picture: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra_state_attributes: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_entity_name: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub should_poll: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub translation_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payload_press: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub step: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatePayload {
    pub value: PayloadValueType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    #[serde(with = "crate::date_serializer")]
    pub last_seen: DateTime<Utc>,
}

impl Default for StatePayload {
    fn default() -> Self {
        StatePayload {
            value: PayloadValueType::None,
            last_seen: Utc::now(),
            description: None,
            label: None,
            notes: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CompoundPayload {
    pub(crate) config: HAConfigPayload,
    pub(crate) config_topic: String,
    pub(crate) state: StatePayload,
    pub(crate) state_topic: String,
}

pub async fn generate_payloads(device: &DeviceInfo, sn: String, p: &Point, val: ValueType) -> Vec<CompoundPayload> {
    let mut config_payload: HAConfigPayload = HAConfigPayload::default();
    let mut state_payload: StatePayload = StatePayload::default();

    config_payload.unique_id = format!("{}_{}_{}_{}", device.manufacturer, device.model, sn, p.name());
    config_payload.device = device.clone();
    config_payload.name = p.name.clone();
    config_payload.device_class = Some(p.device_class.clone());
    config_payload.state_class = Some(p.state_class.clone());
    config_payload.suggested_display_precision = Some(p.precision as u8);
    config_payload.expires_after = 300;
    config_payload.value_template = Some("{{ value_json.value }}".to_string());
    match val {
        ValueType::String(str) => {
            state_payload.value = PayloadValueType::String(str.to_owned());
        }
        ValueType::Integer(int) => {
            state_payload.value = PayloadValueType::Int(int);
        }
        ValueType::Float(float) => {
            state_payload.value = PayloadValueType::Float(float);
            config_payload.native_uom = Some(p.uom.clone());
        }
        ValueType::Boolean(boolean) => {
            state_payload.value = PayloadValueType::Boolean(boolean);
        }
        ValueType::Array(vec) => {
            state_payload.value = PayloadValueType::String(vec.join(","));
        }
        ValueType::Pad => {}
    }

    let config_topic = format!("homeassistant/sensor/{sn}/{}/config", &p.name());
    let state_topic = format!("growatt/{sn}/{}",&p.name());

    config_payload.state_topic = state_topic.clone();

    let resp = CompoundPayload {
        config: config_payload,
        state: state_payload,
        config_topic,
        state_topic,
    };
    vec![resp]
}
