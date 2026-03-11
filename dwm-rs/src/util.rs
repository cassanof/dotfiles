use libc::c_char;
use std::ffi::CStr;
use std::io;
use std::process;

pub fn die(message: impl AsRef<str>) -> ! {
    eprintln!("{}", message.as_ref());
    process::exit(1);
}

pub fn die_perror(prefix: impl AsRef<str>) -> ! {
    eprintln!("{} {}", prefix.as_ref(), io::Error::last_os_error());
    process::exit(1);
}

pub fn max_i32(a: i32, b: i32) -> i32 {
    a.max(b)
}

pub fn min_i32(a: i32, b: i32) -> i32 {
    a.min(b)
}

pub fn between(value: i64, low: i64, high: i64) -> bool {
    low <= value && value <= high
}

pub unsafe fn copy_cstr(dst: &mut [c_char], src: *const c_char) {
    if dst.is_empty() {
        return;
    }

    dst.fill(0);
    if src.is_null() {
        return;
    }

    let bytes = CStr::from_ptr(src).to_bytes();
    let len = bytes.len().min(dst.len() - 1);
    for (index, byte) in bytes.iter().copied().take(len).enumerate() {
        dst[index] = byte as c_char;
    }
}

pub fn copy_bytes_to_cstr(dst: &mut [c_char], src: &[u8]) {
    if dst.is_empty() {
        return;
    }

    dst.fill(0);
    let len = src.len().min(dst.len() - 1);
    for (index, byte) in src.iter().copied().take(len).enumerate() {
        dst[index] = byte as c_char;
    }
}
