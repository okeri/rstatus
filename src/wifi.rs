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

#[derive(Deserialize)]
pub struct Block {
    #[serde(flatten)]
    base: Base,
    interface: String,
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

impl block::Block for Block {
    impl_Block!();
    fn update(&mut self) {
        self.base.value = Value::new(get_wifi_str(&self.interface));
    }
}
