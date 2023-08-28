use super::block_builder::{BlockBuilder, RenderFlags, SubBlock};
use super::utility::read_color;
use serde::{Deserialize, Deserializer};
use std::collections::BTreeMap;

#[derive(Deserialize, Default)]
pub enum Value {
    #[default]
    Invalid,
    Int(u32),
    Str(String),
}

pub trait ValueConstructor {
    fn build(&self) -> Value;
}

impl ValueConstructor for u32 {
    fn build(&self) -> Value {
        Value::Int(*self)
    }
}

impl ValueConstructor for Option<u32> {
    fn build(&self) -> Value {
        self.map_or(Value::Invalid, Value::Int)
    }
}

impl ValueConstructor for String {
    fn build(&self) -> Value {
        Value::Str(self.clone())
    }
}

impl<'a> ValueConstructor for &'a str {
    fn build(&self) -> Value {
        Value::Str(self.to_string())
    }
}

impl<E> ValueConstructor for Result<u32, E> {
    fn build(&self) -> Value {
        match *self {
            Ok(value) => Value::Int(value),
            Err(_) => Value::Invalid,
        }
    }
}

impl<E> ValueConstructor for Result<String, E> {
    fn build(&self) -> Value {
        match *self {
            Ok(ref value) => Value::Str(value.clone()),
            Err(_) => Value::Invalid,
        }
    }
}

impl Value {
    pub fn new<T: ValueConstructor>(arg: T) -> Self {
        arg.build()
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
    /// with of separator
    #[serde(default = "default_zero")]
    separator_width: u32,
    /// use self-rendered arrow as separator
    #[serde(default = "default_str_none")]
    custom_separator: Option<String>,
    /// foreground color for value
    #[serde(default = "default_color", deserialize_with = "parse_color")]
    color: u32,
    /// background color for value
    #[serde(default = "default_none", deserialize_with = "parse_color_maybe")]
    bgcolor: Option<u32>,
    /// prefix for value
    #[serde(default)]
    prefix: String,
    #[serde(skip, default)]
    iprefix: String,
    /// prefix color for value
    #[serde(default = "default_none", deserialize_with = "parse_color_maybe")]
    prefix_color: Option<u32>,
    /// suffix for value
    #[serde(default)]
    suffix: String,
    #[serde(skip, default)]
    isuffix: String,
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
    #[serde(skip, default = "default_index")]
    index: usize,
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
        if self.custom_separator.is_some() && self.bgcolor.is_some() {
            print!("{{\"full_text\":\"{}\",\"separator\":false,\"separator_block_width\":0,\"color\":\"#{:06X}\"",
                   self.custom_separator.as_ref().unwrap(),
                   self.bgcolor.unwrap());
            Base::render_bg(prev_bg);
            print!("}},");
        }

        let suffix_flags = |flags: RenderFlags| {
            flags
                | if self.custom_separator.is_some() {
                    RenderFlags::None
                } else {
                    RenderFlags::Separator
                }
        };

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
                    .add(&self.iprefix, prefix_color, RenderFlags::None)
                    .add(&value.to_string(), color, RenderFlags::Name)
                    .add(&self.isuffix, suffix_color, suffix_flags(RenderFlags::None))
                    .add(&self.suffix, suffix_color, suffix_flags(RenderFlags::None))
                    .get()
            }

            Value::Str(ref value) => BlockBuilder::new()
                .add(
                    &self.prefix,
                    self.prefix_color.unwrap_or(self.color),
                    RenderFlags::None,
                )
                .add(
                    &self.iprefix,
                    self.prefix_color.unwrap_or(self.color),
                    RenderFlags::None,
                )
                .add(value, self.color, RenderFlags::Name)
                .add(
                    &self.isuffix,
                    self.suffix_color.unwrap_or(self.color),
                    suffix_flags(RenderFlags::None),
                )
                .add(
                    &self.suffix,
                    self.suffix_color.unwrap_or(self.color),
                    suffix_flags(RenderFlags::None),
                )
                .get(),
            Value::Invalid => BlockBuilder::new()
                .add(
                    &self.invalid,
                    self.invalid_color,
                    suffix_flags(RenderFlags::Name),
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

    pub fn interval(&self) -> u32 {
        self.interval
    }

    pub fn signal(&self) -> u32 {
        self.signal
    }

    pub fn set_index(&mut self, index: usize) {
        self.index = index;
    }

    pub fn bg(&self) -> Option<u32> {
        self.bgcolor
    }

    pub fn set_prefix(&mut self, prefix: &str) {
        self.iprefix = prefix.to_owned();
    }

    pub fn set_suffix(&mut self, suffix: &str) {
        self.isuffix = suffix.to_owned();
    }

    pub fn set_color(&mut self, color: u32) {
        self.color = color;
    }

    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }

    pub fn index(&self) -> usize {
        self.index
    }
}

fn parse_color<'de, D>(deserializer: D) -> Result<u32, D::Error>
where
    D: Deserializer<'de>,
{
    let s: Option<String> = Option::deserialize(deserializer)?;
    if let Some(text) = s {
        return Ok(read_color(&text, 0));
    }
    Ok(0)
}

fn parse_color_maybe<'de, D>(deserializer: D) -> Result<Option<u32>, D::Error>
where
    D: Deserializer<'de>,
{
    let s: Option<String> = Option::deserialize(deserializer)?;
    if let Some(text) = s {
        return Ok(Some(read_color(&text, 0)));
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
        result.insert(threshold, read_color(&color, 0));
    }
    Ok(result)
}

fn default_invalid() -> String {
    "invalid".to_string()
}

pub fn default_zero() -> u32 {
    0
}

fn default_index() -> usize {
    0
}

fn default_color() -> u32 {
    0xffffff
}

fn default_invalid_color() -> u32 {
    0xff0000
}

fn default_none() -> Option<u32> {
    None
}

pub fn default_str_none() -> Option<String> {
    None
}

fn default_thresholds() -> Thresholds {
    Thresholds::new()
}

pub fn default_false() -> bool {
    false
}
