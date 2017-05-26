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
use libc;

pub struct Block {
    base: block::Base,
    path: String,
}

impl Block {
    pub fn new(base: block::Base, path: String) -> Block {
        Block {
            base: base,
            path: path,
        }
    }
}

impl block::Block for Block {
    impl_Block!();
    fn update(&mut self) {
        use std::mem;
        use std::ffi::CString;
        unsafe {
            let path = CString::new(self.path.clone()).unwrap();
            let mut usage: libc::statvfs = mem::zeroed();
            self.base.data = if libc::statvfs(path.as_ptr() as *const i8, &mut usage) == -1 {
                block::Value::None
            } else {
                let used = (100 * (usage.f_blocks - usage.f_bfree) / usage.f_blocks) as u32;
                block::Value::new((used, self.base.get_color(used)))
            };
        }
    }
}
