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
}

impl Block {
    pub fn new(base: block::Base, command: String) -> Block {
        Block {
            base: base,
            command: command,
        }
    }
}

impl block::Block for Block {
    impl_Block!();
    fn update(&mut self) {
        use std::str;
        use std::process::Command;
        let output = str::from_utf8(&Command::new("sh")
                                         .arg("-c")
                                         .arg(&self.command)
                                         .output()
                                         .expect("failed to execute command")
                                         .stdout)
                .expect("custom process returned bad output")
                .to_string();
        let data: Vec<&str> = output.split('\n').collect();
        self.base.data =
            match data.len() {
                0 => {
                    block::Value::None
                }
                1 => {
                    let value = data[0].parse::<u32>();
                    if value.is_ok() {
                        block::Value::new((data[0],
                                           self.base.get_color(value.unwrap())))
                    } else {
                        block::Value::new(data[0])
                    }
                }
                _ => {
                    block::Value::new((
                        data[0], utility::read_color(data[1], block::DEFAULT_COLOR)))
                }
            }
    }
}
