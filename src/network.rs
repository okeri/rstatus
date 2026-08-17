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

const PERC_LUT: [u32; 100] = [
    100, 100, 100, 100, 100, 100, 100, 100, 100, 100, 100, 100, 100, 100, 100, 100, 100, 100, 100,
    100, 99, 99, 99, 98, 98, 98, 97, 97, 96, 96, 95, 95, 94, 93, 93, 92, 91, 90, 90, 89, 88, 87,
    86, 85, 84, 83, 82, 81, 80, 79, 78, 76, 75, 74, 73, 71, 70, 69, 67, 66, 64, 63, 61, 60, 58, 56,
    55, 53, 51, 50, 48, 46, 44, 42, 40, 38, 36, 34, 32, 30, 28, 26, 24, 22, 20, 17, 15, 13, 10, 8,
    6, 3, 1, 1, 1, 1, 1, 1, 1, 1,
];

fn is_wireless(iname: &str) -> bool {
    std::path::Path::new(&format!("/sys/class/net/{}/wireless", iname)).is_dir()
}

fn get_wifi_strength(iname: &str) -> Result<u32, ()> {
    let ifindex = std::fs::read_to_string(format!("/sys/class/net/{}/ifindex", iname))
        .map_err(|_| ())?
        .trim()
        .parse::<i32>()
        .map_err(|_| ())?;

    let signal = neli_wifi::Socket::connect()
        .map_err(|_| ())?
        .get_station_info(ifindex)
        .map_err(|_| ())?
        .into_iter()
        .find_map(|station| station.signal)
        .ok_or(())?;

    let index = (-1 - i32::from(signal)).clamp(0, PERC_LUT.len() as i32 - 1) as usize;
    Ok(PERC_LUT[index])
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
            if is_wireless(&iface) {
                self.base.set_prefix(&self.wifi);
                self.base.set_suffix("%");
                self.base.value = Value::new(get_wifi_strength(&iface));
            } else {
                self.base.set_prefix("");
                self.base.set_suffix("");
                self.base.value = Value::new(self.ethernet.clone());
            }
        }
    }
}
