/*
  status bar for i3like wms like i3, sway, etc...
  Copyright (C) 2017 Oleg Keri

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
extern crate libc;

#[macro_use]
mod block;

mod utility;
mod config;
mod undefined;
mod cpuload;
mod memory;
mod temperature;
mod battery;
mod filesystem;
mod wifi;
mod time;
mod volume;
mod custom;

use std::sync::{Arc, Mutex, Once, ONCE_INIT};

type BlocksWrapper = Arc<Mutex<Vec<Box<block::Block>>>>;

fn blocks() -> BlocksWrapper {
    static mut SINGLETON: *const BlocksWrapper = 0 as *const BlocksWrapper;
    static ONCE: Once = ONCE_INIT;

    unsafe {
        ONCE.call_once(|| {
            let singleton = Arc::new(Mutex::new(init_blocks()));
            SINGLETON = std::mem::transmute(Box::new(singleton));
        });
        (*SINGLETON).clone()
    }
}

fn update(sig: i32) {
    let blocks = blocks();
    let mut refresh = false;
    for block in blocks.lock().unwrap().iter_mut() {
        if sig == block.info().signal as i32 + utility::SIGRTMIN {
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
    for block in blocks.lock().unwrap().iter_mut() {
        if !first { print!(",") } else { first = false }
        block.serialize();
    }
    println!("],");
}

fn init_blocks() -> Vec<Box<block::Block>> {
    let home = std::env::var_os("HOME").expect("error getting home var");
    let cfg_path = std::path::Path::new(&home)
        .join(".config")
        .join("rstatus")
        .join("config.yaml");

    config::init_from(&cfg_path)
}

fn main() {
    let blocks = blocks();
    let mut gcd: u32 = 0;
    for block in blocks.lock().unwrap().iter() {
        gcd = block.info().interval;
        if gcd != 0 {
            break;
        }
    }

    if gcd != 0 {
        for block in blocks.lock().unwrap().iter_mut() {
            block.update();
            let info = block.info();
            if info.interval != 0 {
                gcd = utility::gcd(gcd, info.interval);
            }
            if info.signal != 0 {
                utility::signal(utility::SIGRTMIN + info.signal as i32, update);
            }
        }

        println!("{{\"version\": 1, \"click_events\": true}}\n[");
        let mut count = 1u64;
        loop {
            std::thread::sleep(std::time::Duration::from_secs(gcd as u64));
            for block in blocks.lock().unwrap().iter_mut() {
                if block.info().interval != 0 &&
                    (block.retry(gcd) || count % block.info().interval as u64 == 0)
                {
                    block.update();
                }

            }
            display_all();
            count += gcd as u64
        }
    }
}
