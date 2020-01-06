/*
  status bar for tiling wms like i3, sway, etc...
  Copyright (C) 2019 Oleg Keri

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

use super::base::{Base, Value};
use super::block;
use serde::Deserialize;
use alsa::{Mixer, mixer::{SelemId, SelemChannelId}};

#[derive(Deserialize)]
pub struct Block {
    #[serde(flatten)]
    base: Base,
    mixer: String,
    #[serde(default = "default_card")]
    card: String,
    #[serde(default = "empty_extras")]
    prefix_extras: Vec<String>,
}

enum MixerValue {
    Off,
    Volume(u32),
}

impl MixerValue {
    pub fn to_value(&self) -> Value {
        match &self {
            MixerValue::Volume(vol) => Value::new(*vol),
            _ => Value::Invalid,
        }
    }

    pub fn ok(&self) -> bool {
        match &self {
            MixerValue::Volume(_) => true,
            _ => false,
        }
    }
}

fn empty_extras() -> Vec<String> {
    Vec::new()
}

fn default_card() -> String {
    "default".to_owned()
}

impl Block {
    fn mixer_value(&self, mixer: &str) -> MixerValue {
	let mixer_device = Mixer::new(&self.card, false).ok();
	if let Some(device) = &mixer_device {
	    let id = SelemId::new(mixer, 0);
	    if let Some(selem) = device.find_selem(&id) {
		if selem.has_playback_switch() {
		    if let Ok(value) = selem.get_playback_switch(SelemChannelId::FrontLeft)  {
			if value == 0 {
			    return MixerValue::Off
			}
		    }
		}
		let (min, max) = selem.get_playback_volume_range();
		if let Ok(value) = selem.get_playback_volume(SelemChannelId::FrontLeft)
		{
		    return MixerValue::Volume(((value - min) * 100 / (max - min)) as u32)
		}
	    }
	}
	MixerValue::Off
    }
}

impl block::Block for Block {
    impl_Block!();
   
    fn update(&mut self) {
        self.base.value = if let MixerValue::Volume(volume) = self.mixer_value("Master")
        {
            if self.prefix_extras.len() > 1 {
                if !self.mixer_value("Speaker").ok()
                    && self.mixer_value("Headphone").ok()
                {
                    self.base.set_prefix(&self.prefix_extras[0]);
                } else {
                    self.base.set_prefix(&self.prefix_extras[1]);
                }
            }
            if self.mixer != "Master" {
                self.mixer_value(&self.mixer).to_value()
            } else {
                Value::new(volume)
            }
        } else {
            Value::Invalid
        }
    }
}
