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

pub struct Status {
    pub prefix: String,
    pub suffix: String,
}

pub struct Block {
    base: block::Base,
    sensor: String,
    statuses: [Status; 3],
}

impl Block {
    pub fn new(base: block::Base, sensor: String, statuses: [Status; 3]) -> Block {
        Block {
            base: base,
            sensor: sensor,
            statuses: statuses,
        }
    }
}


impl block::Block for Block {
    impl_Block!();
    fn update(&mut self) {
        use utility;
        let error = "error".to_string();
        let readed_status = utility::read_file(&(self.sensor.clone() + "/status"));
        let status = readed_status.as_ref().unwrap_or(&error).trim();

        let index = match status.as_ref() {
            "error" => {
                self.base.data = block::Value::None;
                return;
            }
            "Discharging" => 0,
            "Charging" => 1,
            _ => 2,
        };

        let value = utility::read_file(&(self.sensor.clone() + "/capacity"))
            .as_ref()
            .unwrap_or(&error)
            .trim()
            .parse::<u32>()
            .unwrap_or(1000);

        self.base.data = if value != 1000 {
            block::Value::new((
                format!(
                    "{}{}{}",
                    self.statuses[index].prefix,
                    value,
                    self.statuses[index].suffix
                ),
                self.base.get_color(value),
            ))
        } else {
            block::Value::None
        }
    }
}
