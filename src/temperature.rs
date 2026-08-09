use super::base::{Base, Value};
use super::block;
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Deserialize)]
pub struct Block {
    #[serde(flatten)]
    base: Base,
    sensor: String,
    #[serde(skip)]
    inputs: Option<Vec<PathBuf>>,
}

impl block::Block for Block {
    impl_Block!();
    fn update(&mut self) {
        let inputs = match self.inputs {
            Some(ref inputs) => inputs,
            None => {
                let found = discover(&self.sensor);
                if found.is_empty() {
                    // sensor may appear later, so do not cache the empty result
                    self.base.value = Value::Invalid;
                    return;
                }
                self.inputs.insert(found)
            }
        };

        self.base.value = Value::new(
            inputs
                .iter()
                .filter_map(|path| std::fs::read_to_string(path).ok())
                .filter_map(|text| text.trim().parse::<i32>().ok())
                .max()
                .map(|value| (value / 1000).max(0) as u32),
        )
    }
}

fn matches(mask: &str, name: &str) -> bool {
    let m: Vec<char> = mask.chars().collect();
    let n: Vec<char> = name.chars().collect();
    let (mut mi, mut ni) = (0, 0);
    let mut star = None;
    let mut retry = 0;

    while ni < n.len() {
        if mi < m.len() && (m[mi] == '?' || m[mi] == n[ni]) {
            mi += 1;
            ni += 1;
        } else if mi < m.len() && m[mi] == '*' {
            star = Some(mi);
            mi += 1;
            retry = ni;
        } else if let Some(pos) = star {
            mi = pos + 1;
            retry += 1;
            ni = retry;
        } else {
            return false;
        }
    }
    m[mi..].iter().all(|c| *c == '*')
}

fn discover(mask: &str) -> Vec<PathBuf> {
    let mut result = Vec::new();

    if let Ok(entries) = std::fs::read_dir("/sys/class/thermal") {
        for entry in entries.flatten() {
            if !entry
                .file_name()
                .to_string_lossy()
                .starts_with("thermal_zone")
            {
                continue;
            }
            let dir = entry.path();
            if let Ok(zone_type) = std::fs::read_to_string(dir.join("type")) {
                let input = dir.join("temp");
                if matches(mask, zone_type.trim()) && input.exists() {
                    result.push(input);
                }
            }
        }
    }

    if let Ok(entries) = std::fs::read_dir("/sys/class/hwmon") {
        for entry in entries.flatten() {
            let dir = entry.path();
            let chip = std::fs::read_to_string(dir.join("name")).unwrap_or_default();
            let chip_matched = matches(mask, chip.trim());
            let inputs = match std::fs::read_dir(&dir) {
                Ok(inputs) => inputs,
                Err(_) => continue,
            };
            for input in inputs.flatten() {
                let name = input.file_name().to_string_lossy().into_owned();
                let index = match name
                    .strip_prefix("temp")
                    .and_then(|rest| rest.strip_suffix("_input"))
                {
                    Some(index) => index,
                    None => continue,
                };
                if chip_matched {
                    result.push(input.path());
                    continue;
                }
                let label = std::fs::read_to_string(dir.join(format!("temp{}_label", index)))
                    .unwrap_or_default();
                let label = label.trim();
                if !label.is_empty() && matches(mask, label) {
                    result.push(input.path());
                }
            }
        }
    }

    result
}
