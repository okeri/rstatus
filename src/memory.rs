use super::base::{Base, Value};
use super::block;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Block {
    #[serde(flatten)]
    base: Base,
}

fn get_mem_usage() -> Result<u32, ()> {
    let text = std::fs::read_to_string("/proc/meminfo").map_err(|_| ())?;
    let find = |key: &str| -> Option<f64> {
        let pos = text.find(key)?;
        text[pos + key.len()..].split_whitespace().next()?.parse().ok()
    };
    let total = find("MemTotal:").ok_or(())?;
    let available = find("MemAvailable:").ok_or(())?;
    if total == 0.0 {
        return Err(());
    }
    Ok(((1.0 - available / total) * 100.0) as u32)
}

impl block::Block for Block {
    impl_Block!();
    fn update(&mut self) {
        self.base.value = Value::new(get_mem_usage());
    }
}
