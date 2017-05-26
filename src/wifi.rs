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
    interface: String,
}

impl Block {
    pub fn new(base: block::Base, interface: String) -> Block {
        Block {
            base: base,
            interface: interface,
        }
    }
}

impl block::Block for Block {
    impl_Block!();
    fn update(&mut self) {
        use utility;
        let data = utility::read_file("/proc/net/wireless")
            .expect("cannot open /proc/net/wireless");
        for line in data.split('\n') {
            if line.find(&self.interface).is_some() {
                let linedata: Vec<&str> = line.split_whitespace().collect();
                let level = linedata[2][0..linedata[2].len() - 1]
                    .parse::<u32>()
                    .unwrap_or(0) * 100 / 70;
                self.base.data = if level > 0 {
                    block::Value::new((level, self.base.get_color(level)))
                } else {
                    block::Value::None
                };
            }
        }
    }
}
