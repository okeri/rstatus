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
}

impl Block {
    pub fn new(base: block::Base) -> Block {
        Block { base: base }
    }
}

impl block::Block for Block {
    impl_Block!();
    fn update(&mut self) {
        use utility;
        let data = utility::read_file("/proc/meminfo").expect("cannot open /proc/meminfo");
        let values: Vec<&str> = data.split_whitespace().collect();

        let total = values[1].trim().parse::<f64>().expect(
            "cannot parse procfs for memory info",
        );
        let free = values[7].trim().parse::<f64>().expect(
            "cannot parse procfs for memory info",
        );
        let value = (1f64 - free / total) * 100f64;
        self.base.data = block::Value::new((value as u32, self.base.get_color(value as u32)));
    }
}
