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
mod pulse_dev;
mod sound_service;
mod utility;

all_blocks! {mod_blocks}

fn update_by_signal(sig: i32) {
    let mut refresh = false;
    for block in blocks::blocks().lock().unwrap().iter_mut() {
        if sig == block.signal() as i32 + utility::SIGRTMIN {
            block.update();
            refresh = true;
        }
    }
    if refresh {
        blocks::display_all();
    }
}

fn main() {
    let blocks = blocks::blocks();
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
                utility::signal(signal as i32, update_by_signal);
            }
        }

        println!("{{\"version\": 1, \"click_events\": false}}\n[");
        let mut count = 1u64;
        loop {
            std::thread::sleep(std::time::Duration::from_secs(gcd as u64));
            for block in blocks.lock().unwrap().iter_mut() {
                let interval = block.interval();
                if interval != 0 && count % interval as u64 == 0 {
                    block.update();
                }
            }
            blocks::display_all();
            count += gcd as u64
        }
    }
}
