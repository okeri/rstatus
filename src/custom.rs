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
    command: String,
    #[serde(default)]
    retry: u32,
}

impl block::Block for Block {
    impl_Block!();

    fn update(&mut self) {
        use std::process::Command;
        use std::str;
        let output = Command::new("sh")
            .arg("-c")
            .arg(&self.command)
            .output()
            .expect(&format!("failed to execute command '{}'", self.command));

        let strval = str::from_utf8(&output.stdout)
            .expect("custom process returned bad output")
            .to_string();

        if !output.status.success() {
            self.base.set_retry(self.retry);
            return;
        }

        let data: Vec<&str> = strval.split('\n').collect();
        self.base.value = match data.len() {
            0 => Value::Invalid,
            1 => {
                let value = data[0].parse::<u32>();
                if value.is_ok() {
                    Value::new(value.unwrap())
                } else {
                    Value::new(data[0])
                }
            }
            _ => {
                self.base.set_color(read_color(data[1], 0xffffff));
                Value::new(data[0])
            }
        }
    }
}

pub fn read_color(input: &str, default: u32) -> u32 {
    if let Some(first) = input.chars().next() {
        return if first == '#' {
            i64::from_str_radix(&input[1..], 16).unwrap_or(default as i64) as u32
        } else {
            i64::from_str_radix(input, 16).unwrap_or(default as i64) as u32
        };
    }
    default
}
