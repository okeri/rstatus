/*
  status bar for i3like wms like i3, sway, etc...
  Copyright (C) 2017 Oleg Keri

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

use block;

pub struct Block {
    base: block::Base,
    mixer: String,
    use_extra: bool,
    prefix_extra: Vec<String>,
}

impl Block {
    pub fn new(base: block::Base, mixer: String, prefix_extra: Vec<String>) -> Block {
        Block {
            base: base,
            mixer: mixer,
            use_extra: prefix_extra.len() > 1,
            prefix_extra: prefix_extra,
        }
    }
}

enum MixerValue {
    Off,
    Volume(u32),
}

fn get_mixer_value(mixer: &str) -> MixerValue {
    use std::str;
    use std::process::Command;
    let output = str::from_utf8(
        &Command::new("amixer")
            .arg("get")
            .arg(&mixer)
            .output()
            .expect("failed to find amixer")
            .stdout,
    ).expect("process amixer returned bad output")
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

impl block::Block for Block {
    impl_Block!();
    fn update(&mut self) {
        self.base.data = if let MixerValue::Volume(volume) = get_mixer_value(&self.mixer) {
            if self.use_extra {
                if let MixerValue::Off = get_mixer_value("Speaker") {
                    if let MixerValue::Volume(_) = get_mixer_value("Headphone") {
                        self.base.prefix = self.prefix_extra[0].clone()
                    } else {
                        self.base.prefix = self.prefix_extra[1].clone()
                    }
                } else {
                    self.base.prefix = self.prefix_extra[1].clone()
                }
            }
            block::Value::new((volume, self.base.get_color(volume)))
        } else {
            block::Value::None
        }
    }
}
