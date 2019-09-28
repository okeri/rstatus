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

pub trait Block {
    fn update(&mut self);
    fn set_name(&mut self, name: String);
    fn interval(&self) -> u32;
    fn signal(&self) -> u32;
    fn bgcolor(&self) -> Option<u32>;
    fn render(&self, prev_bg: Option<u32>);
    fn retry(&mut self, interval: u32) -> bool;
}

macro_rules! impl_Block {
    () => {
        fn interval(&self) -> u32 {
            self.base.interval()
        }
        fn signal(&self) -> u32 {
            self.base.signal()
        }
        fn render(&self, prev_bg: Option<u32>) {
            self.base.render(prev_bg);
        }
        fn retry(&mut self, interval: u32) -> bool {
            self.base.retry(interval)
        }
        fn set_name(&mut self, name: String) {
            self.base.set_name(name);
        }
        fn bgcolor(&self) -> Option<u32> {
            self.base.bg()
        }
    }
}
