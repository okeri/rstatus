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

use super::base::{Base, Value};
use super::block;
use serde::Deserialize;
use std::collections::HashMap;

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
    statuses: HashMap<String, Status>,
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
        self.base.value = Value::new(value);
    }
}

fn default_statuses() -> HashMap<String, Status> {
    HashMap::new()
}

fn status2rstatus(status: &str) -> &str {
    match status {
        "Charging" => "online",
        "Discharging" => "offline",
        _ => "full",
    }
}
