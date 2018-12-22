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
use utility;

pub struct Block {
    base: block::Base,
    command: String,
    retry: u32,
}

impl Block {
    pub fn new(base: block::Base, command: String, retry: u32) -> Block {
        Block {
            base: base,
            command: command,
            retry: retry,
        }
    }
}

impl block::Block for Block {
    impl_Block!();

    fn update(&mut self) {
        use std::str;
        use std::process::Command;
        let output = Command::new("sh")
            .arg("-c")
            .arg(&self.command)
            .output()
            .expect(&format!("failed to execute command '{}'", self.command));

        let strval = str::from_utf8(&output.stdout)
            .expect("custom process returned bad output")
            .to_string();

        if !output.status.success() {
            self.base.info.retry = self.retry;
            return;
        }

        let data: Vec<&str> = strval.split('\n').collect();
        self.base.data = match data.len() {
            0 => block::Value::None,
            1 => {
                let value = data[0].parse::<u32>();
                if value.is_ok() {
                    block::Value::new((data[0], self.base.get_color(value.unwrap())))
                } else {
                    block::Value::new(data[0])
                }
            }
            _ => {
                block::Value::new((
                    data[0],
                    utility::read_color(data[1], block::DEFAULT_COLOR),
                ))
            }
        }
    }
}
