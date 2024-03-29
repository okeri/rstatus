use super::base::{Base, Value};
use super::block;
use super::utility;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Block {
    #[serde(flatten)]
    base: Base,
}

struct CpuStat {
    idle: u64,
    total: u64,
}

impl CpuStat {
    fn sub(&self) -> u64 {
        self.total - self.idle
    }
}

impl block::Block for Block {
    impl_Block!();
    fn update(&mut self) {
        if let Ok(data) = utility::read_filen("/proc/stat", 96) {
            let values: Vec<&str> = data.split_whitespace().collect();

            static mut PREV_STAT: Option<CpuStat> = None;
            let mut stat: CpuStat = CpuStat { idle: 0, total: 0 };

            for e in 1..7 {
                if e == 4 {
                    stat.idle = values[4].trim().parse::<u64>().unwrap_or(0);
                    stat.total += stat.idle;
                } else {
                    stat.total += values[e].trim().parse::<u64>().unwrap_or(0);
                }
            }
            unsafe {
                self.base.value = if let Some(ref prev) = PREV_STAT {
                    let value = (stat.sub() - prev.sub()) * 100 / (stat.total - prev.total);
                    Value::new(value as u32)
                } else {
                    Value::Invalid
                };
                PREV_STAT = Some(stat);
            }
        }
    }
}
