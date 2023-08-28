#[cfg(feature = "alsa")]
use super::alsa_dev::AlsaDevice;

#[cfg(feature = "pulse")]
use super::pulse_dev::PulseDevice;

#[cfg(feature = "pipewire")]
use super::pipewire_dev::PipewireDevice;

use super::base::{default_false, Base, Value};
use super::block;
use super::sound_service::SoundService;
use serde::Deserialize;

macro_rules! impl_volume_stub {
    ($dev_name:ident) => {
        struct $dev_name {}

        impl SoundService for $dev_name {
            fn id(&self) -> String {
                "unimplemented".to_owned()
            }

            fn sink_name(&self) -> String {
                "".to_owned()
            }

            fn listen(&mut self, _block_index: usize) {}

            fn volume(&self, _mixer: &str) -> Option<u32> {
                None
            }

            fn jack_plugged(&self) -> Option<bool> {
                None
            }

            fn exists(&self, _mixer: &str) -> bool {
                false
            }

            fn update(&self) {}

            fn set_mute(&self, _mixer: &str, _value: bool) {}
        }
    };
}

#[cfg(not(feature = "alsa"))]
impl_volume_stub!(AlsaDevice);

#[cfg(not(feature = "alsa"))]
impl AlsaDevice {
    pub fn new(_card: &str) -> Option<AlsaDevice> {
        None
    }
}
#[cfg(not(feature = "pulse"))]
impl_volume_stub!(PulseDevice);

#[cfg(not(feature = "pulse"))]
impl PulseDevice {
    pub fn new() -> Option<PulseDevice> {
        None
    }
}

#[cfg(not(feature = "pipewire"))]
impl_volume_stub!(PipewireDevice);

#[cfg(not(feature = "pipewire"))]
impl PipewireDevice {
    pub fn new() -> Option<PipewireDevice> {
        None
    }
}

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

            if cfg!(feature = "alsa") && service.id() == "alsa" && self.jack_plugged != jack_plugged
            {
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

            self.jack_plugged = jack_plugged;

            self.base.value = if self.master_exists {
                if self.jack_icons.len() > 1 {
                    if self.jack_plugged || self.jack_only.contains(&service.sink_name()) {
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
            if cfg!(feature = "pipewire") {
                if let Some(pipewire) = PipewireDevice::new() {
                    self.service = Some(Box::from(pipewire));
                }
            }
            if self.service.is_none() && cfg!(feature = "pulse") {
                if let Some(pulse) = PulseDevice::new() {
                    self.service = Some(Box::from(pulse));
                }
            }
            if self.service.is_none() && cfg!(feature = "alsa") {
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
