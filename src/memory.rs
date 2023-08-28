use super::base::{Base, Value};
use super::block;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Block {
    #[serde(flatten)]
    base: Base,
}

fn get_mem_usage() -> Result<u32, ()> {
    std::fs::read_to_string("/proc/meminfo")
        .map_err(|_| ())
        .and_then(|text| {
            let values: Vec<&str> = text.split_whitespace().collect();
            let total = values[1].trim().parse::<f64>().map_err(|_| ())?;
            let free = values[7].trim().parse::<f64>().map_err(|_| ())?;
            let value = (1f64 - free / total) * 100f64;
            Ok(value as u32)
        })
}

impl block::Block for Block {
    impl_Block!();
    fn update(&mut self) {
        self.base.value = Value::new(get_mem_usage());
    }
}
