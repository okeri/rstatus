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

use std::{fs, io, mem, ptr, str};

pub const SIGRTMIN: i32 = 34;

pub fn read_filen(filename: &str, max: usize) -> Result<String, io::Error> {
    use std::io::Read;
    let mut file = fs::File::open(filename)?;
    let mut buf = vec![0u8; max];
    file.read_exact(&mut buf)
        .map(|_| String::from(str::from_utf8(&buf).unwrap()))
}

pub fn gcd(i1: u32, i2: u32) -> u32 {
    let mut x = i1;
    let mut y = i2;
    let mut r = x % y;
    while r != 0 {
        x = y;
        y = r;
        r = x % y;
    }
    y
}

pub fn mask(signals: Vec<i32>) {
    unsafe {
        let mut sigset = mem::MaybeUninit::uninit().assume_init();
        if libc::sigemptyset(&mut sigset) != -1 {
            for signal in signals.iter() {
                libc::sigaddset(&mut sigset, SIGRTMIN + signal);
            }
            libc::pthread_sigmask(libc::SIG_BLOCK, &mut sigset, ptr::null_mut());
        }
    }
}

pub fn signal(signal: i32, action: fn(i32)) {
    unsafe {
        let mut sigset = mem::MaybeUninit::uninit().assume_init();
        if libc::sigfillset(&mut sigset) != -1 {
            let mut sigaction: libc::sigaction = mem::zeroed();
            sigaction.sa_mask = sigset;
            sigaction.sa_sigaction = action as usize;
            libc::sigaction(signal + SIGRTMIN, &sigaction, ptr::null_mut());
        }
    }
}

pub fn read_color(input: &str, default: u32) -> u32 {
    if let Some(first) = input.chars().next() {
        if first == '#' {
            i64::from_str_radix(&input[1..], 16).unwrap_or(default as i64) as u32
        } else {
            i64::from_str_radix(input, 16).unwrap_or(default as i64) as u32
        }
    } else {
        default
    }
}
