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

use super::base::{Base, Value, default_zero};
use super::block;
use serde::{Deserialize, Deserializer, de::Error};
use std::collections::BTreeMap;

#[derive(Deserialize)]
pub struct Status {
    #[serde(default)]
    pub prefix: String,
    #[serde(default)]
    pub suffix: String,
}

pub enum WarningAction {
    DoNothing,
    Shutdown,
    Suspend,
    Hibernate
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
    #[serde(default = "default_action", deserialize_with = "parse_action")]
    warning_action: WarningAction,
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

	let action_available = || {
	    match self.warning_action {
		WarningAction::DoNothing =>
		    false,
		_ => true,
	    }
	};

	let do_action = |action: &str| {
            std::process::Command::new("sh")
		.arg("-c systemctl")
		.arg(action)
		.status()
		.expect("failed to execute systemctl");
	};
	
	if action_available() && status == "Discharging" {
	    if let Ok(v) = value {
		if v < self.warning_level {
		    match self.warning_action {
			WarningAction::Shutdown => {
			    do_action("halt");
			},
			WarningAction::Suspend => {
			    do_action("suspend");
			},
			WarningAction::Hibernate => {
			    do_action("hibernate");
			},
			_ => {},
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


pub fn default_action() -> WarningAction {
    WarningAction::DoNothing
}


fn parse_action<'de, D>(deserializer: D) -> Result<WarningAction, D::Error>
where
    D: Deserializer<'de>,
{
    let mut result: Result<WarningAction, D::Error> = Ok(WarningAction::DoNothing);
    let s: Option<String> = Option::deserialize(deserializer)?;
    if let Some(text) = s {
	let s_action = text.to_lowercase();
	result = match s_action.as_str() {
	    "donothing" => Ok(WarningAction::DoNothing),
	    "shutdown" => Ok(WarningAction::Shutdown),
	    "suspend" => Ok(WarningAction::Suspend),
	    "hibernate" => Ok(WarningAction::Hibernate),
	    _ => Err(D::Error::custom("unknown warning_action value")),
	};
    }
    result
}
