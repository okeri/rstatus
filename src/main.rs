mod base;

#[macro_use]
mod block;

#[macro_use]
mod blocks;

#[cfg(feature = "pulse")]
mod pulse_dev;

#[cfg(feature = "alsa")]
mod alsa_dev;

#[cfg(feature = "pipewire")]
mod pipewire_dev;

mod block_builder;
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
        let mut signals = vec![];
        for block in blocks.lock().unwrap().iter_mut() {
            block.update();
            let interval = block.interval();
            if interval != 0 {
                gcd = utility::gcd(gcd, interval);
            }
            let signal = block.signal();
            if signal != 0 {
                utility::signal(signal as i32, update_by_signal);
                signals.push(signal as i32);
            }
        }

        println!("{{\"version\": 1, \"click_events\": false}}\n[");
        if !signals.is_empty() {
            std::thread::spawn(|| {
                let day = std::time::Duration::from_secs(86400);
                loop {
                    std::thread::sleep(day);
                }
            });
        }
        utility::mask(signals);
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
