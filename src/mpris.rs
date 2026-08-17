use super::base::{default_str_none, default_zero, Base, Status, Value};
use super::block;
use super::mpris_dev::{MprisService, PlaybackStatus, Snapshot, Track};
use serde::Deserialize;
use std::collections::BTreeMap;

#[derive(Deserialize)]
pub struct Block {
    #[serde(flatten)]
    base: Base,
    #[serde(default = "default_statuses")]
    statuses: BTreeMap<String, Status>,
    #[serde(default = "default_players")]
    players: Vec<String>,
    #[serde(default = "default_str_none")]
    format: Option<String>,
    #[serde(default = "default_zero")]
    max_length: u32,
    #[serde(skip, default = "default_service")]
    service: Option<MprisService>,
}

fn default_statuses() -> BTreeMap<String, Status> {
    BTreeMap::new()
}

fn default_players() -> Vec<String> {
    Vec::new()
}

fn default_service() -> Option<MprisService> {
    None
}

enum Resolved {
    Value(String),
    Unresolved,
    Unknown,
}

fn truncate(text: String, budget: usize) -> String {
    if text.chars().count() <= budget {
        return text;
    }
    if budget == 0 {
        return String::new();
    }
    let mut result: String = text.chars().take(budget).collect();
    result.push('…');
    result
}

fn take(text: &str, budget: &mut usize) -> String {
    if *budget == usize::MAX {
        return text.to_owned();
    }
    let count = text.chars().count();
    if count <= *budget {
        *budget -= count;
        return text.to_owned();
    }
    let result = truncate(text.to_owned(), *budget);
    *budget = 0;
    result
}

fn duration(micros: i64) -> String {
    let total = (micros / 1_000_000).max(0);
    let (hours, minutes, seconds) = (total / 3600, (total / 60) % 60, total % 60);
    if hours > 0 {
        format!("{}:{:02}:{:02}", hours, minutes, seconds)
    } else {
        format!("{:02}:{:02}", minutes, seconds)
    }
}

fn position(snapshot: &Snapshot) -> Option<i64> {
    let base = snapshot.track.position?;
    let elapsed = if snapshot.track.status == PlaybackStatus::Playing {
        (snapshot.sampled.elapsed().as_micros() as f64 * snapshot.track.rate) as i64
    } else {
        0
    };
    let value = base.saturating_add(elapsed).max(0);
    Some(match snapshot.track.length {
        Some(length) => value.min(length),
        None => value,
    })
}

fn resolve(name: &str, track: &Track, position: Option<i64>, budget: &mut usize) -> Resolved {
    let text = match name {
        "artist" => &track.artist,
        "title" => &track.title,
        "album" => &track.album,
        "player" => &track.player,
        "position" => {
            return position.map_or(Resolved::Unresolved, |value| {
                Resolved::Value(duration(value))
            })
        }
        "length" => {
            return track.length.map_or(Resolved::Unresolved, |value| {
                Resolved::Value(duration(value))
            })
        }
        _ => return Resolved::Unknown,
    };
    if text.is_empty() {
        return Resolved::Unresolved;
    }
    Resolved::Value(take(text, budget))
}

fn render(
    format: &str,
    track: &Track,
    position: Option<i64>,
    budget: &mut usize,
    grouped: bool,
) -> Option<String> {
    let mut out = String::new();
    let mut rest = format;
    while !rest.is_empty() {
        let Some(at) = rest.find(['{', '[']) else {
            out.push_str(rest);
            break;
        };
        out.push_str(&rest[..at]);
        rest = &rest[at..];

        if let Some(tail) = rest.strip_prefix('{') {
            let Some(end) = tail.find('}') else {
                out.push_str(rest);
                break;
            };
            match resolve(&tail[..end], track, position, budget) {
                Resolved::Value(value) => out.push_str(&value),
                Resolved::Unresolved if grouped => return None,
                Resolved::Unresolved => {}
                Resolved::Unknown => out.push_str(&rest[..end + 2]),
            }
            rest = &tail[end + 1..];
        } else if grouped {
            out.push('[');
            rest = &rest[1..];
        } else {
            let Some(end) = rest.find(']') else {
                out.push('[');
                rest = &rest[1..];
                continue;
            };
            let mut local = *budget;
            if let Some(group) = render(&rest[1..end], track, position, &mut local, true) {
                out.push('[');
                out.push_str(&group);
                out.push(']');
                *budget = local;
            }
            rest = &rest[end + 1..];
        }
    }
    Some(out)
}

impl Block {
    fn text(&self, snapshot: &Snapshot) -> String {
        let budget = if self.max_length == 0 {
            usize::MAX
        } else {
            self.max_length as usize
        };
        let track = &snapshot.track;

        let Some(format) = self.format.as_ref() else {
            let plain = match (track.artist.is_empty(), track.title.is_empty()) {
                (false, false) => format!("{} - {}", track.artist, track.title),
                (true, false) => track.title.clone(),
                (false, true) => track.artist.clone(),
                (true, true) => String::new(),
            };
            return truncate(plain, budget);
        };

        let mut budget = budget;
        render(format, track, position(snapshot), &mut budget, false)
            .map(|text| text.trim().to_owned())
            .unwrap_or_default()
    }
}

impl block::Block for Block {
    impl_Block!();

    fn update(&mut self) {
        if self.service.is_none() {
            self.service = Some(MprisService::new(
                self.base.index(),
                std::mem::take(&mut self.players),
            ));
        }

        let snapshot = self.service.as_ref().and_then(|service| service.snapshot());
        self.base.value = match snapshot {
            Some(snapshot) => {
                if let Some(decoration) = self.statuses.get(snapshot.track.status.name()) {
                    self.base.set_prefix(&decoration.prefix);
                    self.base.set_suffix(&decoration.suffix);
                }
                let text = self.text(&snapshot);
                if text.is_empty() {
                    Value::Invalid
                } else {
                    Value::new(text)
                }
            }
            None => Value::Invalid,
        }
    }
}
