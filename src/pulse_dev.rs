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

use super::blocks::update_by_index;
use super::sound_service::SoundService;
use libpulse_binding::{
    callbacks,
    context::{self, introspect, subscribe::InterestMaskSet, Context, FlagSet},
    def::PortAvailable,
    mainloop::threaded::Mainloop,
    volume::Volume,
};
use std::cell::RefCell;
use std::rc::Rc;
use std::str::from_utf8;
struct PulseCache {
    pub sink_name: String,
    pub volume: Option<u32>,
    pub jack_plugged: Option<bool>,
}

impl PulseCache {
    pub fn new() -> PulseCache {
        PulseCache {
            sink_name: "".to_owned(),
            volume: None,
            jack_plugged: None,
        }
    }

    fn update(&mut self, info: &introspect::SinkInfo) {
        if let Some(bytes) = info.proplist.get("alsa.card_name") {
            self.sink_name = from_utf8(bytes)
                .unwrap_or("")
                .trim_end_matches(char::from(0))
                .to_owned();
        }
        if !info.mute {
            self.volume = Some(info.volume.avg().0 * 100 / Volume::NORMAL.0);
        } else {
            self.volume = None;
        }

        let mut jack_present = false;
        for port in info.ports.iter() {
            match port.available {
                PortAvailable::Yes | PortAvailable::No => {
                    jack_present = true;
                    break;
                }
                _ => {}
            }
        }

        if jack_present {
            if let Some(ref port) = info.active_port {
                self.jack_plugged = match port.available {
                    PortAvailable::Yes => Some(true),
                    _ => Some(false),
                };
            }
        } else {
            self.jack_plugged = None;
        }
    }
}

pub struct PulseDevice {
    dispatcher: Rc<RefCell<Mainloop>>,
    context: Rc<RefCell<Context>>,
    cache: Rc<RefCell<PulseCache>>,
}

impl PulseDevice {
    pub fn new() -> Option<PulseDevice> {
        if let Some(d) = Mainloop::new() {
            if let Some(c) = Context::new(&d, "rstatus") {
                let dispatcher = Rc::new(RefCell::new(d));
                let context = Rc::new(RefCell::new(c));
                let disp_ref = Rc::clone(&dispatcher);
                let context_ref = Rc::clone(&context);

                context
                    .borrow_mut()
                    .set_state_callback(Some(Box::new(move || {
                        let state = unsafe { (*context_ref.as_ptr()).get_state() };
                        match state {
                            context::State::Ready
                            | context::State::Failed
                            | context::State::Terminated => unsafe {
                                (*disp_ref.as_ptr()).signal(false);
                            },
                            _ => {}
                        }
                    })));
                if context
                    .borrow_mut()
                    .connect(None, FlagSet::NOAUTOSPAWN, None)
                    .is_ok()
                {
                    return Some(PulseDevice {
                        dispatcher,
                        context,
                        cache: Rc::new(RefCell::new(PulseCache::new())),
                    });
                }
            }
        }
        return None;
    }

    fn update_cache<'a>(
        context: &'a Rc<RefCell<Context>>,
        cache: &'a Rc<RefCell<PulseCache>>,
        block_index: usize,
    ) {
        let context_ref = Rc::clone(&context);
        let cache_ref = Rc::clone(&cache);
        context
            .borrow_mut()
            .introspect()
            .get_server_info(move |si| {
                if let Some(ref def) = si.default_sink_name {
                    let cache_ref2 = Rc::clone(&cache_ref);
                    context_ref.borrow_mut().introspect().get_sink_info_by_name(
                        &def.to_owned().to_string(),
                        move |def| {
                            if let callbacks::ListResult::Item(sink) = def {
                                cache_ref2.borrow_mut().update(sink);
                                update_by_index(block_index);
                            }
                        },
                    );
                }
            });
    }
}

impl SoundService for PulseDevice {
    fn id(&self) -> String {
        "PulseAudio".to_owned()
    }

    fn sink_name(&self) -> String {
        self.cache.borrow_mut().sink_name.clone()
    }

    fn listen(&mut self, block_index: usize) {
        self.dispatcher.borrow_mut().lock();
        self.dispatcher.borrow_mut().start().unwrap();
        loop {
            match self.context.borrow_mut().get_state() {
                context::State::Ready => {
                    break;
                }
                context::State::Failed | context::State::Terminated => {
                    self.dispatcher.borrow_mut().unlock();
                    self.dispatcher.borrow_mut().stop();
                    return;
                }
                _ => {
                    self.dispatcher.borrow_mut().wait();
                }
            }
        }
        self.dispatcher.borrow_mut().unlock();
        self.context.borrow_mut().set_state_callback(None);
        let context = Rc::clone(&self.context);
        let cache = Rc::clone(&self.cache);

        PulseDevice::update_cache(&context, &cache, block_index);
        self.context
            .borrow_mut()
            .set_subscribe_callback(Some(Box::new(move |_, _, _| {
                PulseDevice::update_cache(&context, &cache, block_index);
            })));
        self.context
            .borrow_mut()
            .subscribe(InterestMaskSet::SERVER | InterestMaskSet::SINK, |_| {});
    }

    fn volume(&self, _mixer: &str) -> Option<u32> {
        self.cache.borrow_mut().volume
    }

    fn jack_plugged(&self) -> Option<bool> {
        self.cache.borrow_mut().jack_plugged
    }

    fn exists(&self, mixer: &str) -> bool {
        mixer == "Master"
    }

    fn update(&self) {}

    fn set_mute(&self, _mixer: &str, _value: bool) {}
}
