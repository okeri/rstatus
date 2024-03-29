use alsa::{
    ctl::ElemId,
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
                name: card.to_owned(),
                mixer,
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
}

impl SoundService for AlsaDevice {
    fn id(&self) -> String {
        "alsa".to_owned()
    }

    fn sink_name(&self) -> String {
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
