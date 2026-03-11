use libc::{c_char, c_float, c_int, c_uint, c_void, pid_t};
use x11::xlib::Window;

pub const CUR_NORMAL: usize = 0;
pub const CUR_RESIZE: usize = 1;
pub const CUR_MOVE: usize = 2;
pub const CUR_LAST: usize = 3;

pub const SCHEME_NORM: usize = 0;
pub const SCHEME_SEL: usize = 1;
pub const NET_SUPPORTED: usize = 0;
pub const NET_WM_NAME: usize = 1;
pub const NET_WM_STATE: usize = 2;
pub const NET_WM_CHECK: usize = 3;
pub const NET_WM_FULLSCREEN: usize = 4;
pub const NET_ACTIVE_WINDOW: usize = 5;
pub const NET_WM_WINDOW_TYPE: usize = 6;
pub const NET_WM_WINDOW_TYPE_DIALOG: usize = 7;
pub const NET_CLIENT_LIST: usize = 8;
pub const NET_LAST: usize = 9;

pub const WM_PROTOCOLS: usize = 0;
pub const WM_DELETE: usize = 1;
pub const WM_STATE: usize = 2;
pub const WM_TAKE_FOCUS: usize = 3;
pub const WM_LAST: usize = 4;

pub const CLK_TAG_BAR: c_uint = 0;
pub const CLK_LT_SYMBOL: c_uint = 1;
pub const CLK_STATUS_TEXT: c_uint = 2;
pub const CLK_WIN_TITLE: c_uint = 3;
pub const CLK_CLIENT_WIN: c_uint = 4;
pub const CLK_ROOT_WIN: c_uint = 5;

pub type ActionFn = unsafe fn(*const Arg);
pub type ArrangeFn = unsafe fn(*mut Monitor);

#[derive(Clone, Copy)]
#[repr(C)]
pub union Arg {
    pub i: c_int,
    pub ui: c_uint,
    pub f: c_float,
    pub v: *const c_void,
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct Button {
    pub click: c_uint,
    pub mask: c_uint,
    pub button: c_uint,
    pub func: Option<ActionFn>,
    pub arg: Arg,
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct Key {
    pub mod_: c_uint,
    pub keysym: c_uint,
    pub func: Option<ActionFn>,
    pub arg: Arg,
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct Layout {
    pub symbol: *const c_char,
    pub arrange: Option<ArrangeFn>,
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct Rule {
    pub class: *const c_char,
    pub instance: *const c_char,
    pub title: *const c_char,
    pub tags: c_uint,
    pub isfloating: c_int,
    pub isterminal: c_int,
    pub noswallow: c_int,
    pub monitor: c_int,
}

#[repr(C)]
pub struct Client {
    pub name: [c_char; 256],
    pub mina: c_float,
    pub maxa: c_float,
    pub x: c_int,
    pub y: c_int,
    pub w: c_int,
    pub h: c_int,
    pub oldx: c_int,
    pub oldy: c_int,
    pub oldw: c_int,
    pub oldh: c_int,
    pub basew: c_int,
    pub baseh: c_int,
    pub incw: c_int,
    pub inch: c_int,
    pub maxw: c_int,
    pub maxh: c_int,
    pub minw: c_int,
    pub minh: c_int,
    pub bw: c_int,
    pub oldbw: c_int,
    pub tags: c_uint,
    pub isfixed: c_int,
    pub isfloating: c_int,
    pub isurgent: c_int,
    pub neverfocus: c_int,
    pub oldstate: c_int,
    pub isfullscreen: c_int,
    pub isterminal: c_int,
    pub noswallow: c_int,
    pub pid: pid_t,
    pub next: *mut Client,
    pub snext: *mut Client,
    pub swallowing: *mut Client,
    pub mon: *mut Monitor,
    pub win: Window,
}

#[repr(C)]
pub struct Monitor {
    pub ltsymbol: [c_char; 16],
    pub mfact: c_float,
    pub nmaster: c_int,
    pub num: c_int,
    pub by: c_int,
    pub mx: c_int,
    pub my: c_int,
    pub mw: c_int,
    pub mh: c_int,
    pub wx: c_int,
    pub wy: c_int,
    pub ww: c_int,
    pub wh: c_int,
    pub seltags: c_uint,
    pub sellt: c_uint,
    pub tagset: [c_uint; 2],
    pub showbar: c_int,
    pub topbar: c_int,
    pub clients: *mut Client,
    pub sel: *mut Client,
    pub stack: *mut Client,
    pub next: *mut Monitor,
    pub barwin: Window,
    pub lt: [*const Layout; 2],
}
