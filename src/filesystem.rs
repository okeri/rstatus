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
            let path = match CString::new(self.path.clone()) {
                Ok(p) => p,
                Err(_) => {
                    self.base.value = Value::Invalid;
                    return;
                }
            };
            let mut usage: libc::statvfs = mem::zeroed();
            if libc::statvfs(path.as_ptr(), &mut usage) == -1 {
                self.base.value = Value::Invalid;
                return;
            }
            let used = usage.f_blocks - usage.f_bfree;
            let total = used + usage.f_bavail;
            self.base.value = if total == 0 {
                Value::Invalid
            } else {
                Value::new(((used * 100 + total - 1) / total) as u32)
            };
        }
    }
}
