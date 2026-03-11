mod config;
mod drw;
mod ffi;
mod types;
mod util;
mod wm;

fn main() {
    unsafe {
        wm::main_entry();
    }
}
