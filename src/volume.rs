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
use super::pulse_dev::PulseDevice;
use super::sound_service::SoundService;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Block {
    #[serde(flatten)]
    base: Base,
    #[serde(default = "default_mixer")]
    mixer: String,
    #[serde(default = "default_card")]
    card: String,
    #[serde(default = "empty_jack_icons")]
    jack_icons: Vec<String>,
    #[serde(default = "empty_jack_only")]
    jack_only: Vec<String>,
    #[serde(default = "default_false")]
    alsa_jack_switch_outputs: bool,
    #[serde(default = "default_false")]
    alsa_jack_mute_on_unplug: bool,
    #[serde(default = "default_false")]
    alsa_jack_unmute_on_plug: bool,
    #[serde(skip, default = "default_false")]
    jack_plugged: bool,
    #[serde(skip, default = "default_false")]
    master_exists: bool,
    #[serde(skip, default = "default_service")]
    service: Option<Box<dyn SoundService>>,
}

fn empty_jack_icons() -> Vec<String> {
    Vec::new()
}

fn empty_jack_only() -> Vec<String> {
    Vec::new()
}

fn default_card() -> String {
    "default".to_owned()
}

fn default_mixer() -> String {
    "PCM".to_owned()
}

fn default_service() -> Option<Box<dyn SoundService>> {
    None
}

impl Block {
    fn handle_events(&mut self) {
        if let Some(ref service) = self.service {
            service.update();
            let jack_plugged = if let Some(plugged) = service.jack_plugged() {
                plugged
            } else {
                false
            };

            if service.id() != "PulseAudio" {
                if self.jack_plugged != jack_plugged {
                    if jack_plugged {
                        if self.alsa_jack_switch_outputs {
                            service.set_mute("Speaker", true);
                            service.set_mute("Headphone", false);
                        }
                        if self.alsa_jack_unmute_on_plug {
                            service.set_mute("Master", false);
                        }
                    } else {
                        if self.alsa_jack_switch_outputs {
                            service.set_mute("Speaker", false);
                            service.set_mute("Headphone", true);
                        }
                        if self.alsa_jack_mute_on_unplug {
                            service.set_mute("Master", true);
                        }
                    }
                }
            }

            self.jack_plugged = jack_plugged;

            self.base.value = if self.master_exists {
                if self.jack_icons.len() > 1 {
                    if self.jack_plugged ||
			self.jack_only.contains(&service.sink_name()) {
                        self.base.set_prefix(&self.jack_icons[0]);
                    } else {
                        self.base.set_prefix(&self.jack_icons[1]);
                    }
                }

                if let Some(volume) = service.volume("Master") {
                    if self.mixer != "Master" {
                        Value::new(service.volume(&self.mixer))
                    } else {
                        Value::new(volume)
                    }
                } else {
                    Value::Invalid
                }
            } else {
                Value::new(service.volume(&self.mixer))
            }
        }
    }
}

impl block::Block for Block {
    impl_Block!();
    fn update(&mut self) {
        if self.service.is_none() {
	    if let Some(pulse) = PulseDevice::new() {
                self.service = Some(Box::from(pulse));
            } else {
                if let Some(alsa) = AlsaDevice::new(&self.card) {
                    self.service = Some(Box::from(alsa));
                }
            }
            if let Some(ref mut service) = self.service {
                if let Some(plugged) = service.jack_plugged() {
                    self.jack_plugged = plugged;
                }
                self.master_exists = service.exists("Master");
                service.listen(self.base.index());
            }
        }
        self.handle_events();
    }
}
