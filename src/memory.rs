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
}

fn get_mem_usage() -> Result<u32, ()> {
    std::fs::read_to_string("/proc/meminfo")
        .map_err(|_| ())
        .and_then(|text| {
            let values: Vec<&str> = text.split_whitespace().collect();
            let total = values[1].trim().parse::<f64>().map_err(|_| ())?;
            let free = values[7].trim().parse::<f64>().map_err(|_| ())?;
            let value = (1f64 - free / total) * 100f64;
            Ok(value as u32)
        })
}

impl block::Block for Block {
    impl_Block!();
    fn update(&mut self) {
        self.base.value = Value::new(get_mem_usage());
    }
}
