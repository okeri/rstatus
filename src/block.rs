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

pub struct Info {
    pub interval: u32,
    pub retry: u32,
    pub signal: u32,
}

pub enum Value {
    None,
    Int { value: u32, color: u32 },
    Str { value: String, color: u32 },
}

pub trait ValueConstructor {
    fn new(&self) -> Value;
}

impl ValueConstructor for u32 {
    fn new(&self) -> Value {
        Value::Int {
            value: *self,
            color: DEFAULT_COLOR,
        }
    }
}

impl ValueConstructor for String {
    fn new(&self) -> Value {
        Value::Str {
            value: self.clone(),
            color: DEFAULT_COLOR,
        }
    }
}

impl<'a> ValueConstructor for &'a str {
    fn new(&self) -> Value {
        Value::Str {
            value: self.to_string(),
            color: DEFAULT_COLOR,
        }
    }
}

impl ValueConstructor for (u32, u32) {
    fn new(&self) -> Value {
        Value::Int {
            value: self.0,
            color: self.1,
        }
    }
}

impl<'a> ValueConstructor for (&'a str, u32) {
    fn new(&self) -> Value {
        Value::Str {
            value: self.0.to_string(),
            color: self.1,
        }
    }
}

impl ValueConstructor for (String, u32) {
    fn new(&self) -> Value {
        Value::Str {
            value: self.0.clone(),
            color: self.1,
        }
    }
}

impl Value {
    pub fn new<T: ValueConstructor>(arg: T) -> Self {
        arg.new()
    }
}

pub struct Threshold {
    pub value: u32,
    pub color: u32,
}
pub const DEFAULT_COLOR: u32 = 0xffffff;

pub struct Base {
    pub info: Info,
    name: String,
    pub prefix: String,
    suffix: String,
    invalid: Value,
    pub color: u32,
    pub data: Value,
    thresholds: Vec<Threshold>,
}

impl Base {
    pub fn set_invalid(&mut self, name: String, color: u32) {
        self.invalid = Value::new((name, color));
    }
    pub fn push_threshold(&mut self, value: u32, color: u32) {
        self.thresholds.push(Threshold {
            value: value,
            color: color,
        });
    }
    pub fn new(info: Info) -> Self {
        Base {
            info: info,
            name: "undefined".to_string(),
            prefix: "".to_string(),
            suffix: "".to_string(),
            invalid: Value::None,
            color: DEFAULT_COLOR,
            data: Value::None,
            thresholds: vec![],
        }
    }
    pub fn name(&mut self) -> &str {
        &self.name
    }
    pub fn set_name(&mut self, name: String) {
        self.name = name
    }
    pub fn set_prefix(&mut self, prefix: String) {
        self.prefix = prefix
    }
    pub fn set_suffix(&mut self, suffix: String) {
        self.suffix = suffix
    }

    pub fn set_color(&mut self, color: u32) {
        self.color = color;
    }
    pub fn retry(&mut self, interval: u32) -> bool {
        if self.info.retry != 0 {
            if self.info.retry <= interval {
                self.info.retry = 0;
                return true;
            } else {
                self.info.retry -= interval;
            }
        }
        false
    }

    pub fn serialize(&self) {
        print!("{{\"name\":\"{}\",\"full_text\":", self.name);
        let mut print_color: u32 = self.color;
        match self.data {
            Value::Int { value, color } => {
                print!("\"{}{}{}\"", self.prefix, value, self.suffix);
                print_color = color
            }
            Value::Str { ref value, color } => {
                print!("\"{}{}{}\"", self.prefix, value, self.suffix);
                print_color = color
            }
            Value::None => {
                if let Value::Str { ref value, color } = self.invalid {
                    print!("\"{}\"", value);
                    print_color = color
                }
            }
        }

        if print_color != DEFAULT_COLOR {
            print!(",\"color\":\"#{:06X}\"", print_color);
        }
        print!(",\"separator_block_width\":16}}");
    }

    pub fn get_color(&self, value: u32) -> u32 {
        for to in self.thresholds.iter().rev() {
            if value >= to.value {
                return to.color;
            }
        }
        self.color
    }
}

pub trait Block {
    fn update(&mut self);
    fn info(&self) -> &Info;
    fn serialize(&self);
    fn retry(&mut self, interval: u32) -> bool;
}

macro_rules! impl_Block {
    () => {
        fn info(&self) -> &block::Info {
            &self.base.info
        }
        fn serialize(&self) {
            self.base.serialize();
        }
        fn retry(&mut self, interval: u32) -> bool {
            self.base.retry(interval)
        }
    }
}
