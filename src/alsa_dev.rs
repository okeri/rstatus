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

use alsa::{
    ctl::{Ctl, ElemId},
    hctl::HCtl,
    mixer::{Mixer, SelemChannelId, SelemId},
};

use super::blocks::update_by_index;
use super::sound_service::SoundService;
use std::thread;

pub struct AlsaDevice {
    name: String,
    mixer: Mixer,
    hctl: Option<HCtl>,
    jack: Option<ElemId>,
}

impl AlsaDevice {
    fn find_jack(hctl: &HCtl) -> Option<ElemId> {
        hctl.load().ok()?;
        for b in hctl.elem_iter() {
            if let Ok(id) = b.get_id() {
                if let Ok(name) = id.get_name() {
                    if name.ends_with(" Jack") && !name.ends_with(" Phantom Jack") {
                        return Some(id);
                    }
                }
            }
        }
        None
    }

    pub fn new(card: &str) -> Option<AlsaDevice> {
        if let Ok(mixer) = Mixer::new(card, false) {
            let mut result = AlsaDevice {
                name: AlsaDevice::card_name(card),
                mixer: mixer,
                hctl: HCtl::new(card, false).ok(),
                jack: None,
            };
            if let Some(ref hctl) = result.hctl {
                result.jack = AlsaDevice::find_jack(hctl);
            }
            Some(result)
        } else {
            None
        }
    }

    pub fn card_name(card_name: &str) -> String {
        if let Ok(card) = Ctl::new(card_name, false) {
            if let Ok(info) = card.card_info() {
                let result = info.get_name().unwrap_or(card_name);
                return result.to_owned();
            }
        }
        card_name.to_owned()
    }
}

impl SoundService for AlsaDevice {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn listen(&mut self, block_index: usize) {
        if let Ok(mixer) = Mixer::new(&self.name, false) {
            thread::spawn(move || loop {
                if mixer.wait(None).is_ok() && mixer.handle_events().is_ok() {
                    update_by_index(block_index);
                }
            });
        }
    }

    fn volume(&self, mixer: &str) -> Option<u32> {
        let id = SelemId::new(mixer, 0);
        let selem = self.mixer.find_selem(&id)?;
        if selem.has_playback_switch() {
            if let Ok(value) = selem.get_playback_switch(SelemChannelId::FrontLeft) {
                if value == 0 {
                    return None;
                }
            }
        }
        let (min, max) = selem.get_playback_volume_range();
        selem
            .get_playback_volume(SelemChannelId::FrontLeft)
            .ok()
            .map(|value| ((value - min) * 100 / (max - min)) as u32)
    }

    fn jack_plugged(&self) -> Option<bool> {
        if let Some(hctl) = &self.hctl {
            if let Some(ref id) = self.jack {
                let jack = hctl.find_elem(id)?;
                let value = jack.read().ok()?;
                return value.get_boolean(0);
            }
        }
        None
    }

    fn exists(&self, mixer: &str) -> bool {
        let id = SelemId::new(mixer, 0);
        self.mixer.find_selem(&id).is_some()
    }

    fn set_mute(&self, mixer: &str, value: bool) {
        let id = SelemId::new(mixer, 0);
        if let Some(selem) = self.mixer.find_selem(&id) {
            if selem.has_playback_switch() {
                selem
                    .set_playback_switch_all(if value { 0 } else { 1 })
                    .unwrap_or(());
            }
        }
    }

    fn update(&self) {
        self.mixer.handle_events().unwrap_or(0);
    }
}
