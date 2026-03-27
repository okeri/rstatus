use super::base::{Base, Value};
use super::block;
use super::utility::read_color;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Block {
    #[serde(flatten)]
    base: Base,
    command: String,
}

impl block::Block for Block {
    impl_Block!();

    fn update(&mut self) {
        use std::process::Command;
        use std::str;
        let output = match Command::new("sh").arg("-c").arg(&self.command).output() {
            Ok(o) => o,
            Err(_) => {
                self.base.value = Value::Invalid;
                return;
            }
        };
        let strval = match str::from_utf8(&output.stdout) {
            Ok(s) => s.to_string(),
            Err(_) => {
                self.base.value = Value::Invalid;
                return;
            }
        };

        let data: Vec<&str> = strval.split('\n').collect();
        self.base.value = match data.len() {
            0 => Value::Invalid,
            1 => {
                if let Ok(value) = data[0].parse::<u32>() {
                    Value::new(value)
                } else if !data[0].is_empty() {
                    Value::new(data[0])
                } else {
                    Value::Invalid
                }
            }
            _ => {
                self.base.set_color(read_color(data[1], 0xffffff));
                Value::new(data[0])
            }
        }
    }
}
