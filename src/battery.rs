use super::base::{default_str_none, default_zero, Base, Status, Value};
use super::block;
use serde::Deserialize;
use std::collections::BTreeMap;

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

impl Block {
    fn read_u64(&self, name: &str) -> Result<u64, ()> {
        std::fs::read_to_string(self.sensor.clone() + "/" + name)
            .map_err(|_| ())
            .and_then(|text| text.trim().parse::<u64>().map_err(|_| ()))
    }

    fn read_capacity(&self) -> Result<u32, ()> {
        if let Ok(capacity) = self.read_u64("capacity") {
            return Ok(capacity.min(100) as u32);
        }

        for (now, full) in [("energy_now", "energy_full"), ("charge_now", "charge_full")] {
            if let (Ok(now), Ok(full)) = (self.read_u64(now), self.read_u64(full)) {
                if full > 0 {
                    return Ok((((now * 100 + full / 2) / full).min(100)) as u32);
                }
            }
        }

        Err(())
    }
}

impl block::Block for Block {
    impl_Block!();

    fn update(&mut self) {
        let status_raw = std::fs::read_to_string(self.sensor.clone() + "/status")
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

        let value = self.read_capacity();

        if let Some(ref action) = self.warning_action {
            if status == "Discharging" {
                if let Ok(v) = value {
                    if v < self.warning_level {
                        let _ = std::process::Command::new("sh")
                            .arg("-c")
                            .arg(action)
                            .status();
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
