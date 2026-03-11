use crate::types::{Arg, Button, Key, Layout, Rule};
use libc::{c_char, c_float, c_int, c_uint};
use std::ptr;
use x11::keysym;
use x11::xlib;

macro_rules! cstr {
    ($value:literal) => {
        concat!($value, "\0").as_ptr().cast::<c_char>()
    };
}

pub const VERSION: &str = "6.2";
pub const PRINT_SCREEN_DWM: c_uint = 0x0000ff61;

pub const BORDERPX: c_uint = 2;
pub const GAPPX: c_uint = 8;
pub const SNAP: c_uint = 32;
pub const SWALLOWFLOATING: c_int = 0;
pub const SHOWBAR: c_int = 1;
pub const TOPBAR: c_int = 1;

pub const FONTS: [*const c_char; 1] = [cstr!(
    "Jetbrains Mono:pixelsize=18:antialias=true:autohint=true"
)];

pub const COLORS: [[*const c_char; 3]; 2] = [
    [cstr!("#EBDBB2"), cstr!("#282828"), cstr!("#282828")],
    [cstr!("#EBDBB2"), cstr!("#98971A"), cstr!("#FE8019")],
];

pub const TAGS: [*const c_char; 9] = [
    cstr!("1"),
    cstr!("2"),
    cstr!("3"),
    cstr!("4"),
    cstr!("5"),
    cstr!("6"),
    cstr!("7"),
    cstr!("8"),
    cstr!("9"),
];

pub const RULES: [Rule; 6] = [
    Rule {
        class: cstr!("Gimp"),
        instance: ptr::null(),
        title: ptr::null(),
        tags: 0,
        isfloating: 1,
        isterminal: 0,
        noswallow: 0,
        monitor: -1,
    },
    Rule {
        class: cstr!("firefox"),
        instance: ptr::null(),
        title: ptr::null(),
        tags: 0,
        isfloating: 0,
        isterminal: 0,
        noswallow: -1,
        monitor: -1,
    },
    Rule {
        class: cstr!("LibreWolf"),
        instance: ptr::null(),
        title: ptr::null(),
        tags: 0,
        isfloating: 0,
        isterminal: 0,
        noswallow: -1,
        monitor: -1,
    },
    Rule {
        class: cstr!("Chromium"),
        instance: ptr::null(),
        title: ptr::null(),
        tags: 0,
        isfloating: 0,
        isterminal: 0,
        noswallow: -1,
        monitor: -1,
    },
    Rule {
        class: cstr!("St"),
        instance: ptr::null(),
        title: ptr::null(),
        tags: 0,
        isfloating: 0,
        isterminal: 1,
        noswallow: 0,
        monitor: -1,
    },
    Rule {
        class: ptr::null(),
        instance: ptr::null(),
        title: cstr!("Event Tester"),
        tags: 0,
        isfloating: 0,
        isterminal: 0,
        noswallow: 1,
        monitor: -1,
    },
];

pub const MFACT: c_float = 0.55;
pub const NMASTER: c_int = 1;
pub const RESIZEHINTS: c_int = 1;

pub const LAYOUTS: [Layout; 2] = [
    Layout {
        symbol: cstr!("[]="),
        arrange: Some(crate::wm::tile),
    },
    Layout {
        symbol: cstr!("[M]"),
        arrange: Some(crate::wm::monocle),
    },
];

pub const MODKEY: c_uint = xlib::Mod4Mask;

pub const TERMCMD: [*const c_char; 4] = [
    cstr!("tabbed"),
    cstr!("-c"),
    cstr!("/home/federico/code/dotfiles/scripts/st_start_wininfo.sh"),
    ptr::null(),
];

pub const DMENURUN: [*const c_char; 4] = [
    cstr!("j4-dmenu-desktop"),
    cstr!("--term"),
    cstr!("st"),
    ptr::null(),
];

pub const BROWSER: [*const c_char; 2] = [cstr!("firefox"), ptr::null()];
pub const FILEMANAGER: [*const c_char; 3] = [cstr!("st"), cstr!("lf"), ptr::null()];
pub const CALENDAR: [*const c_char; 2] = [cstr!("gsimplecal"), ptr::null()];
pub const AUDIOCTL: [*const c_char; 2] = [cstr!("pavucontrol"), ptr::null()];
pub const BRIGHTNESS_UP: [*const c_char; 3] = [cstr!("goblight"), cstr!("+10"), ptr::null()];
pub const BRIGHTNESS_DOWN: [*const c_char; 3] = [cstr!("goblight"), cstr!("-10"), ptr::null()];

pub const SCREENSHOT_CMD: [*const c_char; 4] = [
    cstr!("/bin/sh"),
    cstr!("-c"),
    cstr!("maim --select /tmp/pic.png && mv /tmp/pic.png $HOME/Downloads/$(: | dmenu -i -p \"gib output name\").png"),
    ptr::null(),
];

pub const VOLUME_UP_CMD: [*const c_char; 4] = [
    cstr!("/bin/sh"),
    cstr!("-c"),
    cstr!("/home/federico/code/dotfiles/scripts/change_volume.sh 5%+"),
    ptr::null(),
];

pub const VOLUME_DOWN_CMD: [*const c_char; 4] = [
    cstr!("/bin/sh"),
    cstr!("-c"),
    cstr!("/home/federico/code/dotfiles/scripts/change_volume.sh 5%-"),
    ptr::null(),
];

pub const KEYS: &[Key] = &[
    Key {
        mod_: MODKEY,
        keysym: keysym::XK_r,
        func: Some(crate::wm::spawn),
        arg: Arg {
            v: DMENURUN.as_ptr().cast(),
        },
    },
    Key {
        mod_: MODKEY,
        keysym: keysym::XK_w,
        func: Some(crate::wm::spawn),
        arg: Arg {
            v: BROWSER.as_ptr().cast(),
        },
    },
    Key {
        mod_: MODKEY,
        keysym: keysym::XK_f,
        func: Some(crate::wm::spawn),
        arg: Arg {
            v: FILEMANAGER.as_ptr().cast(),
        },
    },
    Key {
        mod_: MODKEY,
        keysym: keysym::XK_c,
        func: Some(crate::wm::spawn),
        arg: Arg {
            v: CALENDAR.as_ptr().cast(),
        },
    },
    Key {
        mod_: MODKEY,
        keysym: keysym::XK_p,
        func: Some(crate::wm::spawn),
        arg: Arg {
            v: AUDIOCTL.as_ptr().cast(),
        },
    },
    Key {
        mod_: MODKEY | xlib::ShiftMask,
        keysym: keysym::XK_Return,
        func: Some(crate::wm::spawn),
        arg: Arg {
            v: TERMCMD.as_ptr().cast(),
        },
    },
    Key {
        mod_: MODKEY,
        keysym: keysym::XK_b,
        func: Some(crate::wm::togglebar),
        arg: Arg { i: 0 },
    },
    Key {
        mod_: MODKEY,
        keysym: keysym::XK_j,
        func: Some(crate::wm::focusstack),
        arg: Arg { i: 1 },
    },
    Key {
        mod_: MODKEY,
        keysym: keysym::XK_k,
        func: Some(crate::wm::focusstack),
        arg: Arg { i: -1 },
    },
    Key {
        mod_: MODKEY,
        keysym: keysym::XK_i,
        func: Some(crate::wm::incnmaster),
        arg: Arg { i: 1 },
    },
    Key {
        mod_: MODKEY,
        keysym: keysym::XK_d,
        func: Some(crate::wm::incnmaster),
        arg: Arg { i: -1 },
    },
    Key {
        mod_: MODKEY,
        keysym: keysym::XK_h,
        func: Some(crate::wm::setmfact),
        arg: Arg { f: -0.05 },
    },
    Key {
        mod_: MODKEY,
        keysym: keysym::XK_l,
        func: Some(crate::wm::setmfact),
        arg: Arg { f: 0.05 },
    },
    Key {
        mod_: MODKEY,
        keysym: keysym::XK_Return,
        func: Some(crate::wm::zoom),
        arg: Arg { i: 0 },
    },
    Key {
        mod_: MODKEY,
        keysym: keysym::XK_Tab,
        func: Some(crate::wm::view),
        arg: Arg { i: 0 },
    },
    Key {
        mod_: MODKEY | xlib::ShiftMask,
        keysym: keysym::XK_k,
        func: Some(crate::wm::view_adjacent),
        arg: Arg { i: 1 },
    },
    Key {
        mod_: MODKEY | xlib::ShiftMask,
        keysym: keysym::XK_j,
        func: Some(crate::wm::view_adjacent),
        arg: Arg { i: -1 },
    },
    Key {
        mod_: MODKEY | xlib::ShiftMask,
        keysym: keysym::XK_l,
        func: Some(crate::wm::movestack),
        arg: Arg { i: 1 },
    },
    Key {
        mod_: MODKEY | xlib::ShiftMask,
        keysym: keysym::XK_h,
        func: Some(crate::wm::movestack),
        arg: Arg { i: -1 },
    },
    Key {
        mod_: MODKEY | xlib::ShiftMask,
        keysym: keysym::XK_c,
        func: Some(crate::wm::killclient),
        arg: Arg { i: 0 },
    },
    Key {
        mod_: MODKEY,
        keysym: keysym::XK_t,
        func: Some(crate::wm::setlayout),
        arg: Arg {
            v: (&LAYOUTS[0] as *const Layout).cast(),
        },
    },
    Key {
        mod_: MODKEY,
        keysym: keysym::XK_m,
        func: Some(crate::wm::setlayout),
        arg: Arg {
            v: (&LAYOUTS[1] as *const Layout).cast(),
        },
    },
    Key {
        mod_: MODKEY,
        keysym: keysym::XK_space,
        func: Some(crate::wm::setlayout),
        arg: Arg { i: 0 },
    },
    Key {
        mod_: MODKEY | xlib::ShiftMask,
        keysym: keysym::XK_space,
        func: Some(crate::wm::togglefloating),
        arg: Arg { i: 0 },
    },
    Key {
        mod_: MODKEY,
        keysym: keysym::XK_0,
        func: Some(crate::wm::view),
        arg: Arg { ui: u32::MAX },
    },
    Key {
        mod_: MODKEY | xlib::ShiftMask,
        keysym: keysym::XK_0,
        func: Some(crate::wm::tag),
        arg: Arg { ui: u32::MAX },
    },
    Key {
        mod_: MODKEY,
        keysym: keysym::XK_comma,
        func: Some(crate::wm::focusmon),
        arg: Arg { i: -1 },
    },
    Key {
        mod_: MODKEY,
        keysym: keysym::XK_period,
        func: Some(crate::wm::focusmon),
        arg: Arg { i: 1 },
    },
    Key {
        mod_: MODKEY | xlib::ShiftMask,
        keysym: keysym::XK_comma,
        func: Some(crate::wm::tagmon),
        arg: Arg { i: -1 },
    },
    Key {
        mod_: MODKEY | xlib::ShiftMask,
        keysym: keysym::XK_period,
        func: Some(crate::wm::tagmon),
        arg: Arg { i: 1 },
    },
    Key {
        mod_: 0,
        keysym: keysym::XF86XK_MonBrightnessUp,
        func: Some(crate::wm::spawn),
        arg: Arg {
            v: BRIGHTNESS_UP.as_ptr().cast(),
        },
    },
    Key {
        mod_: 0,
        keysym: keysym::XF86XK_MonBrightnessDown,
        func: Some(crate::wm::spawn),
        arg: Arg {
            v: BRIGHTNESS_DOWN.as_ptr().cast(),
        },
    },
    Key {
        mod_: 0,
        keysym: PRINT_SCREEN_DWM,
        func: Some(crate::wm::spawn),
        arg: Arg {
            v: SCREENSHOT_CMD.as_ptr().cast(),
        },
    },
    Key {
        mod_: 0,
        keysym: keysym::XF86XK_AudioRaiseVolume,
        func: Some(crate::wm::spawn),
        arg: Arg {
            v: VOLUME_UP_CMD.as_ptr().cast(),
        },
    },
    Key {
        mod_: 0,
        keysym: keysym::XF86XK_AudioLowerVolume,
        func: Some(crate::wm::spawn),
        arg: Arg {
            v: VOLUME_DOWN_CMD.as_ptr().cast(),
        },
    },
    Key {
        mod_: MODKEY,
        keysym: keysym::XK_1,
        func: Some(crate::wm::view),
        arg: Arg { ui: 1 << 0 },
    },
    Key {
        mod_: MODKEY | xlib::ControlMask,
        keysym: keysym::XK_1,
        func: Some(crate::wm::toggleview),
        arg: Arg { ui: 1 << 0 },
    },
    Key {
        mod_: MODKEY | xlib::ShiftMask,
        keysym: keysym::XK_1,
        func: Some(crate::wm::tag),
        arg: Arg { ui: 1 << 0 },
    },
    Key {
        mod_: MODKEY | xlib::ControlMask | xlib::ShiftMask,
        keysym: keysym::XK_1,
        func: Some(crate::wm::toggletag),
        arg: Arg { ui: 1 << 0 },
    },
    Key {
        mod_: MODKEY,
        keysym: keysym::XK_2,
        func: Some(crate::wm::view),
        arg: Arg { ui: 1 << 1 },
    },
    Key {
        mod_: MODKEY | xlib::ControlMask,
        keysym: keysym::XK_2,
        func: Some(crate::wm::toggleview),
        arg: Arg { ui: 1 << 1 },
    },
    Key {
        mod_: MODKEY | xlib::ShiftMask,
        keysym: keysym::XK_2,
        func: Some(crate::wm::tag),
        arg: Arg { ui: 1 << 1 },
    },
    Key {
        mod_: MODKEY | xlib::ControlMask | xlib::ShiftMask,
        keysym: keysym::XK_2,
        func: Some(crate::wm::toggletag),
        arg: Arg { ui: 1 << 1 },
    },
    Key {
        mod_: MODKEY,
        keysym: keysym::XK_3,
        func: Some(crate::wm::view),
        arg: Arg { ui: 1 << 2 },
    },
    Key {
        mod_: MODKEY | xlib::ControlMask,
        keysym: keysym::XK_3,
        func: Some(crate::wm::toggleview),
        arg: Arg { ui: 1 << 2 },
    },
    Key {
        mod_: MODKEY | xlib::ShiftMask,
        keysym: keysym::XK_3,
        func: Some(crate::wm::tag),
        arg: Arg { ui: 1 << 2 },
    },
    Key {
        mod_: MODKEY | xlib::ControlMask | xlib::ShiftMask,
        keysym: keysym::XK_3,
        func: Some(crate::wm::toggletag),
        arg: Arg { ui: 1 << 2 },
    },
    Key {
        mod_: MODKEY,
        keysym: keysym::XK_4,
        func: Some(crate::wm::view),
        arg: Arg { ui: 1 << 3 },
    },
    Key {
        mod_: MODKEY | xlib::ControlMask,
        keysym: keysym::XK_4,
        func: Some(crate::wm::toggleview),
        arg: Arg { ui: 1 << 3 },
    },
    Key {
        mod_: MODKEY | xlib::ShiftMask,
        keysym: keysym::XK_4,
        func: Some(crate::wm::tag),
        arg: Arg { ui: 1 << 3 },
    },
    Key {
        mod_: MODKEY | xlib::ControlMask | xlib::ShiftMask,
        keysym: keysym::XK_4,
        func: Some(crate::wm::toggletag),
        arg: Arg { ui: 1 << 3 },
    },
    Key {
        mod_: MODKEY,
        keysym: keysym::XK_5,
        func: Some(crate::wm::view),
        arg: Arg { ui: 1 << 4 },
    },
    Key {
        mod_: MODKEY | xlib::ControlMask,
        keysym: keysym::XK_5,
        func: Some(crate::wm::toggleview),
        arg: Arg { ui: 1 << 4 },
    },
    Key {
        mod_: MODKEY | xlib::ShiftMask,
        keysym: keysym::XK_5,
        func: Some(crate::wm::tag),
        arg: Arg { ui: 1 << 4 },
    },
    Key {
        mod_: MODKEY | xlib::ControlMask | xlib::ShiftMask,
        keysym: keysym::XK_5,
        func: Some(crate::wm::toggletag),
        arg: Arg { ui: 1 << 4 },
    },
    Key {
        mod_: MODKEY,
        keysym: keysym::XK_6,
        func: Some(crate::wm::view),
        arg: Arg { ui: 1 << 5 },
    },
    Key {
        mod_: MODKEY | xlib::ControlMask,
        keysym: keysym::XK_6,
        func: Some(crate::wm::toggleview),
        arg: Arg { ui: 1 << 5 },
    },
    Key {
        mod_: MODKEY | xlib::ShiftMask,
        keysym: keysym::XK_6,
        func: Some(crate::wm::tag),
        arg: Arg { ui: 1 << 5 },
    },
    Key {
        mod_: MODKEY | xlib::ControlMask | xlib::ShiftMask,
        keysym: keysym::XK_6,
        func: Some(crate::wm::toggletag),
        arg: Arg { ui: 1 << 5 },
    },
    Key {
        mod_: MODKEY,
        keysym: keysym::XK_7,
        func: Some(crate::wm::view),
        arg: Arg { ui: 1 << 6 },
    },
    Key {
        mod_: MODKEY | xlib::ControlMask,
        keysym: keysym::XK_7,
        func: Some(crate::wm::toggleview),
        arg: Arg { ui: 1 << 6 },
    },
    Key {
        mod_: MODKEY | xlib::ShiftMask,
        keysym: keysym::XK_7,
        func: Some(crate::wm::tag),
        arg: Arg { ui: 1 << 6 },
    },
    Key {
        mod_: MODKEY | xlib::ControlMask | xlib::ShiftMask,
        keysym: keysym::XK_7,
        func: Some(crate::wm::toggletag),
        arg: Arg { ui: 1 << 6 },
    },
    Key {
        mod_: MODKEY,
        keysym: keysym::XK_8,
        func: Some(crate::wm::view),
        arg: Arg { ui: 1 << 7 },
    },
    Key {
        mod_: MODKEY | xlib::ControlMask,
        keysym: keysym::XK_8,
        func: Some(crate::wm::toggleview),
        arg: Arg { ui: 1 << 7 },
    },
    Key {
        mod_: MODKEY | xlib::ShiftMask,
        keysym: keysym::XK_8,
        func: Some(crate::wm::tag),
        arg: Arg { ui: 1 << 7 },
    },
    Key {
        mod_: MODKEY | xlib::ControlMask | xlib::ShiftMask,
        keysym: keysym::XK_8,
        func: Some(crate::wm::toggletag),
        arg: Arg { ui: 1 << 7 },
    },
    Key {
        mod_: MODKEY,
        keysym: keysym::XK_9,
        func: Some(crate::wm::view),
        arg: Arg { ui: 1 << 8 },
    },
    Key {
        mod_: MODKEY | xlib::ControlMask,
        keysym: keysym::XK_9,
        func: Some(crate::wm::toggleview),
        arg: Arg { ui: 1 << 8 },
    },
    Key {
        mod_: MODKEY | xlib::ShiftMask,
        keysym: keysym::XK_9,
        func: Some(crate::wm::tag),
        arg: Arg { ui: 1 << 8 },
    },
    Key {
        mod_: MODKEY | xlib::ControlMask | xlib::ShiftMask,
        keysym: keysym::XK_9,
        func: Some(crate::wm::toggletag),
        arg: Arg { ui: 1 << 8 },
    },
    Key {
        mod_: MODKEY | xlib::ShiftMask,
        keysym: keysym::XK_q,
        func: Some(crate::wm::quit),
        arg: Arg { i: 0 },
    },
];

pub const BUTTONS: &[Button] = &[
    Button {
        click: crate::types::CLK_LT_SYMBOL,
        mask: 0,
        button: xlib::Button1,
        func: Some(crate::wm::setlayout),
        arg: Arg { i: 0 },
    },
    Button {
        click: crate::types::CLK_LT_SYMBOL,
        mask: 0,
        button: xlib::Button3,
        func: Some(crate::wm::setlayout),
        arg: Arg {
            v: (&LAYOUTS[1] as *const Layout).cast(),
        },
    },
    Button {
        click: crate::types::CLK_WIN_TITLE,
        mask: 0,
        button: xlib::Button2,
        func: Some(crate::wm::zoom),
        arg: Arg { i: 0 },
    },
    Button {
        click: crate::types::CLK_STATUS_TEXT,
        mask: 0,
        button: xlib::Button2,
        func: Some(crate::wm::spawn),
        arg: Arg {
            v: TERMCMD.as_ptr().cast(),
        },
    },
    Button {
        click: crate::types::CLK_CLIENT_WIN,
        mask: MODKEY,
        button: xlib::Button1,
        func: Some(crate::wm::movemouse),
        arg: Arg { i: 0 },
    },
    Button {
        click: crate::types::CLK_CLIENT_WIN,
        mask: MODKEY,
        button: xlib::Button2,
        func: Some(crate::wm::togglefloating),
        arg: Arg { i: 0 },
    },
    Button {
        click: crate::types::CLK_CLIENT_WIN,
        mask: MODKEY,
        button: xlib::Button3,
        func: Some(crate::wm::resizemouse),
        arg: Arg { i: 0 },
    },
    Button {
        click: crate::types::CLK_TAG_BAR,
        mask: 0,
        button: xlib::Button1,
        func: Some(crate::wm::view),
        arg: Arg { i: 0 },
    },
    Button {
        click: crate::types::CLK_TAG_BAR,
        mask: 0,
        button: xlib::Button3,
        func: Some(crate::wm::toggleview),
        arg: Arg { i: 0 },
    },
    Button {
        click: crate::types::CLK_TAG_BAR,
        mask: MODKEY,
        button: xlib::Button1,
        func: Some(crate::wm::tag),
        arg: Arg { i: 0 },
    },
    Button {
        click: crate::types::CLK_TAG_BAR,
        mask: MODKEY,
        button: xlib::Button3,
        func: Some(crate::wm::toggletag),
        arg: Arg { i: 0 },
    },
];
