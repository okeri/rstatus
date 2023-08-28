use std::{
    fs, io,
    mem::{zeroed, MaybeUninit},
    ptr, str,
};
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
        let mut sigset = MaybeUninit::uninit();
        if libc::sigemptyset(sigset.as_mut_ptr()) != -1 {
            let mut sigset = sigset.assume_init();
            for signal in signals.iter() {
                libc::sigaddset(&mut sigset, SIGRTMIN + signal);
            }
            libc::pthread_sigmask(libc::SIG_BLOCK, &sigset, ptr::null_mut());
        }
    }
}

pub fn signal(signal: i32, action: fn(i32)) {
    unsafe {
        let mut sigset = MaybeUninit::uninit();
        if libc::sigfillset(sigset.as_mut_ptr()) != -1 {
            let mut sigaction: libc::sigaction = zeroed();
            sigaction.sa_mask = sigset.assume_init();
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
