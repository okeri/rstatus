use super::base::{Base, Value};
use super::block;
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

fn get_wifi_strength(iname: &str) -> Result<u32, ()> {
    let perc_lut = vec![
        100, 100, 100, 100, 100, 100, 100, 100, 100, 100, 100, 100, 100, 100, 100, 100, 100, 100,
        100, 100, 99, 99, 99, 98, 98, 98, 97, 97, 96, 96, 95, 95, 94, 93, 93, 92, 91, 90, 90, 89,
        88, 87, 86, 85, 84, 83, 82, 81, 80, 79, 78, 76, 75, 74, 73, 71, 70, 69, 67, 66, 64, 63, 61,
        60, 58, 56, 55, 53, 51, 50, 48, 46, 44, 42, 40, 38, 36, 34, 32, 30, 28, 26, 24, 22, 20, 17,
        15, 13, 10, 8, 6, 3, 1, 1, 1, 1, 1, 1, 1, 1,
    ];

    let interfaces = nl80211::Socket::connect()
        .map_err(|_| ())?
        .get_interfaces_info()
        .map_err(|_| ())?;

    for interface in interfaces {
        if interface.index.is_some() {
            if let Some(ref name_bytes) = interface.name {
                let name_bytes = std::str::from_utf8(name_bytes).map_err(|_| ())?;
                let name = name_bytes
                    .strip_suffix('\0')
                    .or(Some(name_bytes))
                    .ok_or(())?;
                if name == iname {
                    let station = interface.get_station_info().map_err(|_| ())?;
                    let signal: usize =
                        255 - (*station.signal.ok_or(())?.first().ok_or(())? as usize);
                    return Ok(perc_lut[signal.clamp(0, perc_lut.len())]);
                }
            }
        }
    }
    Err(())
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
