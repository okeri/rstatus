use super::base::{Base, Value};
use super::block;
use chrono::Local;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Block {
    #[serde(flatten)]
    base: Base,
    #[serde(default = "default_format")]
    format: String,
}

impl block::Block for Block {
    impl_Block!();
    fn update(&mut self) {
        self.base.value = Value::new(format!("{}", Local::now().format(&self.format)));
    }
}

fn default_format() -> String {
    "%d.%m.%Y %H:%M".to_string()
}
