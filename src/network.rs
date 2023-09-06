use super::base::{Base, Value};
use super::block;
use iwlib;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Block {
    #[serde(flatten)]
    base: Base,
    #[serde(default = "default_wifi")]
    wifi: String,
    #[serde(default = "default_ethernet")]
    ethernet: String,
}

fn default_ethernet() -> String {
    "eth".to_string()
}

fn default_wifi() -> String {
    "wifi".to_string()
}

fn get_wifi_strength(interface: &str) -> Result<u32, ()> {
    iwlib::get_wireless_info(interface)
        .ok_or(())
        .map(|info| info.wi_quality.into())
}

fn get_active_interface() -> Result<String, ()> {
    if let Ok(text) = std::fs::read_to_string("/proc/net/route") {
        let mut lowest_metric = 0xffff;
        let mut iface = "".to_owned();
        for line in text.split('\n').filter(|value| !value.contains("Iface")) {
            let linedata: Vec<&str> = line.split_whitespace().collect();
            if linedata.len() > 4 && linedata[1] == "00000000" {
                let metric = linedata[5].parse::<u32>().unwrap_or(0xffff);
                if metric < lowest_metric {
                    lowest_metric = metric;
                    iface = linedata[0].to_owned();
                }
            }
        }
        if !iface.is_empty() {
            return Ok(iface);
        }
    }
    Err(())
}

impl block::Block for Block {
    impl_Block!();

    fn update(&mut self) {
        self.base.value = Value::Invalid;
        if let Ok(iface) = get_active_interface() {
            if let Ok(strength) = get_wifi_strength(&iface) {
                self.base.set_prefix(&self.wifi);
                self.base.set_suffix("%");
                self.base.value = Value::new(strength);
            } else if let Value::Invalid = self.base.value {
                self.base.set_prefix("");
                self.base.set_suffix("");
                self.base.value = Value::new(self.ethernet.clone());
            }
        }
    }
}
