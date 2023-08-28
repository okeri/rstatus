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
        *self
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
            color,
            flags,
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
