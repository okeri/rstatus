/*
  status bar for i3like wms like i3, sway, etc...
  Copyright (C) 2017 Oleg Keri

  This program is free software: you can redistribute it and/or modify
  it under the terms of the GNU General Public License as published by
  the Free Software Foundation, either version 3 of the License, or
  (at your option) any later version.
  This program is distributed in the hope that it will be useful,
  but WITHOUT ANY WARRANTY; without even the implied warranty of
  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
  GNU General Public License for more details.
  You should have received a copy of the GNU General Public License
  along with this program.  If not, see <http://www.gnu.org/licenses/>
*/

extern crate yaml_rust;
use std;
use block;
use undefined;
use memory;
use temperature;
use filesystem;
use volume;
use cpuload;
use wifi;
use time;
use battery;
use custom;
use utility;
use self::yaml_rust::yaml::Yaml;

fn create(config: &Yaml) -> Box<block::Block> {
    let mut base = block::Base::new(block::Info {
        interval: 0,
        retry: 0,
        signal: 0,
    });

    if let Yaml::Hash(ref h) = *config {
        let (name, description) = h.iter().next().expect("empty blocks are not allowed");
        base.set_name(
            name.as_str()
                .expect("block name should be a string")
                .to_string(),
        );
        if let Yaml::Hash(ref items) = *description {
            let mut invalid_name = "invalid".to_string();
            let mut invalid_color = block::DEFAULT_COLOR;
            for (key, val) in items {
                match key.as_str().expect(
                    "identifiers inside blocks must be strings",
                ) {
                    "interval" => {
                        base.info.interval =
                            val.as_i64().expect("interval value must be integer") as u32;
                    }
                    "signal" => {
                        base.info.signal = val.as_i64().expect("signal value must be integer") as
                            u32;
                    }
                    "suffix" => {
                        base.set_suffix(
                            val.as_str()
                                .expect("suffix value must be string")
                                .to_string(),
                        );
                    }
                    "prefix" => {
                        base.set_prefix(
                            val.as_str()
                                .expect("prefix value must be string")
                                .to_string(),
                        );
                    }
                    "invalid" => {
                        invalid_name = val.as_str()
                            .expect("invalid value must be string")
                            .to_string();
                    }
                    "invalid_color" => {
                        invalid_color = utility::read_color(
                            val.as_str().expect("invalid color must be color value"),
                            block::DEFAULT_COLOR,
                        );
                    }
                    "color" => {
                        base.set_color(utility::read_color(
                            val.as_str().expect("invalid color must be color value"),
                            block::DEFAULT_COLOR));
                    }
                    "thresholds" => {
                        if let Yaml::Array(ref thresholds) = *val {
                            for to in thresholds {
                                base.push_threshold(
                                    to["threshold"].as_i64().expect(
                                        "threshold value must be defined",
                                    ) as u32,
                                    utility::read_color(
                                        to["color"].as_str().expect(
                                            "color of threshold must be defined",
                                        ),
                                        block::DEFAULT_COLOR,
                                    ),
                                );

                            }
                        }
                    }
                    _ => {}
                }
            }
            base.set_invalid(invalid_name, invalid_color);
            match base.name() {
                "temperature" => {
                    return Box::new(temperature::Block::new(
                        base,
                        items[&Yaml::String("sensor".to_string())]
                            .as_str()
                            .expect("temperature::sensor must be defined")
                            .to_string(),
                    ));
                }
                "battery" => {
                    let mut statuses = [
                        battery::Status {
                            prefix: "".to_string(),
                            suffix: "".to_string(),
                        },
                        battery::Status {
                            prefix: "".to_string(),
                            suffix: "".to_string(),
                        },
                        battery::Status {
                            prefix: "".to_string(),
                            suffix: "".to_string(),
                        },
                    ];
                    let status_root = items.get(&Yaml::String("statuses".to_string())).unwrap();
                    if let Yaml::Hash(ref status_hash) = *status_root {
                        for (kk, vv) in status_hash {
                            let index = match kk.as_str().expect("should be a string").as_ref() {
                                "online" => 1,
                                "full" => 2,
                                _ => 0,
                            };
                            if let Yaml::Hash(ref hash) = *vv {
                                statuses[index].prefix = hash.get(
                                    &Yaml::String("prefix".to_string()),
                                ).unwrap_or(&Yaml::String("".to_string()))
                                    .as_str()
                                    .expect("should be  a string")
                                    .to_string();
                                statuses[index].suffix = hash.get(
                                    &Yaml::String("suffix".to_string()),
                                ).unwrap_or(&Yaml::String("".to_string()))
                                    .as_str()
                                    .expect("should be  a string")
                                    .to_string();
                            }
                        }
                    }

                    return Box::new(battery::Block::new(
                        base,
                        items[&Yaml::String("sensor".to_string())]
                            .as_str()
                            .expect("battery::sensor must be defined")
                            .to_string(),
                        statuses,
                    ));
                }
                "filesystem" => {
                    return Box::new(filesystem::Block::new(
                        base,
                        items[&Yaml::String("path".to_string())]
                            .as_str()
                            .expect("filesystem::path must be defined")
                            .to_string(),
                    ));
                }
                "cpuload" => {
                    return Box::new(cpuload::Block::new(base));
                }

                "memory" => {
                    return Box::new(memory::Block::new(base));
                }

                "time" => {
                    return Box::new(time::Block::new(
                        base,
                        items[&Yaml::String("format".to_string())]
                            .as_str()
                            .unwrap_or("%d.%m.%Y %H:%M")
                            .to_string(),
                    ));
                }
                "wifi" => {
                    return Box::new(wifi::Block::new(
                        base,
                        items[&Yaml::String("interface".to_string())]
                            .as_str()
                            .unwrap_or("wlan0")
                            .to_string(),
                    ));
                }

                "volume" => {
                    let mut extra = vec![];
                    if let Yaml::Array(ref yextra) =
                        items[&Yaml::String("prefix_extras".to_string())]
                    {
                        for item in yextra {
                            extra.push(item.as_str().unwrap().to_string());
                        }
                    }
                    return Box::new(volume::Block::new(
                        base,
                        items[&Yaml::String("mixer".to_string())]
                            .as_str()
                            .unwrap_or("Master")
                            .to_string(),
                        extra,
                    ));
                }
                _ => {
                    return Box::new(custom::Block::new(
                        base,
                        items[&Yaml::String("command".to_string())]
                            .as_str()
                            .unwrap_or("echo invalid")
                            .to_string(),
                        items[&Yaml::String("retry".to_string())].as_i64().expect(
                            "retry value must be integer",
                        ) as
                            u32,
                    ));

                }
            }
        }
    }
    Box::new(undefined::Block::new(base))
}

pub fn init_from(config_path: &std::path::Path) -> Vec<Box<block::Block>> {
    let mut blocks = Vec::new();

    let data = utility::read_file(config_path.to_str().unwrap()).expect("cannot read config file");

    let config = yaml_rust::yaml::YamlLoader::load_from_str(&data).expect("couldn't parse config");
    if !config.is_empty() {
        if let Yaml::Hash(ref h) = config[0] {
            let root = h.get(&Yaml::String("config".to_string())).expect(
                "cannot find config section",
            );
            if let Yaml::Array(ref sections) = *root {
                for section in sections {
                    blocks.push(create(section));
                }
            }
        }
    }
    blocks
}
