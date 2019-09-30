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

use super::block_builder::{BlockBuilder, RenderFlags, SubBlock};
use serde::{Deserialize, Deserializer};
use std::collections::BTreeMap;

#[derive(Deserialize)]
pub enum Value {
    Invalid,
    Int(u32),
    Str(String),
}

pub trait ValueConstructor {
    fn new(&self) -> Value;
}

impl ValueConstructor for u32 {
    fn new(&self) -> Value {
        Value::Int(*self)
    }
}

impl ValueConstructor for String {
    fn new(&self) -> Value {
        Value::Str(self.clone())
    }
}

impl<'a> ValueConstructor for &'a str {
    fn new(&self) -> Value {
        Value::Str(self.to_string())
    }
}

impl<E> ValueConstructor for Result<u32, E> {
    fn new(&self) -> Value {
        match *self {
            Ok(value) => Value::Int(value),
            Err(_) => Value::Invalid,
        }
    }
}

impl<E> ValueConstructor for Result<String, E> {
    fn new(&self) -> Value {
        match *self {
            Ok(ref value) => Value::Str(value.clone()),
            Err(_) => Value::Invalid,
        }
    }
}

impl Value {
    pub fn new<T: ValueConstructor>(arg: T) -> Self {
        arg.new()
    }
}

type Thresholds = BTreeMap<u32, u32>;

#[derive(Deserialize)]
pub struct Base {
    /// value to display
    #[serde(skip)]
    pub value: Value,
    /// interval for update block
    #[serde(default = "default_zero")]
    interval: u32,
    /// signal for update block
    #[serde(default = "default_zero")]
    signal: u32,
    /// name of the block
    #[serde(skip)]
    name: String,
    /// retry interval in case of value is invalid
    #[serde(skip, default = "default_zero")]
    retry: u32,
    /// with of separator
    #[serde(default = "default_zero")]
    separator_width: u32,
    /// use self-rendered arrow as separator
    #[serde(default = "default_false")]
    separator_arrow: bool,
    /// foreground color for value
    #[serde(default = "default_color", deserialize_with = "parse_color")]
    color: u32,
    /// background color for value
    #[serde(default = "default_none", deserialize_with = "parse_color_maybe")]
    bgcolor: Option<u32>,
    /// prefix for value
    #[serde(default)]
    prefix: String,
    /// prefix color for value
    #[serde(default = "default_none", deserialize_with = "parse_color_maybe")]
    prefix_color: Option<u32>,
    /// suffix for value
    #[serde(default)]
    suffix: String,
    /// suffix color
    #[serde(default = "default_none", deserialize_with = "parse_color_maybe")]
    suffix_color: Option<u32>,
    /// invalid value to be shown
    #[serde(default = "default_invalid")]
    invalid: String,
    /// color of invalid value
    #[serde(default = "default_invalid_color", deserialize_with = "parse_color")]
    invalid_color: u32,
    /// fix prefix and suffix color depend on threshold
    #[serde(default = "default_false")]
    threshold_fix: bool,
    /// color thresholds
    #[serde(default = "default_thresholds", deserialize_with = "parse_thresholds")]
    thresholds: Thresholds,
}

impl Base {
    fn render_bg(bg: Option<u32>) {
        if let Some(c) = bg {
            print!(",\"background\":\"#{:06X}\"", c);
        }
    }

    fn render_subblock(&self, subblock: &SubBlock) {
        print!("{{\"full_text\":\"{}\",", subblock.text);
        if subblock.flags & RenderFlags::Separator {
            print!("\"separator_block_width\":{}", self.separator_width);
        } else {
            print!("\"separator\":false,\"separator_block_width\":0");
        }
        Base::render_bg(self.bg());
        if subblock.flags & RenderFlags::Name {
            print!(",\"name\":\"{}\"", self.name);
        }
        print!(",\"color\":\"#{:06X}\"}}", subblock.color);
    }

    fn get_to_color(&self, value: u32) -> u32 {
        for (to, color) in self.thresholds.iter().rev() {
            if value >= *to {
                return *color;
            }
        }
        self.color
    }
    
    pub fn render(&self, prev_bg: Option<u32>) {
        if self.separator_arrow && self.bgcolor.is_some() {
            print!("{{\"full_text\":\"\\ue0b2\",\"separator\":false,\"separator_block_width\":0,\"color\":\"#{:06X}\"",
                   self.bgcolor.unwrap());
            Base::render_bg(prev_bg);
            print!("}},");
        }

        let subblocks = match self.value {
            Value::Int(value) => {
                let color = self.get_to_color(value);
                let mut prefix_color = self.prefix_color.unwrap_or(color);
                let mut suffix_color = self.suffix_color.unwrap_or(color);
                if self.threshold_fix && color != self.color {
                    prefix_color = color;
                    suffix_color = color;
                }
                BlockBuilder::new()
                    .add(&self.prefix, prefix_color, RenderFlags::None)
                    .add(&value.to_string(), color, RenderFlags::Name)
                    .add(
                        &self.suffix,
                        suffix_color,
                        if self.separator_arrow {
                            RenderFlags::None
                        } else {
                            RenderFlags::Separator
                        },
                    )
                    .get()
            }
            Value::Str(ref value) => BlockBuilder::new()
                .add(
                    &self.prefix,
                    self.prefix_color.unwrap_or(self.color),
                    RenderFlags::None,
                )
                .add(value, self.color, RenderFlags::Name)
                .add(
                    &self.suffix,
                    self.suffix_color.unwrap_or(self.color),
                    if self.separator_arrow {
                        RenderFlags::None
                    } else {
                        RenderFlags::Separator
                    },
                )
                .get(),
            Value::Invalid => BlockBuilder::new()
                .add(
                    &self.invalid,
                    self.invalid_color,
                    if self.separator_arrow {
                        RenderFlags::Name
                    } else {
                        RenderFlags::Separator | RenderFlags::Name
                    },
                )
                .get(),
        };

        let mut first = true;
        for sb in subblocks.iter() {
            if !first {
                print!(",");
            } else {
                first = false
            }
            self.render_subblock(sb);
        }
    }

    pub fn retry(&mut self, interval: u32) -> bool {
        if self.retry != 0 {
            if self.retry <= interval {
                self.retry = 0;
                return true;
            } else {
                self.retry -= interval;
            }
        }
        false
    }

    pub fn interval(&self) -> u32 {
        self.interval
    }

    pub fn signal(&self) -> u32 {
        self.signal
    }

    pub fn bg(&self) -> Option<u32> {
        self.bgcolor
    }

    pub fn set_prefix(&mut self, prefix: &str) {
        self.prefix = prefix.to_owned();
    }

    pub fn set_suffix(&mut self, suffix: &str) {
        self.suffix = suffix.to_owned();
    }

    pub fn set_color(&mut self, color: u32) {
        self.color = color;
    }

    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }

    pub fn set_retry(&mut self, retry: u32) {
        self.retry = retry;
    }
}

fn read_color(input: &str) -> u32 {
    if input.chars().next().expect("invalid color value") == '#' {
        i64::from_str_radix(&input[1..], 16).expect("invalid color value") as u32
    } else {
        i64::from_str_radix(input, 16).expect("invalid color value") as u32
    }
}

fn parse_color<'de, D>(deserializer: D) -> Result<u32, D::Error>
where
    D: Deserializer<'de>,
{
    let s: Option<String> = Option::deserialize(deserializer)?;
    if let Some(text) = s {
        return Ok(read_color(&text));
    }
    Ok(0)
}

fn parse_color_maybe<'de, D>(deserializer: D) -> Result<Option<u32>, D::Error>
where
    D: Deserializer<'de>,
{
    let s: Option<String> = Option::deserialize(deserializer)?;
    if let Some(text) = s {
        return Ok(Some(read_color(&text)));
    }
    Ok(None)
}

fn parse_thresholds<'de, D>(deserializer: D) -> Result<Thresholds, D::Error>
where
    D: Deserializer<'de>,
{
    let ms: BTreeMap<u32, String> = BTreeMap::deserialize(deserializer)?;
    let mut result = Thresholds::new();
    for (threshold, color) in ms {
        result.insert(threshold, read_color(&color));
    }
    Ok(result)
}

fn default_invalid() -> String {
    "invalid".to_string()
}

fn default_zero() -> u32 {
    0
}

fn default_color() -> u32 {
    0xffffff
}

fn default_invalid_color() -> u32 {
    0xff0000
}

fn default_false() -> bool {
    false
}

fn default_none() -> Option<u32> {
    None
}

fn default_thresholds() -> Thresholds {
    Thresholds::new()
}

impl Default for Value {
    fn default() -> Self {
        Value::Invalid
    }
}
