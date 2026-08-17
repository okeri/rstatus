use super::blocks::update_by_index;

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    thread,
    time::Instant,
};

use zbus::{
    blocking::{
        fdo::{DBusProxy, PropertiesProxy},
        Connection, MessageIterator,
    },
    message::Type as MessageType,
    names::{BusName, InterfaceName},
    zvariant::{OwnedValue, Value},
    MatchRule,
};

const RECONNECT_DELAY_SECS: u64 = 1;
const BUS_NAMESPACE: &str = "org.mpris.MediaPlayer2";
const BUS_PREFIX: &str = "org.mpris.MediaPlayer2.";
const OBJECT_PATH: &str = "/org/mpris/MediaPlayer2";
const PLAYER_INTERFACE: &str = "org.mpris.MediaPlayer2.Player";
const PROPERTIES_INTERFACE: &str = "org.freedesktop.DBus.Properties";
const DBUS_INTERFACE: &str = "org.freedesktop.DBus";

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum PlaybackStatus {
    Stopped = 0,
    Paused = 1,
    Playing = 2,
}

impl PlaybackStatus {
    fn parse(value: &str) -> Self {
        match value {
            "Playing" => PlaybackStatus::Playing,
            "Paused" => PlaybackStatus::Paused,
            _ => PlaybackStatus::Stopped,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            PlaybackStatus::Playing => "playing",
            PlaybackStatus::Paused => "paused",
            PlaybackStatus::Stopped => "stopped",
        }
    }
}

#[derive(Clone, PartialEq)]
pub struct Track {
    pub status: PlaybackStatus,
    pub artist: String,
    pub title: String,
    pub album: String,
    pub player: String,
    pub position: Option<i64>,
    pub length: Option<i64>,
    pub rate: f64,
}

impl Track {
    fn known(&self) -> bool {
        !self.artist.is_empty() || !self.title.is_empty() || !self.album.is_empty()
    }
}

pub struct Snapshot {
    pub track: Track,
    pub sampled: Instant,
}

struct MprisCache {
    track: Option<Track>,
    sampled: Instant,
}

impl MprisCache {
    fn new() -> Self {
        MprisCache {
            track: None,
            sampled: Instant::now(),
        }
    }
}

struct PlayerState {
    bus_name: String,
    track: Track,
    sampled: Instant,
    play_seq: u64,
}

impl PlayerState {
    fn new(bus_name: String) -> Self {
        let player = bus_name
            .strip_prefix(BUS_PREFIX)
            .unwrap_or(&bus_name)
            .split('.')
            .next()
            .unwrap_or_default()
            .to_owned();
        PlayerState {
            bus_name,
            track: Track {
                status: PlaybackStatus::Stopped,
                artist: String::new(),
                title: String::new(),
                album: String::new(),
                player,
                position: None,
                length: None,
                rate: 1.,
            },
            sampled: Instant::now(),
            play_seq: 0,
        }
    }
}

fn as_text(value: &Value<'_>) -> Option<String> {
    match value {
        Value::Str(text) => {
            let text = text.as_str();
            (!text.is_empty()).then(|| text.to_owned())
        }
        Value::Array(items) => {
            let joined = items
                .iter()
                .filter_map(as_text)
                .collect::<Vec<String>>()
                .join(", ");
            (!joined.is_empty()).then_some(joined)
        }
        Value::Value(inner) => as_text(inner),
        _ => None,
    }
}

fn as_i64(value: &Value<'_>) -> Option<i64> {
    match value {
        Value::I64(number) => Some(*number),
        Value::I32(number) => Some(i64::from(*number)),
        Value::U64(number) => i64::try_from(*number).ok(),
        Value::U32(number) => Some(i64::from(*number)),
        Value::Value(inner) => as_i64(inner),
        _ => None,
    }
}

fn as_f64(value: &Value<'_>) -> Option<f64> {
    match value {
        Value::F64(number) => Some(*number),
        Value::Value(inner) => as_f64(inner),
        _ => as_i64(value).map(|number| number as f64),
    }
}

fn suffix_matches(bus_name: &str, pattern: &str) -> bool {
    match bus_name.strip_prefix(BUS_PREFIX) {
        Some(suffix) => {
            suffix == pattern
                || suffix
                    .strip_prefix(pattern)
                    .is_some_and(|rest| rest.starts_with('.'))
        }
        None => false,
    }
}

fn fetch_props(conn: &Connection, bus_name: &str) -> Option<HashMap<String, OwnedValue>> {
    let proxy = PropertiesProxy::builder(conn)
        .destination(bus_name.to_owned())
        .ok()?
        .path(OBJECT_PATH)
        .ok()?
        .build()
        .ok()?;
    proxy
        .get_all(InterfaceName::try_from(PLAYER_INTERFACE).ok()?)
        .ok()
}

fn fetch_position(conn: &Connection, bus_name: &str) -> Option<i64> {
    let proxy = PropertiesProxy::builder(conn)
        .destination(bus_name.to_owned())
        .ok()?
        .path(OBJECT_PATH)
        .ok()?
        .build()
        .ok()?;
    let value = proxy
        .get(InterfaceName::try_from(PLAYER_INTERFACE).ok()?, "Position")
        .ok()?;
    as_i64(&value)
}

struct Monitor {
    block_index: usize,
    cache: Arc<Mutex<MprisCache>>,
    priority: Vec<String>,
    players: HashMap<String, PlayerState>,
    seq: u64,
}

impl Monitor {
    fn new(block_index: usize, cache: Arc<Mutex<MprisCache>>, priority: Vec<String>) -> Self {
        Monitor {
            block_index,
            cache,
            priority,
            players: HashMap::new(),
            seq: 0,
        }
    }

    fn rank(&self, bus_name: &str) -> usize {
        self.priority
            .iter()
            .position(|pattern| suffix_matches(bus_name, pattern))
            .unwrap_or(usize::MAX)
    }

    fn accepts(&self, bus_name: &str) -> bool {
        self.priority.is_empty() || self.rank(bus_name) != usize::MAX
    }

    fn apply_props(&mut self, owner: &str, props: &HashMap<String, OwnedValue>) -> bool {
        let seq = self.seq;
        let Some(player) = self.players.get_mut(owner) else {
            return false;
        };
        let mut resync = false;

        if let Some(status) = props.get("PlaybackStatus").and_then(|v| as_text(v)) {
            let status = PlaybackStatus::parse(&status);
            if status == PlaybackStatus::Playing && player.track.status != PlaybackStatus::Playing {
                player.play_seq = seq + 1;
                self.seq = seq + 1;
            }
            player.track.status = status;
            resync = true;
        }

        if let Some(rate) = props.get("Rate").and_then(|v| as_f64(v)) {
            if rate > 0. {
                player.track.rate = rate;
            }
        }

        if let Some(position) = props.get("Position").and_then(|v| as_i64(v)) {
            player.track.position = Some(position.max(0));
            player.sampled = Instant::now();
            resync = false;
        }

        if let Some(Value::Dict(metadata)) = props.get("Metadata").map(|v| &**v) {
            let (mut title, mut artist, mut album_artist, mut album, mut length) =
                (None, None, None, None, None);
            for (key, value) in metadata.iter() {
                let Value::Str(key) = key else {
                    continue;
                };
                match key.as_str() {
                    "xesam:title" => title = as_text(value),
                    "xesam:artist" => artist = as_text(value),
                    "xesam:albumArtist" => album_artist = as_text(value),
                    "xesam:album" => album = as_text(value),
                    "mpris:length" => length = as_i64(value),
                    _ => {}
                }
            }
            player.track.title = title.unwrap_or_default();
            player.track.artist = artist.or(album_artist).unwrap_or_default();
            player.track.album = album.unwrap_or_default();
            player.track.length = length.filter(|value| *value > 0);
            resync = true;
        }
        resync
    }

    fn seek(&mut self, owner: &str, position: i64) {
        if let Some(player) = self.players.get_mut(owner) {
            player.track.position = Some(position.max(0));
            player.sampled = Instant::now();
        }
    }

    fn resync(&mut self, conn: &Connection, owner: &str) {
        let Some(bus_name) = self.players.get(owner).map(|p| p.bus_name.clone()) else {
            return;
        };
        if let Some(position) = fetch_position(conn, &bus_name) {
            self.seek(owner, position);
        }
    }

    fn add(&mut self, conn: &Connection, bus_name: &str, owner: String) {
        if !self.accepts(bus_name) {
            return;
        }
        self.players
            .insert(owner.clone(), PlayerState::new(bus_name.to_owned()));
        if let Some(props) = fetch_props(conn, bus_name) {
            self.apply_props(&owner, &props);
        }
    }

    fn select(&self) -> Option<&PlayerState> {
        self.players.values().max_by_key(|player| {
            (
                player.track.status as u8,
                std::cmp::Reverse(self.rank(&player.bus_name)),
                player.play_seq,
            )
        })
    }

    fn refresh(&self) {
        let winner = self.select().filter(|player| player.track.known());
        let track = winner.map(|player| player.track.clone());
        let sampled = winner.map_or_else(Instant::now, |player| player.sampled);

        {
            let mut cache = self.cache.lock().unwrap();
            if cache.track == track {
                cache.sampled = sampled;
                return;
            }
            cache.track = track;
            cache.sampled = sampled;
        }
        update_by_index(self.block_index);
    }

    fn run(&mut self) -> zbus::Result<()> {
        let conn = Connection::session()?;
        let dbus = DBusProxy::new(&conn)?;

        dbus.add_match_rule(
            MatchRule::builder()
                .msg_type(MessageType::Signal)
                .interface(PROPERTIES_INTERFACE)?
                .member("PropertiesChanged")?
                .path(OBJECT_PATH)?
                .build(),
        )?;

        dbus.add_match_rule(
            MatchRule::builder()
                .msg_type(MessageType::Signal)
                .sender(DBUS_INTERFACE)?
                .interface(DBUS_INTERFACE)?
                .member("NameOwnerChanged")?
                .arg0ns(BUS_NAMESPACE)?
                .build(),
        )?;

        dbus.add_match_rule(
            MatchRule::builder()
                .msg_type(MessageType::Signal)
                .interface(PLAYER_INTERFACE)?
                .member("Seeked")?
                .path(OBJECT_PATH)?
                .build(),
        )?;

        for name in dbus.list_names()?.iter() {
            let name = name.as_str();
            if !name.starts_with(BUS_PREFIX) {
                continue;
            }
            let Ok(bus_name) = BusName::try_from(name) else {
                continue;
            };
            if let Ok(owner) = dbus.get_name_owner(bus_name) {
                self.add(&conn, name, owner.to_string());
            }
        }
        self.refresh();

        for message in MessageIterator::from(&conn) {
            let Ok(message) = message else {
                continue;
            };
            let header = message.header();
            let (Some(interface), Some(member)) = (header.interface(), header.member()) else {
                continue;
            };
            let body = message.body();

            if interface == PROPERTIES_INTERFACE && member == "PropertiesChanged" {
                let Some(owner) = header.sender().map(|sender| sender.to_string()) else {
                    continue;
                };
                if !self.players.contains_key(&owner) {
                    continue;
                }
                let Ok((iface, changed, _invalidated)) =
                    body.deserialize::<(String, HashMap<String, OwnedValue>, Vec<String>)>()
                else {
                    continue;
                };
                if iface != PLAYER_INTERFACE {
                    continue;
                }
                if self.apply_props(&owner, &changed) {
                    self.resync(&conn, &owner);
                }
                self.refresh();
            } else if interface == PLAYER_INTERFACE && member == "Seeked" {
                let Some(owner) = header.sender().map(|sender| sender.to_string()) else {
                    continue;
                };
                let Ok((position,)) = body.deserialize::<(i64,)>() else {
                    continue;
                };
                self.seek(&owner, position);
                self.refresh();
            } else if interface == DBUS_INTERFACE && member == "NameOwnerChanged" {
                let Ok((name, _old, new)) = body.deserialize::<(String, String, String)>() else {
                    continue;
                };
                if !name.starts_with(BUS_PREFIX) {
                    continue;
                }
                if new.is_empty() {
                    self.players.retain(|_, player| player.bus_name != name);
                } else {
                    self.add(&conn, &name, new);
                }
                self.refresh();
            }
        }
        Ok(())
    }
}

pub struct MprisService {
    cache: Arc<Mutex<MprisCache>>,
}

impl MprisService {
    pub fn new(block_index: usize, priority: Vec<String>) -> Self {
        let cache = Arc::new(Mutex::new(MprisCache::new()));
        let thread_cache = cache.clone();
        thread::spawn(move || {
            let mut first = true;
            loop {
                if !first {
                    thread::sleep(std::time::Duration::from_secs(RECONNECT_DELAY_SECS));
                }
                first = false;
                let mut monitor = Monitor::new(block_index, thread_cache.clone(), priority.clone());
                let _ = monitor.run();
                monitor.players.clear();
                monitor.refresh();
            }
        });
        MprisService { cache }
    }

    pub fn snapshot(&self) -> Option<Snapshot> {
        let cache = self.cache.lock().unwrap();
        cache.track.clone().map(|track| Snapshot {
            track,
            sampled: cache.sampled,
        })
    }
}
