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

use std::convert::From;
use std::ops::{BitAnd, BitOr, BitOrAssign};

#[repr(u8)]
#[derive(Copy)]
pub enum RenderFlags {
    None = 0,
    Separator = 1,
    Name = 2,
    Both = 3,
}

impl From<u8> for RenderFlags {
    fn from(value: u8) -> Self {
        match value {
            1 => RenderFlags::Separator,
            2 => RenderFlags::Name,
            3 => RenderFlags::Both,
            _ => RenderFlags::None,
        }
    }
}

impl Clone for RenderFlags {
    fn clone(&self) -> Self {
        return RenderFlags::from(*self as u8);
    }
}

impl BitOrAssign for RenderFlags {
    fn bitor_assign(&mut self, rhs: Self) {
        *self = RenderFlags::from(*self as u8 | rhs as u8);
    }
}

impl BitOr for RenderFlags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        RenderFlags::from(self as u8 | rhs as u8)
    }
}

impl BitAnd for RenderFlags {
    type Output = bool;
    fn bitand(self, rhs: Self) -> Self::Output {
        (self as u8 & rhs as u8) != 0
    }
}

pub struct SubBlock {
    pub text: String,
    pub color: u32,
    pub flags: RenderFlags,
}

impl SubBlock {
    fn new(text: &str, color: u32, flags: RenderFlags) -> Self {
        SubBlock {
            text: text.to_owned(),
            color: color,
            flags: flags,
        }
    }
}

pub struct BlockBuilder {
    sub_blocks: Vec<SubBlock>,
}

impl BlockBuilder {
    pub fn new() -> Self {
        BlockBuilder {
            sub_blocks: Vec::new(),
        }
    }

    pub fn add(mut self, text: &str, color: u32, flags: RenderFlags) -> Self {
        let last_exists = self.sub_blocks.iter_mut().last();

        if let Some(last) = last_exists {
            if color == last.color {
                last.text += text;
                last.flags |= flags;
                return self;
            }
        }
        self.sub_blocks.push(SubBlock::new(text, color, flags));
        self
    }

    pub fn get(self) -> Vec<SubBlock> {
        self.sub_blocks
    }
}
