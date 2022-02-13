/*
  status bar for tiling wms like i3, sway, etc...
  Copyright (C) 2020 Oleg Keri

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

pub trait SoundService {
    fn id(&self) -> String;
    fn sink_name(&self) -> String;
    fn exists(&self, mixer: &str) -> bool;
    fn jack_plugged(&self) -> Option<bool>;
    fn update(&self);
    fn volume(&self, mixer: &str) -> Option<u32>;
    fn listen(&mut self, block_index: usize);
    fn set_mute(&self, mixer: &str, value: bool);
}
