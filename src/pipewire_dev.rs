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

use std::{
    cell::RefCell,
    collections::HashMap,
    rc::Rc,
    sync::{Arc, Mutex},
    thread,
};

use pipewire as pw;
use pw::metadata::Metadata;
use pw::node::Node;
use pw::proxy::{Listener, ProxyListener, ProxyT};
use pw::spa::dict::{ForeignDict, ReadableDict};
use pw::spa::param::ParamType;
use pw::spa::pod::{deserialize::PodDeserializer, Value, ValueArray};
use pw::types::ObjectType;

#[derive(Copy, Clone)]
struct Volume {
    mute: bool,
    volume: u32,
}

impl Volume {
    fn new() -> Self {
        Volume {
            mute: true,
            volume: 0,
        }
    }
}

struct Proxy<T> {
    proxy: T,
    listeners: Vec<Box<dyn Listener>>,
}

impl<T> Proxy<T> {
    fn new(proxy: T, listener: Box<dyn Listener>) -> Self {
        Proxy {
            proxy,
            listeners: vec![listener],
        }
    }

    fn add_listener(&mut self, listener: ProxyListener) {
        self.listeners.push(Box::new(listener));
    }

    fn proxy(&self) -> &T {
        &self.proxy
    }
}

struct Sink {
    name: String,
    card_name: String,
    volume: Volume,
}

impl Sink {
    fn new(name: String, card_name: String) -> Self {
        Sink {
            name,
            card_name,
            volume: Volume::new(),
        }
    }
}

struct PipewireCache {
    pub sink_name: String,
    pub volume: Option<u32>,
    pub jack_plugged: Option<bool>,
}

impl PipewireCache {
    pub fn new() -> PipewireCache {
        PipewireCache {
            sink_name: "".to_owned(),
            volume: None,
            jack_plugged: None,
        }
    }

    fn update_vol(&mut self, volume: Volume) {
        self.volume = if volume.mute {
            None
        } else {
            Some(volume.volume)
        };
    }

    fn update_name(&mut self, name: String) {
        self.sink_name = name;
    }
}

struct MonitorData {
    block_index: usize,
    nodes: HashMap<u32, Proxy<Node>>,
    sinks: HashMap<u32, Sink>,
    cache: Arc<Mutex<PipewireCache>>,
    meta: Option<Proxy<Metadata>>,
    def_sink: Option<String>,
    def_sink_id: u32,
    sink_id: u32,
}

impl MonitorData {
    fn new(block_index: usize, cache: Arc<Mutex<PipewireCache>>) -> Self {
        Self {
            block_index,
            nodes: HashMap::new(),
            sinks: HashMap::new(),
            cache,
            meta: None,
            def_sink: None,
            def_sink_id: 0,
            sink_id: 0,
        }
    }

    fn add_node(&mut self, node: Node, listener: Box<dyn Listener>) {
        self.nodes
            .insert(node.upcast_ref().id(), Proxy::<Node>::new(node, listener));
    }

    fn add_node_listener(&mut self, proxy_id: u32, listener: ProxyListener) {
        self.nodes
            .entry(proxy_id)
            .and_modify(|node| node.add_listener(listener));
    }

    fn set_meta(&mut self, meta: Metadata, listener: Box<dyn Listener>) {
        self.meta = Some(Proxy::<Metadata>::new(meta, listener));
    }

    fn remove_sink(&mut self, id: u32) {
        self.sinks.remove(&id);
    }

    fn remove_node(&mut self, proxy_id: u32) {
        self.nodes.remove(&proxy_id);
    }

    fn set_default_sink(&mut self, sink_data: String) {
        let v: Vec<&str> = sink_data.split('"').collect();
        if v.len() > 2 {
            self.def_sink = Some(v[v.len() - 2].to_string());
        }
        if let Some(def_sink) = self.def_sink.as_ref() {
            if let Some(sink) = self.sinks.iter().find(|s| &s.1.name == def_sink) {
                self.def_sink_id = *sink.0;
                if let Ok(mut cache) = self.cache.lock() {
                    cache.update_name(sink.1.card_name.to_owned());
                    cache.update_vol(sink.1.volume);
                }
                update_by_index(self.block_index);
            }
        }
    }

    fn update_sink(&mut self, sink_name: Option<&str>, card_name: Option<&str>, id: u32) {
        if let Some(name) = sink_name {
            let cache = &self.cache;
            self.sinks
                .entry(id)
                .and_modify(|s| s.name = name.to_string())
                .or_insert_with(|| {
                    if let Some(card) = card_name {
                        cache.lock().unwrap().update_name(card.to_owned());
                    }
                    Sink::new(name.to_string(), card_name.unwrap().to_owned())
                });
            if let Some(ref def_sink) = self.def_sink {
                if def_sink == name {
                    self.def_sink_id = id;
                }
            }
        }
    }

    fn update_volume(&mut self, volume: Volume) {
        self.sinks.entry(self.sink_id).and_modify(|s| {
            s.volume = volume;
        });
        if self.sink_id == self.def_sink_id {
            self.cache.lock().unwrap().update_vol(volume);
            update_by_index(self.block_index);
        }
    }
}

pub struct PipewireDevice {
    cache: Arc<Mutex<PipewireCache>>,
}

impl PipewireDevice {
    pub fn new(block_index: usize) -> Option<PipewireDevice> {
        pw::init();
        let cache = Arc::new(Mutex::new(PipewireCache::new()));
        let cache_clone = cache.clone();
        thread::spawn(move || {
            let monitor = Rc::new(RefCell::new(MonitorData::new(block_index, cache_clone)));
            let main_loop = pw::MainLoop::new().expect("cannot create loop");

            let context = pw::Context::new(&main_loop).unwrap();
            let core = context.connect(None).unwrap();

            let registry = Arc::new(core.get_registry().unwrap());
            let registry_weak = Arc::downgrade(&registry);

            let mon_data = monitor.clone();

            let check_prop = |props: &Option<ForeignDict>, key: &str, value: &str| -> bool {
                let mut result = false;
                if let Some(p) = props.as_ref() {
                    if let Some(c) = p.get(key) {
                        result = c == value;
                    }
                }
                result
            };
            let _reg = registry
                .add_listener_local()
                .global(move |obj| {
                    if let Some(registry) = registry_weak.upgrade() {
                        match obj.type_ {
                            ObjectType::Node => {
                                if !check_prop(&obj.props, "media.class", "Audio/Sink") {
                                    return;
                                }
                                let node: Node = registry.bind(obj).unwrap();
                                let mon_weak = Rc::downgrade(&mon_data);
                                let mon_info_weak = Rc::downgrade(&mon_data);
                                let mon_weak_params = Rc::downgrade(&mon_data);
                                let proxy_id = node.upcast_ref().id();
                                let obj_listener = node
                                    .add_listener_local()
                                    .info(move |info| {
                                        if let Some(mon) = mon_info_weak.upgrade() {
                                            if let Some(props) = info.props() {
                                                mon.borrow_mut().update_sink(
                                                    props.get("node.name"),
                                                    props.get("node.nick"),
                                                    info.id(),
                                                );
                                            }
                                            mon.borrow_mut().sink_id = info.id();
                                            if let Some(node) =
                                                mon.borrow_mut().nodes.get_mut(&proxy_id)
                                            {
                                                node.proxy().enum_params(
                                                    0,
                                                    Some(ParamType::Props),
                                                    0,
                                                    4,
                                                )
                                            }
                                        }
                                    })
                                    .param(move |_seq, _id, _index, _next, param| {
                                        if let Ok(props) =
                                            PodDeserializer::deserialize_any_from(param)
                                        {
                                            let mut volume: Option<Volume> = None;
                                            if let Value::Object(o) = props.1 {
                                                for i in o.properties.iter() {
                                                    if i.key == 0x10004 {
                                                        if let Value::Bool(mute) = i.value {
                                                            volume
                                                                .get_or_insert(Volume::new())
                                                                .mute = mute;
                                                        }
                                                    } else if i.key == 0x10008 {
                                                        if let Value::ValueArray(
                                                            ValueArray::Float(volumes),
                                                        ) = &i.value
                                                        {
                                                            let channel_volume =
                                                                (((volumes[0] + volumes[1]) / 2.)
                                                                    .cbrt()
                                                                    * 100.)
                                                                    as u32;
                                                            volume
                                                                .get_or_insert(Volume::new())
                                                                .volume = channel_volume;
                                                        }
                                                    }
                                                }
                                                if let Some(vol) = volume {
                                                    if let Some(mon) = mon_weak_params.upgrade() {
                                                        mon.borrow_mut().update_volume(vol)
                                                    }
                                                }
                                            }
                                        }
                                    })
                                    .register();
                                let proxy_id = node.upcast_ref().id();
                                let listener = node
                                    .upcast_ref()
                                    .add_listener_local()
                                    .removed(move || {
                                        if let Some(monp) = mon_weak.upgrade() {
                                            monp.borrow_mut().remove_node(proxy_id);
                                        }
                                    })
                                    .register();
                                mon_data.borrow_mut().add_node(node, Box::new(obj_listener));
                                mon_data.borrow_mut().add_node_listener(proxy_id, listener);
                            }
                            ObjectType::Metadata => {
                                if !check_prop(&obj.props, "metadata.name", "default") {
                                    return;
                                }
                                let metadata: Metadata = registry.bind(obj).unwrap();
                                let mon_weak_params = Rc::downgrade(&mon_data);
                                let obj_listener = metadata
                                    .add_listener_local()
                                    .property(move |_subject, key, _type, value| {
                                        if let Some(k) = key {
                                            if k == "default.audio.sink" {
                                                if let Some(mon) = mon_weak_params.upgrade() {
                                                    mon.borrow_mut().set_default_sink(
                                                        value.unwrap().to_string(),
                                                    );
                                                }
                                            }
                                        }
                                        0
                                    })
                                    .register();
                                mon_data
                                    .borrow_mut()
                                    .set_meta(metadata, Box::new(obj_listener));
                            }
                            _ => {}
                        };
                    }
                })
                .global_remove(move |id| {
                    monitor.borrow_mut().remove_sink(id);
                })
                .register();
            main_loop.run();
        });
        Some(PipewireDevice { cache })
    }
}

impl Drop for PipewireDevice {
    fn drop(&mut self) {
        unsafe {
            pw::deinit();
        }
    }
}

impl SoundService for PipewireDevice {
    fn id(&self) -> String {
        "Pipewire".to_owned()
    }

    fn sink_name(&self) -> String {
        self.cache.lock().unwrap().sink_name.clone()
    }

    fn listen(&mut self, _block_index: usize) {}

    fn volume(&self, _mixer: &str) -> Option<u32> {
        self.cache.lock().unwrap().volume
    }

    fn jack_plugged(&self) -> Option<bool> {
        self.cache.lock().unwrap().jack_plugged
    }

    fn exists(&self, _mixer: &str) -> bool {
        true
    }

    fn update(&self) {}

    fn set_mute(&self, _mixer: &str, _value: bool) {}
}
