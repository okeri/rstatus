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
    mixer: String,
    #[serde(default = "empty_extras")]
    prefix_extras: Vec<String>,
}

enum MixerValue {
    Off,
    Volume(u32),
}

fn get_mixer_value(mixer: &str) -> MixerValue {
    use std::process::Command;
    use std::str;

    let output = str::from_utf8(
        &Command::new("amixer")
            .arg("get")
            .arg(&mixer)
            .output()
            .expect("failed to find amixer")
            .stdout,
    )
    .expect("process amixer returned bad output")
    .to_string();
    let data: Vec<&str> = output.split_whitespace().collect();
    return if data.len() == 0 || data[data.len() - 1] != "[on]" {
        MixerValue::Off
    } else {
        MixerValue::Volume(
            data[data.len() - 3][1..data[data.len() - 3].len() - 2]
                .parse::<u32>()
                .expect("amixer output parse error"),
        )
    };
}

fn empty_extras() -> Vec<String> {
    Vec::new()
}

impl block::Block for Block {
    impl_Block!();
    fn update(&mut self) {
        self.base.value = if let MixerValue::Volume(volume) = get_mixer_value(&self.mixer) {
            if self.prefix_extras.len() > 1 {
                if let MixerValue::Off = get_mixer_value("Speaker") {
                    if let MixerValue::Volume(_) = get_mixer_value("Headphone") {
                        self.base.set_prefix(&self.prefix_extras[0]);
                    } else {
                        self.base.set_prefix(&self.prefix_extras[1]);
                    }
                } else {
                    self.base.set_prefix(&self.prefix_extras[1]);
                }
            }
            Value::new(volume)
        } else {
            Value::Invalid
        }
    }
}
