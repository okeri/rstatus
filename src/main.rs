/*
  status bar for tiling wms like i3, sway, etc...
  Copyright (C) 2019 Oleg Keri

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
mod base;

#[macro_use]
mod block;

#[macro_use]
mod blocks;
mod alsa_dev;
mod block_builder;
mod utility;

all_blocks! {mod_blocks}

use std::sync::{Arc, Mutex, Once};

type BlocksWrapper = Arc<Mutex<blocks::BlocksCollection>>;

fn blocks() -> BlocksWrapper {
    static mut SINGLETON: *const BlocksWrapper = 0 as *const BlocksWrapper;
    static ONCE: Once = Once::new();

    unsafe {
        ONCE.call_once(|| {
            let singleton = Arc::new(Mutex::new(init_blocks()));
            SINGLETON = std::mem::transmute(Box::new(singleton));
        });
        (*SINGLETON).clone()
    }
}

fn update(sig: i32) {
    let mut refresh = false;
    let blocks = blocks();
    for block in blocks.lock().unwrap().iter_mut() {
        if sig == block.signal() as i32 + utility::SIGRTMIN {
            block.update();
            refresh = true;
        }
    }
    if refresh {
        display_all();
    }
}

fn display_all() {
    print!("[");
    let mut first = true;
    let blocks = blocks();
    let mut prev_bg: Option<u32> = None;
    for block in blocks.lock().unwrap().iter() {
        if !first {
            print!(",");
        } else {
            first = false
        }
        block.render(prev_bg);
        prev_bg = block.bgcolor();
    }
    println!("],");
}

fn init_blocks() -> blocks::BlocksCollection {
    let home = std::env::var_os("HOME").expect("error getting home var");
    let cfg_path = std::path::Path::new(&home)
        .join(".config")
        .join("rstatus")
        .join("config.yaml");

    blocks::init_from_config(&cfg_path)
}

fn main() {
    let blocks = blocks();
    let mut gcd: u32 = 0;
    for block in blocks.lock().unwrap().iter() {
        gcd = block.interval();
        if gcd != 0 {
            break;
        }
    }

    if gcd != 0 {
        for block in blocks.lock().unwrap().iter_mut() {
            block.update();
            let interval = block.interval();
            if interval != 0 {
                gcd = utility::gcd(gcd, interval);
            }
            let signal = block.signal();
            if signal != 0 {
                utility::signal(utility::SIGRTMIN + signal as i32, update);
            }
        }

        println!("{{\"version\": 1, \"click_events\": false}}\n[");
        let mut count = 1u64;
        loop {
            std::thread::sleep(std::time::Duration::from_secs(gcd as u64));
            for block in blocks.lock().unwrap().iter_mut() {
                let interval = block.interval();
                if interval != 0 && (block.retry(gcd) || count % interval as u64 == 0) {
                    block.update();
                }
            }
            display_all();
            count += gcd as u64
        }
    }
}
