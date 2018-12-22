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

extern crate time;
use block;

pub struct Block {
    base: block::Base,
    sensor: String,
}

impl Block {
    pub fn new(base: block::Base, sensor: String) -> Block {
        Block {
            base: base,
            sensor: sensor,
        }
    }
}

impl block::Block for Block {
    impl_Block!();
    fn update(&mut self) {
        use utility;
        let val = utility::read_file(&self.sensor).expect(&format!("cannot open {}", self.sensor));

        let temp = val.trim().parse::<u32>().expect(&format!(
            "expected integer in {}",
            self.sensor
        )) / 1000;
        self.base.data = block::Value::new((temp, self.base.get_color(temp)));
    }
}
