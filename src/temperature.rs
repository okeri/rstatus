use super::base::{Base, Value};
use super::block;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Block {
    #[serde(flatten)]
    base: Base,
    sensor: String,
}

impl block::Block for Block {
    impl_Block!();
    fn update(&mut self) {
        self.base.value = Value::new(
            std::fs::read_to_string(&self.sensor)
                .ok()
                .and_then(|text| text.trim().parse::<u32>().ok().map(|v| v / 1000)),
        )
    }
}
