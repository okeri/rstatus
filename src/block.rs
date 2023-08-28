pub trait Block {
    fn update(&mut self);
    fn set_name(&mut self, name: String);
    fn interval(&self) -> u32;
    fn signal(&self) -> u32;
    fn bgcolor(&self) -> Option<u32>;
    fn render(&self, prev_bg: Option<u32>);
    fn set_index(&mut self, index: usize);
}

macro_rules! impl_Block {
    () => {
        fn interval(&self) -> u32 {
            self.base.interval()
        }
        fn signal(&self) -> u32 {
            self.base.signal()
        }
        fn render(&self, prev_bg: Option<u32>) {
            self.base.render(prev_bg);
        }
        fn set_name(&mut self, name: String) {
            self.base.set_name(name);
        }
        fn bgcolor(&self) -> Option<u32> {
            self.base.bg()
        }
        fn set_index(&mut self, index: usize) {
            self.base.set_index(index);
        }
    };
}
