/*
  status bar for tiling wms like i3, sway, etc...
  Copyright (C) 2019 Oleg Keri

  This program is free software: you can redistribute it and/or modify
  it under the terms of the GNU General Public License as published by
  the Free Software Foundation, either version 3 of the License, or
  (at your option) any later version.
  This program is distributed in the hope that it will be useful,
  but WITHOUT ANY WARRANTY; without even the implied warranty of
  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
  GNU General Public License for more details.
  You should have received a copy of the GNU General Public License
  along with this program.  If not, see <http://www.gnu.org/licenses/>
*/

use super::base::{Base, Value, default_zero, default_str_none};
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
            .unwrap_or("error".to_string());

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
			    .expect(&format!("failed to execute {}", action));
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
