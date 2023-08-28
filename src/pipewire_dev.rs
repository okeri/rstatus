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
use pw::proxy::{Listener, ProxyT};
use pw::registry::{GlobalObject, Registry};
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

struct Proxy<T> {
    proxy: T,
    listeners: Vec<Box<dyn Listener>>,
}

impl<T> Proxy<T> {
    fn new(proxy: T) -> Self {
        Proxy {
            proxy,
            listeners: vec![],
        }
    }

    fn proxy(&self) -> &T {
        &self.proxy
    }

    fn listener(mut self, listener: Box<dyn Listener>) -> Self {
        self.listeners.push(listener);
        Proxy { ..self }
    }
}

struct Sink {
    name: String,
    card_name: String,
    volume: Volume,
    node: Proxy<Node>,
}

impl Sink {
    fn new(name: String, card_name: String, node: Proxy<Node>) -> Self {
        Sink {
            name,
            card_name,
            volume: Volume::new(),
            node,
        }
    }
}

struct MonitorData {
    block_index: usize,
    sinks: HashMap<u32, Sink>,
    cache: Arc<Mutex<PipewireCache>>,
    meta: Option<Proxy<Metadata>>,
    def_sink: Option<String>,
    def_sink_id: u32,
}

struct Monitor {
    data: Rc<RefCell<MonitorData>>,
}

impl MonitorData {
    fn new(block_index: usize, cache: Arc<Mutex<PipewireCache>>) -> Self {
        Self {
            block_index,
            sinks: HashMap::new(),
            cache,
            meta: None,
            def_sink: None,
            def_sink_id: 0,
        }
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

    fn set_def_sink_id_maybe(&mut self, id: u32, sink_name: &str) {
        if let Some(def) = &self.def_sink {
            if sink_name == def {
                self.def_sink_id = id;
            }
        }
    }

    fn update_volume(&mut self, id: u32, volume: Volume) {
        self.sinks.entry(id).and_modify(|s| {
            s.volume = volume;
        });
        if id == self.def_sink_id {
            self.cache.lock().unwrap().update_vol(volume);
            update_by_index(self.block_index);
        }
    }
}

impl Monitor {
    fn new(block_index: usize, cache: Arc<Mutex<PipewireCache>>) -> Self {
        Monitor {
            data: Rc::new(RefCell::new(MonitorData::new(block_index, cache))),
        }
    }

    fn add_sink(&self, registry: Arc<Registry>, sink_obj: &GlobalObject<ForeignDict>) {
        let node: Node = registry.bind(sink_obj).unwrap();
        let mon_weak = Rc::downgrade(&self.data);
        let mon_weak_info = Rc::downgrade(&self.data);
        let mon_weak_param = Rc::downgrade(&self.data);
        let id = node.upcast_ref().id();
        let node_listener = Box::new(
            node.add_listener_local()
                .info(move |_info| {
                    if let Some(data) = mon_weak_info.upgrade() {
                        if let Some(sink) = data.borrow_mut().sinks.get_mut(&id) {
                            sink.node
                                .proxy()
                                .enum_params(0, Some(ParamType::Props), 0, 4)
                        }
                    }
                })
                .param(move |_seq, _id, _index, _next, param| {
                    if let Ok(props) = PodDeserializer::deserialize_any_from(param) {
                        let mut volume: Option<Volume> = None;
                        if let Value::Object(o) = props.1 {
                            for i in o.properties.iter() {
                                if i.key == 0x10004 {
                                    if let Value::Bool(mute) = i.value {
                                        volume.get_or_insert(Volume::new()).mute = mute;
                                    }
                                } else if i.key == 0x10008 {
                                    if let Value::ValueArray(ValueArray::Float(volumes)) = &i.value
                                    {
                                        let channel_volume =
                                            (((volumes[0] + volumes[1]) / 2.).cbrt() * 100.) as u32;
                                        volume.get_or_insert(Volume::new()).volume = channel_volume;
                                    }
                                }
                            }
                            if let Some(vol) = volume {
                                if let Some(data) = mon_weak_param.upgrade() {
                                    data.borrow_mut().update_volume(id, vol)
                                }
                            }
                        }
                    }
                })
                .register(),
        );

        let rm_listener = Box::new(
            node.upcast_ref()
                .add_listener_local()
                .removed(move || {
                    if let Some(monp) = mon_weak.upgrade() {
                        monp.borrow_mut().sinks.remove(&id);
                    }
                })
                .register(),
        );
        let node_proxy = Proxy::<Node>::new(node)
            .listener(node_listener)
            .listener(rm_listener);
        let ref props = sink_obj.props.as_ref().unwrap();
        let sink = Sink::new(
            props.get("node.name").unwrap().to_string(),
            props.get("node.nick").unwrap_or("unknown").to_string(),
            node_proxy,
        );
        let mut data = self.data.borrow_mut();
        data.set_def_sink_id_maybe(id, &sink.name);
        data.sinks.insert(id, sink);
    }

    fn set_meta(&self, registry: Arc<Registry>, meta_obj: &GlobalObject<ForeignDict>) {
        let metadata: Metadata = registry.bind(meta_obj).unwrap();
        let data_weak = Rc::downgrade(&self.data);
        let obj_listener = Box::new(
            metadata
                .add_listener_local()
                .property(move |_subject, key, _type, value| {
                    if let Some(k) = key {
                        if k == "default.audio.sink" {
                            if let Some(v) = value {
                                if let Some(data) = data_weak.upgrade() {
                                    data.borrow_mut().set_default_sink(v.to_string());
                                }
                            }
                        }
                    }
                    0
                })
                .register(),
        );
        self.data.borrow_mut().meta = Some(Proxy::<Metadata>::new(metadata).listener(obj_listener));
    }
}

pub struct PipewireDevice {
    cache: Arc<Mutex<PipewireCache>>,
}

impl PipewireDevice {
    pub fn new() -> Option<PipewireDevice> {
        pw::init();
        let cache = Arc::new(Mutex::new(PipewireCache::new()));
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

    fn listen(&mut self, block_index: usize) {
        let cache2 = self.cache.clone();
        thread::spawn(move || loop {
            let cache_clone = cache2.clone();
            let monitor = Monitor::new(block_index, cache_clone);
            let main_loop = pw::MainLoop::new().expect("cannot create loop");

            let context = pw::Context::new(&main_loop).unwrap();
            let main_loop_weak = main_loop.downgrade();
            let core = context.connect(None).unwrap();
            let _core_listener = core
                .add_listener_local()
                .error(move |_, _, _, _| {
                    if let Some(main_loop) = main_loop_weak.upgrade() {
                        main_loop.quit();
                    }
                })
                .register();

            let registry = Arc::new(core.get_registry().unwrap());
            let registry_weak = Arc::downgrade(&registry);

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
                                monitor.add_sink(registry, obj);
                            }
                            ObjectType::Metadata => {
                                if !check_prop(&obj.props, "metadata.name", "default") {
                                    return;
                                }
                                monitor.set_meta(registry, obj);
                            }
                            _ => {}
                        };
                    }
                })
                .register();
            main_loop.run();
        });
    }

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
