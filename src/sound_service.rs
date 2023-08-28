pub trait SoundService {
    fn id(&self) -> String;
    fn sink_name(&self) -> String;
    fn exists(&self, mixer: &str) -> bool;
    fn jack_plugged(&self) -> Option<bool>;
    fn update(&self);
    fn volume(&self, mixer: &str) -> Option<u32>;
    fn listen(&mut self, block_index: usize);
    fn set_mute(&self, mixer: &str, value: bool);
}
