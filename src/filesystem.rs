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
    path: String,
}

impl block::Block for Block {
    impl_Block!();
    fn update(&mut self) {
        use std::ffi::CString;
        use std::mem;
        unsafe {
            let path = CString::new(self.path.clone()).unwrap();
            let mut usage: libc::statvfs = mem::zeroed();
            self.base.value = if libc::statvfs(path.as_ptr() as *const i8, &mut usage) == -1 {
                Value::Invalid
            } else {
                let used = (100 * (usage.f_blocks - usage.f_bfree) / usage.f_blocks) as u32;
                Value::new(used)
            };
        }
    }
}
