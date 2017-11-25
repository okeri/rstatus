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

struct CpuStat {
    idle: u64,
    total: u64,
}

impl block::Block for Block {
    impl_Block!();
    fn update(&mut self) {
        use utility;
        let data = utility::read_file("/proc/stat").expect("cannot open /proc/stat");
        let values: Vec<&str> = data.split_whitespace().collect();

        static mut INITED: bool = false;
        static mut PREV_STAT: CpuStat = CpuStat{idle: 0, total: 0};
        let mut stat: CpuStat = CpuStat{idle: 0, total: 0};


        for e in 1..7 {
            if e == 4 {
                stat.idle = values[4].trim()
                    .parse::<u64>()
                    .expect("cannot parse procfs stat info");
                stat.total += stat.idle;
            } else {
                stat.total += values[e].trim()
                    .parse::<u64>()
                    .expect("cannot parse procfs stat info");
            }
        }
        unsafe {
            if INITED {
                let value = ((stat.total - stat.idle) - (PREV_STAT.total - PREV_STAT.idle)) * 100 /
                    (stat.total - PREV_STAT.total);

                self.base.data = block::Value::new((value as u32, self.base.get_color(value as u32)));
            } else {
                INITED = true;
                self.base.data = block::Value::None;
            }
            PREV_STAT = stat;
        }
    }
}
