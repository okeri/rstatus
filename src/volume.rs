/*
  status bar for tiling wms like i3, sway, etc...
  Copyright (C) 2020 Oleg Keri

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

use super::alsa_dev::AlsaDevice;
use super::base::{default_false, Base, Value};
use super::block;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Block {
    #[serde(flatten)]
    base: Base,
    mixer: String,
    #[serde(default = "default_card")]
    card: String,
    #[serde(default = "empty_extras")]
    prefix_extras: Vec<String>,
    #[serde(default = "default_false")]
    jack_switch_outputs: bool,
    #[serde(default = "default_false")]
    jack_mute_on_unplug: bool,
    #[serde(default = "default_false")]
    jack_unmute_on_plug: bool,
    #[serde(skip, default = "default_false")]
    jack_plugged: bool,
    #[serde(skip, default = "default_false")]
    master_exists: bool,
    #[serde(skip, default = "default_none")]
    dev: Option<AlsaDevice>,
}

fn empty_extras() -> Vec<String> {
    Vec::new()
}

fn default_card() -> String {
    "default".to_owned()
}

fn default_none() -> Option<AlsaDevice> {
    None
}

impl Block {
    fn jack_plug_check(&mut self) {
        if let Some(dev) = &self.dev {
            if let Some(plugged) = dev.jack_plugged() {
                if self.jack_plugged != plugged {
                    self.jack_plugged = plugged;
                    if plugged {
                        if self.jack_switch_outputs {
                            dev.set_mute("Speaker", true);
                            dev.set_mute("Headphone", false);
                        }
                        if self.jack_unmute_on_plug {
                            dev.set_mute("Master", false);
                        }
                    } else {
                        if self.jack_switch_outputs {
                            dev.set_mute("Speaker", false);
                            dev.set_mute("Headphone", true);
                        }
                        if self.jack_mute_on_unplug {
                            dev.set_mute("Master", true);
                        }
                    }
                }
            }
        }
    }
}

impl block::Block for Block {
    impl_Block!();

    fn update(&mut self) {
        if self.dev.is_none() {
            self.dev = AlsaDevice::new(&self.card);
            if let Some(dev) = &self.dev {
                if let Some(plugged) = dev.jack_plugged() {
                    self.jack_plugged = plugged;
                }
                self.master_exists = dev.exists("Master");
                let signal = self.signal();
                if signal != 0 {
                    dev.listen(&self.card, signal as i32);
                }
            }
        } else {
            if let Some(dev) = &self.dev {
                dev.update();
            }
        }
        self.jack_plug_check();
        if let Some(dev) = &self.dev {
            self.base.value = if self.master_exists {
                if self.prefix_extras.len() > 1 {
                    if dev.volume("Speaker").is_none() && dev.volume("Headphone").is_some() {
                        self.base.set_prefix(&self.prefix_extras[0]);
                    } else {
                        self.base.set_prefix(&self.prefix_extras[1]);
                    }
                }

                if let Some(volume) = dev.volume("Master") {
                    if self.mixer != "Master" {
                        Value::new(dev.volume(&self.mixer))
                    } else {
                        Value::new(volume)
                    }
                } else {
                    Value::Invalid
                }
            } else {
                Value::new(dev.volume(&self.mixer))
            }
        } else {
            self.base.value = Value::Invalid
        }
    }
}
