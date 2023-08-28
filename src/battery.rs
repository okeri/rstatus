use super::base::{default_str_none, default_zero, Base, Value};
use super::block;
use serde::Deserialize;
use std::collections::BTreeMap;

#[derive(Deserialize)]
pub struct Status {
    #[serde(default)]
    pub prefix: String,
    #[serde(default)]
    pub suffix: String,
}

#[derive(Deserialize)]
pub struct Block {
    #[serde(flatten)]
    base: Base,
    sensor: String,
    #[serde(default = "default_statuses")]
    statuses: BTreeMap<String, Status>,
    #[serde(default = "default_zero")]
    warning_level: u32,
    #[serde(default = "default_str_none")]
    warning_action: Option<String>,
}

impl block::Block for Block {
    impl_Block!();

    fn update(&mut self) {
        let status_raw = std::fs::read_to_string(&(self.sensor.clone() + "/status"))
            .unwrap_or_else(|_| "error".to_string());

        let status = status_raw.trim();
        if status == "error" {
            self.base.value = Value::Invalid;
            return;
        }

        let decoration = self.statuses.get(status2rstatus(status));
        if let Some(d) = decoration {
            self.base.set_prefix(&d.prefix);
            self.base.set_suffix(&d.suffix);
        }

        let value = std::fs::read_to_string(&(self.sensor.clone() + "/capacity"))
            .map_err(|_| ())
            .and_then(|text| text.trim().parse::<u32>().map_err(|_| ()));

        if let Some(ref action) = self.warning_action {
            if status == "Discharging" {
                if let Ok(v) = value {
                    if v < self.warning_level {
                        std::process::Command::new("sh")
                            .arg("-c")
                            .arg(action)
                            .status()
                            .unwrap_or_else(|_| panic!("failed to execute {}", action));
                    }
                }
            }
        }
        self.base.value = Value::new(value);
    }
}

fn default_statuses() -> BTreeMap<String, Status> {
    BTreeMap::new()
}

fn status2rstatus(status: &str) -> &str {
    match status {
        "Charging" => "online",
        "Discharging" => "offline",
        _ => "full",
    }
}
