use super::base::{Base, Value};
use super::block;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Block {
    #[serde(flatten)]
    base: Base,
    path: String,
}

impl block::Block for Block {
    impl_Block!();
    fn update(&mut self) {
        use std::ffi::CString;
        use std::mem;
        unsafe {
            let path = CString::new(self.path.clone()).unwrap();
            let mut usage: libc::statvfs = mem::zeroed();
            self.base.value = if libc::statvfs(path.as_ptr(), &mut usage) == -1 {
                Value::Invalid
            } else {
                let used = (100 * (usage.f_blocks - usage.f_bfree) / usage.f_blocks) as u32;
                Value::new(used)
            };
        }
    }
}
