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
use std::collections::HashSet;

#[derive(Deserialize)]
pub struct Block {
    #[serde(flatten)]
    base: Base,
    #[serde(default = "default_ethernet")]
    ethernet: String,
}

fn default_ethernet() -> String {
    "eth".to_string()
}

fn parse_wifi_str(interface: &str, text: String) -> Result<u32, ()> {
    for line in text.split('\n') {
        if line.find(interface).is_some() {
            let linedata: Vec<&str> = line.split_whitespace().collect();
            return linedata[2][0..linedata[2].len() - 1]
                .parse::<u32>()
                .map(|value| value * 100 / 70)
                .map_err(|_| ());
        }
    }
    return Err(());
}

fn get_wifi_str(interface: &str) -> Result<u32, ()> {
    std::fs::read_to_string("/proc/net/wireless")
        .map_err(|_| ())
        .and_then(|text| parse_wifi_str(interface, text))
}

fn get_active_interfaces() -> HashSet<String> {
    let mut result = HashSet::new();
    if let Ok(text) = std::fs::read_to_string("/proc/net/route") {
	for line in text.split('\n').filter(| value | !value.contains("Iface")) {
	    let linedata: Vec<&str> = line.split_whitespace().collect();
	    if let Some(name) = linedata.iter().next() {
		result.insert(name.to_string());
	    }
	}	
    }
    result
}

impl block::Block for Block {
    impl_Block!();
    
    fn update(&mut self) {
	self.base.value = Value::Invalid;
	for iface in get_active_interfaces() {
	    if let Ok(strength) = get_wifi_str(&iface) {
		self.base.value = Value::new(strength);
		self.base.set_ignore_decoration(false);
		return;
	    } else if let Value::Invalid = self.base.value {
		self.base.value = Value::new(self.ethernet.clone());
		self.base.set_ignore_decoration(true);
	    }
	}
    }
}
