use crate::config;
use crate::drw::{self, Clr, Cur, Drw};
use crate::ffi;
use crate::types::*;
use crate::util;
use libc::{c_char, c_int, c_long, c_uint, c_ulong, pid_t};
use std::env;
use std::ffi::CStr;
use std::fs;
use std::mem;
use std::ptr;
use x11::xinerama;
use x11::xlib;
use x11::xlib_xcb;

const COL_BORDER: usize = 2;

const X_CONFIGURE_WINDOW: u8 = 12;
const X_GRAB_BUTTON: u8 = 28;
const X_GRAB_KEY: u8 = 33;
const X_SET_INPUT_FOCUS: u8 = 42;
const X_COPY_AREA: u8 = 62;
const X_POLY_SEGMENT: u8 = 66;
const X_POLY_FILL_RECTANGLE: u8 = 70;
const X_POLY_TEXT8: u8 = 74;

static BROKEN: &[u8] = b"broken\0";
static mut STATE: *mut State = ptr::null_mut();

struct State {
    stext: [c_char; 256],
    screen: c_int,
    sw: c_int,
    sh: c_int,
    bh: c_int,
    blw: c_int,
    lrpad: c_int,
    xerrorxlib: Option<unsafe extern "C" fn(*mut xlib::Display, *mut xlib::XErrorEvent) -> c_int>,
    numlockmask: c_uint,
    running: bool,
    cursor: [*mut Cur; CUR_LAST],
    scheme: Vec<*mut Clr>,
    dpy: *mut xlib::Display,
    drw: *mut Drw,
    mons: *mut Monitor,
    selmon: *mut Monitor,
    root: xlib::Window,
    wmcheckwin: xlib::Window,
    xcon: *mut xlib_xcb::xcb_connection_t,
    netatom: [xlib::Atom; NET_LAST],
    wmatom: [xlib::Atom; WM_LAST],
    motion_mon: *mut Monitor,
}

impl Default for State {
    fn default() -> Self {
        Self {
            stext: [0; 256],
            screen: 0,
            sw: 0,
            sh: 0,
            bh: 0,
            blw: 0,
            lrpad: 0,
            xerrorxlib: None,
            numlockmask: 0,
            running: true,
            cursor: [ptr::null_mut(); CUR_LAST],
            scheme: Vec::new(),
            dpy: ptr::null_mut(),
            drw: ptr::null_mut(),
            mons: ptr::null_mut(),
            selmon: ptr::null_mut(),
            root: 0,
            wmcheckwin: 0,
            xcon: ptr::null_mut(),
            netatom: [0; NET_LAST],
            wmatom: [0; WM_LAST],
            motion_mon: ptr::null_mut(),
        }
    }
}

unsafe fn state() -> &'static mut State {
    &mut *STATE
}

unsafe fn init_state() {
    STATE = Box::into_raw(Box::new(State::default()));
}

unsafe fn destroy_state() {
    if !STATE.is_null() {
        drop(Box::from_raw(STATE));
        STATE = ptr::null_mut();
    }
}

unsafe fn boxed_zeroed<T>() -> *mut T {
    Box::into_raw(Box::new(mem::zeroed()))
}

unsafe fn broken_ptr() -> *const c_char {
    BROKEN.as_ptr().cast()
}

fn tagmask() -> c_uint {
    (1u32 << config::tags().len()) - 1
}

unsafe fn cleanmask(mask: c_uint) -> c_uint {
    mask & !(state().numlockmask | xlib::LockMask)
        & (xlib::ShiftMask
            | xlib::ControlMask
            | xlib::Mod1Mask
            | xlib::Mod2Mask
            | xlib::Mod3Mask
            | xlib::Mod4Mask
            | xlib::Mod5Mask)
}

unsafe fn intersect(x: c_int, y: c_int, w: c_int, h: c_int, m: *mut Monitor) -> c_int {
    util::max_i32(
        0,
        util::min_i32(x + w, (*m).wx + (*m).ww) - util::max_i32(x, (*m).wx),
    ) * util::max_i32(
        0,
        util::min_i32(y + h, (*m).wy + (*m).wh) - util::max_i32(y, (*m).wy),
    )
}

unsafe fn isvisible(client: *mut Client) -> bool {
    ((*client).tags & (*(*client).mon).tagset[(*(*client).mon).seltags as usize]) != 0
}

unsafe fn width(client: *mut Client) -> c_int {
    (*client).w + 2 * (*client).bw
}

unsafe fn height(client: *mut Client) -> c_int {
    (*client).h + 2 * (*client).bw
}

unsafe fn textw(text: *const c_char) -> c_int {
    drw::drw_fontset_getwidth(state().drw, text) as c_int + state().lrpad
}

unsafe fn selected_layout(monitor: *mut Monitor) -> *const Layout {
    (*monitor).lt[(*monitor).sellt as usize]
}

unsafe fn layout_has_arrange(monitor: *mut Monitor) -> bool {
    !selected_layout(monitor).is_null() && (*selected_layout(monitor)).arrange.is_some()
}

unsafe fn is_monocle_layout(monitor: *mut Monitor) -> bool {
    selected_layout(monitor) == config::monocle_layout()
}

unsafe fn handle_event(event: *mut xlib::XEvent) {
    match (*event).get_type() {
        xlib::ButtonPress => buttonpress(event),
        xlib::ClientMessage => clientmessage(event),
        xlib::ConfigureRequest => configurerequest(event),
        xlib::ConfigureNotify => configurenotify(event),
        xlib::DestroyNotify => destroynotify(event),
        xlib::EnterNotify => enternotify(event),
        xlib::Expose => expose(event),
        xlib::FocusIn => focusin(event),
        xlib::KeyPress => keypress(event),
        xlib::MappingNotify => mappingnotify(event),
        xlib::MapRequest => maprequest(event),
        xlib::MotionNotify => motionnotify(event),
        xlib::PropertyNotify => propertynotify(event),
        xlib::UnmapNotify => unmapnotify(event),
        _ => {}
    }
}

pub unsafe fn applyrules(client: *mut Client) {
    let st = state();
    let mut class_hint: xlib::XClassHint = mem::zeroed();
    (*client).isfloating = 0;
    (*client).tags = 0;
    xlib::XGetClassHint(st.dpy, (*client).win, &mut class_hint);

    let class = if class_hint.res_class.is_null() {
        broken_ptr()
    } else {
        class_hint.res_class
    };
    let instance = if class_hint.res_name.is_null() {
        broken_ptr()
    } else {
        class_hint.res_name
    };

    for rule in config::rules().iter() {
        let title_match =
            rule.title.is_null() || !libc::strstr((*client).name.as_ptr(), rule.title).is_null();
        let class_match = rule.class.is_null() || !libc::strstr(class, rule.class).is_null();
        let instance_match =
            rule.instance.is_null() || !libc::strstr(instance, rule.instance).is_null();
        if title_match && class_match && instance_match {
            (*client).isterminal = rule.isterminal;
            (*client).noswallow = rule.noswallow;
            (*client).isfloating = rule.isfloating;
            (*client).tags |= rule.tags;
            let mut monitor = st.mons;
            while !monitor.is_null() && (*monitor).num != rule.monitor {
                monitor = (*monitor).next;
            }
            if !monitor.is_null() {
                (*client).mon = monitor;
            }
        }
    }

    if !class_hint.res_class.is_null() {
        xlib::XFree(class_hint.res_class.cast());
    }
    if !class_hint.res_name.is_null() {
        xlib::XFree(class_hint.res_name.cast());
    }

    if (*client).tags & tagmask() != 0 {
        (*client).tags &= tagmask();
    } else {
        (*client).tags = (*(*client).mon).tagset[(*(*client).mon).seltags as usize];
    }
}

pub unsafe fn applysizehints(
    client: *mut Client,
    x: &mut c_int,
    y: &mut c_int,
    w: &mut c_int,
    h: &mut c_int,
    interact: c_int,
) -> c_int {
    let st = state();
    let monitor = (*client).mon;
    let baseismin;

    *w = util::max_i32(1, *w);
    *h = util::max_i32(1, *h);
    if interact != 0 {
        if *x > st.sw {
            *x = st.sw - width(client);
        }
        if *y > st.sh {
            *y = st.sh - height(client);
        }
        if *x + *w + 2 * (*client).bw < 0 {
            *x = 0;
        }
        if *y + *h + 2 * (*client).bw < 0 {
            *y = 0;
        }
    } else {
        if *x >= (*monitor).wx + (*monitor).ww {
            *x = (*monitor).wx + (*monitor).ww - width(client);
        }
        if *y >= (*monitor).wy + (*monitor).wh {
            *y = (*monitor).wy + (*monitor).wh - height(client);
        }
        if *x + *w + 2 * (*client).bw <= (*monitor).wx {
            *x = (*monitor).wx;
        }
        if *y + *h + 2 * (*client).bw <= (*monitor).wy {
            *y = (*monitor).wy;
        }
    }
    if *h < st.bh {
        *h = st.bh;
    }
    if *w < st.bh {
        *w = st.bh;
    }
    if config::resizehints() != 0 || (*client).isfloating != 0 || !layout_has_arrange((*client).mon)
    {
        baseismin = (*client).basew == (*client).minw && (*client).baseh == (*client).minh;
        if !baseismin {
            *w -= (*client).basew;
            *h -= (*client).baseh;
        }
        if (*client).mina > 0.0 && (*client).maxa > 0.0 {
            if (*client).maxa < (*w as f32 / *h as f32) {
                *w = (*h as f32 * (*client).maxa + 0.5) as c_int;
            } else if (*client).mina < (*h as f32 / *w as f32) {
                *h = (*w as f32 * (*client).mina + 0.5) as c_int;
            }
        }
        if baseismin {
            *w -= (*client).basew;
            *h -= (*client).baseh;
        }
        if (*client).incw != 0 {
            *w -= *w % (*client).incw;
        }
        if (*client).inch != 0 {
            *h -= *h % (*client).inch;
        }
        *w = util::max_i32(*w + (*client).basew, (*client).minw);
        *h = util::max_i32(*h + (*client).baseh, (*client).minh);
        if (*client).maxw != 0 {
            *w = util::min_i32(*w, (*client).maxw);
        }
        if (*client).maxh != 0 {
            *h = util::min_i32(*h, (*client).maxh);
        }
    }

    ((*x != (*client).x) || (*y != (*client).y) || (*w != (*client).w) || (*h != (*client).h))
        as c_int
}

pub unsafe fn arrange(monitor: *mut Monitor) {
    let mut current = monitor;
    if !current.is_null() {
        showhide((*current).stack);
    } else {
        current = state().mons;
        while !current.is_null() {
            showhide((*current).stack);
            current = (*current).next;
        }
    }

    current = monitor;
    if !current.is_null() {
        arrangemon(current);
        restack(current);
    } else {
        current = state().mons;
        while !current.is_null() {
            arrangemon(current);
            current = (*current).next;
        }
    }
}

unsafe fn arrangemon(monitor: *mut Monitor) {
    util::copy_cstr(&mut (*monitor).ltsymbol, (*selected_layout(monitor)).symbol);
    if let Some(arrange_fn) = (*selected_layout(monitor)).arrange {
        arrange_fn(monitor);
    }
}

unsafe fn attach(client: *mut Client) {
    (*client).next = (*(*client).mon).clients;
    (*(*client).mon).clients = client;
}

unsafe fn attachstack(client: *mut Client) {
    (*client).snext = (*(*client).mon).stack;
    (*(*client).mon).stack = client;
}

unsafe fn swallow(parent: *mut Client, child: *mut Client) {
    if (*child).noswallow != 0 || (*child).isterminal != 0 {
        return;
    }
    if (*child).noswallow != 0 && config::swallowfloating() == 0 && (*child).isfloating != 0 {
        return;
    }

    detach(child);
    detachstack(child);

    setclientstate(child, ffi::WITHDRAWN_STATE);
    xlib::XUnmapWindow(state().dpy, (*parent).win);

    (*parent).swallowing = child;
    (*child).mon = (*parent).mon;

    let window = (*parent).win;
    (*parent).win = (*child).win;
    (*child).win = window;
    updatetitle(parent);
    xlib::XMoveResizeWindow(
        state().dpy,
        (*parent).win,
        (*parent).x,
        (*parent).y,
        (*parent).w as c_uint,
        (*parent).h as c_uint,
    );
    arrange((*parent).mon);
    configure(parent);
    updateclientlist();
}

unsafe fn unswallow(client: *mut Client) {
    (*client).win = (*(*client).swallowing).win;
    drop(Box::from_raw((*client).swallowing));
    (*client).swallowing = ptr::null_mut();

    setfullscreen(client, 0);
    updatetitle(client);
    arrange((*client).mon);
    xlib::XMapWindow(state().dpy, (*client).win);
    xlib::XMoveResizeWindow(
        state().dpy,
        (*client).win,
        (*client).x,
        (*client).y,
        (*client).w as c_uint,
        (*client).h as c_uint,
    );
    setclientstate(client, ffi::NORMAL_STATE);
    focus(ptr::null_mut());
    arrange((*client).mon);
}

unsafe fn buttonpress(event: *mut xlib::XEvent) {
    let st = state();
    let ev = &(*event).button;
    let mut arg = Arg { i: 0 };
    let mut click = CLK_ROOT_WIN;

    let monitor = wintomon(ev.window);
    if !monitor.is_null() && monitor != st.selmon {
        unfocus((*st.selmon).sel, 1);
        st.selmon = monitor;
        focus(ptr::null_mut());
    }

    if ev.window == (*st.selmon).barwin {
        let mut index = 0usize;
        let mut x = 0;
        loop {
            x += textw(config::tags()[index]);
            if !(ev.x >= x && index + 1 < config::tags().len()) {
                break;
            }
            index += 1;
        }
        if index < config::tags().len() {
            click = CLK_TAG_BAR;
            arg.ui = 1 << index;
        } else if ev.x < x + st.blw {
            click = CLK_LT_SYMBOL;
        } else if ev.x > (*st.selmon).ww - textw(st.stext.as_ptr()) {
            click = CLK_STATUS_TEXT;
        } else {
            click = CLK_WIN_TITLE;
        }
    } else {
        let client = wintoclient(ev.window);
        if !client.is_null() {
            focus(client);
            restack(st.selmon);
            xlib::XAllowEvents(st.dpy, xlib::ReplayPointer, xlib::CurrentTime);
            click = CLK_CLIENT_WIN;
        }
    }

    for button in config::buttons().iter() {
        if click == button.click
            && button.func.is_some()
            && button.button == ev.button
            && cleanmask(button.mask) == cleanmask(ev.state)
        {
            let arg_ptr = if click == CLK_TAG_BAR && unsafe { button.arg.i } == 0 {
                &arg as *const Arg
            } else {
                &button.arg as *const Arg
            };
            (button.func.unwrap())(arg_ptr);
        }
    }
}

unsafe fn checkotherwm() {
    let st = state();
    st.xerrorxlib = xlib::XSetErrorHandler(Some(xerrorstart));
    xlib::XSelectInput(
        st.dpy,
        xlib::XDefaultRootWindow(st.dpy),
        xlib::SubstructureRedirectMask,
    );
    xlib::XSync(st.dpy, xlib::False);
    xlib::XSetErrorHandler(Some(xerror));
    xlib::XSync(st.dpy, xlib::False);
}

unsafe fn cleanup() {
    let st = state();
    let arg = Arg { ui: u32::MAX };
    let mut foo = Layout {
        symbol: b"\0".as_ptr().cast(),
        arrange: None,
    };

    view(&arg);
    (*st.selmon).lt[(*st.selmon).sellt as usize] = &mut foo;
    let mut monitor = st.mons;
    while !monitor.is_null() {
        while !(*monitor).stack.is_null() {
            unmanage((*monitor).stack, 0);
        }
        monitor = (*monitor).next;
    }

    xlib::XUngrabKey(st.dpy, xlib::AnyKey, xlib::AnyModifier, st.root);
    while !st.mons.is_null() {
        cleanupmon(st.mons);
    }
    for cursor in st.cursor {
        drw::drw_cur_free(st.drw, cursor);
    }
    for scheme in st.scheme.drain(..) {
        if !scheme.is_null() {
            let slice = ptr::slice_from_raw_parts_mut(scheme, config::colors()[0].len());
            drop(Box::from_raw(slice));
        }
    }
    if !st.drw.is_null() && !(*st.drw).fonts.is_null() {
        drw::drw_fontset_free((*st.drw).fonts);
    }
    xlib::XDestroyWindow(st.dpy, st.wmcheckwin);
    drw::drw_free(st.drw);
    xlib::XSync(st.dpy, xlib::False);
    xlib::XSetInputFocus(
        st.dpy,
        xlib::PointerRoot as c_ulong,
        xlib::RevertToPointerRoot,
        xlib::CurrentTime,
    );
    xlib::XDeleteProperty(st.dpy, st.root, st.netatom[NET_ACTIVE_WINDOW]);
}

unsafe fn cleanupmon(monitor: *mut Monitor) {
    let st = state();
    if monitor == st.mons {
        st.mons = (*monitor).next;
    } else {
        let mut current = st.mons;
        while !current.is_null() && (*current).next != monitor {
            current = (*current).next;
        }
        if !current.is_null() {
            (*current).next = (*monitor).next;
        }
    }
    xlib::XUnmapWindow(st.dpy, (*monitor).barwin);
    xlib::XDestroyWindow(st.dpy, (*monitor).barwin);
    drop(Box::from_raw(monitor));
}

unsafe fn clientmessage(event: *mut xlib::XEvent) {
    let st = state();
    let ev = &(*event).client_message;
    let client = wintoclient(ev.window);
    if client.is_null() {
        return;
    }

    if ev.message_type == st.netatom[NET_WM_STATE] {
        let data = ev.data.as_longs();
        if data[1] as c_ulong == st.netatom[NET_WM_FULLSCREEN]
            || data[2] as c_ulong == st.netatom[NET_WM_FULLSCREEN]
        {
            setfullscreen(
                client,
                (data[0] == 1 || (data[0] == 2 && (*client).isfullscreen == 0)) as c_int,
            );
        }
    } else if ev.message_type == st.netatom[NET_ACTIVE_WINDOW] {
        if client != (*st.selmon).sel && (*client).isurgent == 0 {
            seturgent(client, 1);
        }
    }
}

unsafe fn configure(client: *mut Client) {
    let st = state();
    let mut ce: xlib::XConfigureEvent = mem::zeroed();
    ce.type_ = xlib::ConfigureNotify;
    ce.display = st.dpy;
    ce.event = (*client).win;
    ce.window = (*client).win;
    ce.x = (*client).x;
    ce.y = (*client).y;
    ce.width = (*client).w;
    ce.height = (*client).h;
    ce.border_width = (*client).bw;
    ce.above = 0;
    ce.override_redirect = xlib::False;
    xlib::XSendEvent(
        st.dpy,
        (*client).win,
        xlib::False,
        xlib::StructureNotifyMask,
        (&mut ce as *mut xlib::XConfigureEvent).cast(),
    );
}

unsafe fn configurenotify(event: *mut xlib::XEvent) {
    let st = state();
    let ev = &(*event).configure;
    if ev.window == st.root {
        let dirty = st.sw != ev.width || st.sh != ev.height;
        st.sw = ev.width;
        st.sh = ev.height;
        if updategeom() != 0 || dirty {
            drw::drw_resize(st.drw, st.sw as c_uint, st.bh as c_uint);
            updatebars();
            let mut monitor = st.mons;
            while !monitor.is_null() {
                let mut client = (*monitor).clients;
                while !client.is_null() {
                    if (*client).isfullscreen != 0 {
                        resizeclient(
                            client,
                            (*monitor).mx,
                            (*monitor).my,
                            (*monitor).mw,
                            (*monitor).mh,
                        );
                    }
                    client = (*client).next;
                }
                xlib::XMoveResizeWindow(
                    st.dpy,
                    (*monitor).barwin,
                    (*monitor).wx,
                    (*monitor).by,
                    (*monitor).ww as c_uint,
                    st.bh as c_uint,
                );
                monitor = (*monitor).next;
            }
            focus(ptr::null_mut());
            arrange(ptr::null_mut());
        }
    }
}

unsafe fn configurerequest(event: *mut xlib::XEvent) {
    let st = state();
    let ev = &(*event).configure_request;
    let client = wintoclient(ev.window);
    let mut wc: xlib::XWindowChanges = mem::zeroed();

    if !client.is_null() {
        if ev.value_mask & xlib::CWBorderWidth as c_ulong != 0 {
            (*client).bw = ev.border_width;
        } else if (*client).isfloating != 0 || !layout_has_arrange(st.selmon) {
            let monitor = (*client).mon;
            if ev.value_mask & xlib::CWX as c_ulong != 0 {
                (*client).oldx = (*client).x;
                (*client).x = (*monitor).mx + ev.x;
            }
            if ev.value_mask & xlib::CWY as c_ulong != 0 {
                (*client).oldy = (*client).y;
                (*client).y = (*monitor).my + ev.y;
            }
            if ev.value_mask & xlib::CWWidth as c_ulong != 0 {
                (*client).oldw = (*client).w;
                (*client).w = ev.width;
            }
            if ev.value_mask & xlib::CWHeight as c_ulong != 0 {
                (*client).oldh = (*client).h;
                (*client).h = ev.height;
            }
            if (*client).x + (*client).w > (*monitor).mx + (*monitor).mw
                && (*client).isfloating != 0
            {
                (*client).x = (*monitor).mx + ((*monitor).mw / 2 - width(client) / 2);
            }
            if (*client).y + (*client).h > (*monitor).my + (*monitor).mh
                && (*client).isfloating != 0
            {
                (*client).y = (*monitor).my + ((*monitor).mh / 2 - height(client) / 2);
            }
            if ev.value_mask & ((xlib::CWX | xlib::CWY) as c_ulong) != 0
                && ev.value_mask & ((xlib::CWWidth | xlib::CWHeight) as c_ulong) == 0
            {
                configure(client);
            }
            if isvisible(client) {
                xlib::XMoveResizeWindow(
                    st.dpy,
                    (*client).win,
                    (*client).x,
                    (*client).y,
                    (*client).w as c_uint,
                    (*client).h as c_uint,
                );
            }
        } else {
            configure(client);
        }
    } else {
        wc.x = ev.x;
        wc.y = ev.y;
        wc.width = ev.width;
        wc.height = ev.height;
        wc.border_width = ev.border_width;
        wc.sibling = ev.above;
        wc.stack_mode = ev.detail;
        xlib::XConfigureWindow(st.dpy, ev.window, ev.value_mask as c_uint, &mut wc);
    }

    xlib::XSync(st.dpy, xlib::False);
}

unsafe fn createmon() -> *mut Monitor {
    let monitor = boxed_zeroed::<Monitor>();
    (*monitor).tagset[0] = 1;
    (*monitor).tagset[1] = 1;
    (*monitor).mfact = config::mfact();
    (*monitor).nmaster = config::nmaster();
    (*monitor).showbar = config::showbar();
    (*monitor).topbar = config::topbar();
    (*monitor).lt[0] = &config::layouts()[0];
    (*monitor).lt[1] = &config::layouts()[1 % config::layouts().len()];
    util::copy_cstr(&mut (*monitor).ltsymbol, config::layouts()[0].symbol);
    monitor
}

unsafe fn destroynotify(event: *mut xlib::XEvent) {
    let ev = &(*event).destroy_window;
    let client = wintoclient(ev.window);
    if !client.is_null() {
        unmanage(client, 1);
    } else {
        let swallowing = swallowingclient(ev.window);
        if !swallowing.is_null() {
            unmanage((*swallowing).swallowing, 1);
        }
    }
}

unsafe fn detach(client: *mut Client) {
    let mut current = &mut (*(*client).mon).clients as *mut *mut Client;
    while !(*current).is_null() && *current != client {
        current = &mut (**current).next;
    }
    *current = (*client).next;
}

unsafe fn detachstack(client: *mut Client) {
    let mut current = &mut (*(*client).mon).stack as *mut *mut Client;
    while !(*current).is_null() && *current != client {
        current = &mut (**current).snext;
    }
    *current = (*client).snext;

    if client == (*(*client).mon).sel {
        let mut t = (*(*client).mon).stack;
        while !t.is_null() && !isvisible(t) {
            t = (*t).snext;
        }
        (*(*client).mon).sel = t;
    }
}

unsafe fn dirtomon(dir: c_int) -> *mut Monitor {
    let st = state();
    if dir > 0 {
        if !(*st.selmon).next.is_null() {
            (*st.selmon).next
        } else {
            st.mons
        }
    } else if st.selmon == st.mons {
        let mut monitor = st.mons;
        while !(*monitor).next.is_null() {
            monitor = (*monitor).next;
        }
        monitor
    } else {
        let mut monitor = st.mons;
        while !(*monitor).next.is_null() && (*monitor).next != st.selmon {
            monitor = (*monitor).next;
        }
        monitor
    }
}

unsafe fn drawbar(monitor: *mut Monitor) {
    let st = state();
    let boxs = (*(*st.drw).fonts).h as c_int / 9;
    let boxw = (*(*st.drw).fonts).h as c_int / 6 + 2;
    let mut occ: c_uint = 0;
    let mut urg: c_uint = 0;
    let mut sw = 0;

    if monitor == st.selmon {
        drw::drw_setscheme(st.drw, st.scheme[SCHEME_NORM]);
        sw = textw(st.stext.as_ptr()) - st.lrpad + 2;
        drw::drw_text(
            st.drw,
            (*monitor).ww - sw,
            0,
            sw as c_uint,
            st.bh as c_uint,
            0,
            st.stext.as_ptr(),
            0,
        );
    }

    let mut client = (*monitor).clients;
    while !client.is_null() {
        occ |= (*client).tags;
        if (*client).isurgent != 0 {
            urg |= (*client).tags;
        }
        client = (*client).next;
    }

    let mut x = 0;
    for (index, tag) in config::tags().iter().copied().enumerate() {
        let w = textw(tag);
        let scheme = if (*monitor).tagset[(*monitor).seltags as usize] & (1 << index) != 0 {
            st.scheme[SCHEME_SEL]
        } else {
            st.scheme[SCHEME_NORM]
        };
        drw::drw_setscheme(st.drw, scheme);
        drw::drw_text(
            st.drw,
            x,
            0,
            w as c_uint,
            st.bh as c_uint,
            (st.lrpad / 2) as c_uint,
            tag,
            (urg & (1 << index) != 0) as c_int,
        );
        if occ & (1 << index) != 0 {
            drw::drw_rect(
                st.drw,
                x + boxs,
                boxs,
                boxw as c_uint,
                boxw as c_uint,
                (monitor == st.selmon
                    && !(*st.selmon).sel.is_null()
                    && (*(*st.selmon).sel).tags & (1 << index) != 0) as c_int,
                (urg & (1 << index) != 0) as c_int,
            );
        }
        x += w;
    }

    st.blw = textw((*monitor).ltsymbol.as_ptr());
    let w = st.blw;
    drw::drw_setscheme(st.drw, st.scheme[SCHEME_NORM]);
    x = drw::drw_text(
        st.drw,
        x,
        0,
        w as c_uint,
        st.bh as c_uint,
        (st.lrpad / 2) as c_uint,
        (*monitor).ltsymbol.as_ptr(),
        0,
    );

    let title_width = (*monitor).ww - sw - x;
    if title_width > st.bh {
        if !(*monitor).sel.is_null() {
            let scheme = if monitor == st.selmon {
                st.scheme[SCHEME_SEL]
            } else {
                st.scheme[SCHEME_NORM]
            };
            drw::drw_setscheme(st.drw, scheme);
            drw::drw_text(
                st.drw,
                x,
                0,
                title_width as c_uint,
                st.bh as c_uint,
                (st.lrpad / 2) as c_uint,
                (*(*monitor).sel).name.as_ptr(),
                0,
            );
            if (*(*monitor).sel).isfloating != 0 {
                drw::drw_rect(
                    st.drw,
                    x + boxs,
                    boxs,
                    boxw as c_uint,
                    boxw as c_uint,
                    (*(*monitor).sel).isfixed,
                    0,
                );
            }
        } else {
            drw::drw_setscheme(st.drw, st.scheme[SCHEME_NORM]);
            drw::drw_rect(st.drw, x, 0, title_width as c_uint, st.bh as c_uint, 1, 1);
        }
    }
    drw::drw_map(
        st.drw,
        (*monitor).barwin,
        0,
        0,
        (*monitor).ww as c_uint,
        st.bh as c_uint,
    );
}

unsafe fn drawbars() {
    let mut monitor = state().mons;
    while !monitor.is_null() {
        drawbar(monitor);
        monitor = (*monitor).next;
    }
}

unsafe fn enternotify(event: *mut xlib::XEvent) {
    let st = state();
    let ev = &(*event).crossing;
    if (ev.mode != xlib::NotifyNormal || ev.detail == xlib::NotifyInferior) && ev.window != st.root
    {
        return;
    }
    let client = wintoclient(ev.window);
    let monitor = if client.is_null() {
        wintomon(ev.window)
    } else {
        (*client).mon
    };
    if monitor != st.selmon {
        unfocus((*st.selmon).sel, 1);
        st.selmon = monitor;
    } else if client.is_null() || client == (*st.selmon).sel {
        return;
    }
    focus(client);
}

unsafe fn expose(event: *mut xlib::XEvent) {
    let ev = &(*event).expose;
    if ev.count == 0 {
        let monitor = wintomon(ev.window);
        if !monitor.is_null() {
            drawbar(monitor);
        }
    }
}

unsafe fn focus(mut client: *mut Client) {
    let st = state();
    if client.is_null() || !isvisible(client) {
        client = (*st.selmon).stack;
        while !client.is_null() && !isvisible(client) {
            client = (*client).snext;
        }
    }

    if !(*st.selmon).sel.is_null() && (*st.selmon).sel != client {
        unfocus((*st.selmon).sel, 0);
    }
    if !client.is_null() {
        if (*client).mon != st.selmon {
            st.selmon = (*client).mon;
        }
        if (*client).isurgent != 0 {
            seturgent(client, 0);
        }
        detachstack(client);
        attachstack(client);
        grabbuttons(client, 1);
        xlib::XSetWindowBorder(
            st.dpy,
            (*client).win,
            (*st.scheme[SCHEME_SEL].add(COL_BORDER)).pixel,
        );
        setfocus(client);
    } else {
        xlib::XSetInputFocus(
            st.dpy,
            st.root,
            xlib::RevertToPointerRoot,
            xlib::CurrentTime,
        );
        xlib::XDeleteProperty(st.dpy, st.root, st.netatom[NET_ACTIVE_WINDOW]);
    }
    (*st.selmon).sel = client;
    drawbars();
}

unsafe fn focusin(event: *mut xlib::XEvent) {
    let st = state();
    let ev = &(*event).focus_change;
    if !(*st.selmon).sel.is_null() && ev.window != (*(*st.selmon).sel).win {
        setfocus((*st.selmon).sel);
    }
}

pub unsafe fn focusmon(arg: *const Arg) {
    let st = state();
    if (*st.mons).next.is_null() {
        return;
    }
    let monitor = dirtomon((*arg).i);
    if monitor == st.selmon {
        return;
    }
    unfocus((*st.selmon).sel, 0);
    st.selmon = monitor;
    focus(ptr::null_mut());
}

pub unsafe fn focusstack(arg: *const Arg) {
    let st = state();
    if (*st.selmon).sel.is_null() {
        return;
    }

    let mut client = ptr::null_mut();
    if (*arg).i > 0 {
        client = (*(*st.selmon).sel).next;
        while !client.is_null() && !isvisible(client) {
            client = (*client).next;
        }
        if client.is_null() {
            client = (*st.selmon).clients;
            while !client.is_null() && !isvisible(client) {
                client = (*client).next;
            }
        }
    } else {
        let mut iter = (*st.selmon).clients;
        while !iter.is_null() && iter != (*st.selmon).sel {
            if isvisible(iter) {
                client = iter;
            }
            iter = (*iter).next;
        }
        if client.is_null() {
            while !iter.is_null() {
                if isvisible(iter) {
                    client = iter;
                }
                iter = (*iter).next;
            }
        }
    }

    if !client.is_null() {
        focus(client);
        restack(st.selmon);
    }
}

unsafe fn getatomprop(client: *mut Client, prop: xlib::Atom) -> xlib::Atom {
    let st = state();
    let mut da = 0;
    let mut di = 0;
    let mut dl = 0;
    let mut extra = 0;
    let mut p: *mut u8 = ptr::null_mut();
    let mut atom = 0;

    if xlib::XGetWindowProperty(
        st.dpy,
        (*client).win,
        prop,
        0,
        mem::size_of::<xlib::Atom>() as c_long,
        xlib::False,
        xlib::XA_ATOM,
        &mut da,
        &mut di,
        &mut dl,
        &mut extra,
        &mut p,
    ) == xlib::Success as c_int
        && !p.is_null()
    {
        atom = *(p as *mut xlib::Atom);
        xlib::XFree(p.cast());
    }
    atom
}

unsafe fn getrootptr(x: &mut c_int, y: &mut c_int) -> c_int {
    let st = state();
    let mut di = 0;
    let mut dui = 0;
    let mut dummy = 0;
    xlib::XQueryPointer(
        st.dpy, st.root, &mut dummy, &mut dummy, x, y, &mut di, &mut di, &mut dui,
    )
}

unsafe fn getstate(window: xlib::Window) -> c_long {
    let st = state();
    let mut format = 0;
    let mut result = -1;
    let mut p: *mut u8 = ptr::null_mut();
    let mut n = 0;
    let mut extra = 0;
    let mut real = 0;
    if xlib::XGetWindowProperty(
        st.dpy,
        window,
        st.wmatom[WM_STATE],
        0,
        2,
        xlib::False,
        st.wmatom[WM_STATE],
        &mut real,
        &mut format,
        &mut n,
        &mut extra,
        &mut p,
    ) != xlib::Success as c_int
    {
        return -1;
    }
    if n != 0 {
        result = *(p as *mut u8) as c_long;
    }
    if !p.is_null() {
        xlib::XFree(p.cast());
    }
    result
}

unsafe fn gettextprop(window: xlib::Window, atom: xlib::Atom, text: &mut [c_char]) -> c_int {
    let st = state();
    if text.is_empty() {
        return 0;
    }
    text.fill(0);
    let mut name: xlib::XTextProperty = mem::zeroed();
    if xlib::XGetTextProperty(st.dpy, window, &mut name, atom) == 0 || name.nitems == 0 {
        return 0;
    }

    if name.encoding == xlib::XA_STRING {
        util::copy_cstr(text, name.value.cast());
    } else {
        let mut list: *mut *mut c_char = ptr::null_mut();
        let mut n = 0;
        if xlib::XmbTextPropertyToTextList(st.dpy, &name, &mut list, &mut n)
            >= xlib::Success as c_int
            && n > 0
            && !list.is_null()
            && !(*list).is_null()
        {
            util::copy_cstr(text, *list);
            xlib::XFreeStringList(list);
        }
    }
    xlib::XFree(name.value.cast());
    1
}

unsafe fn grabbuttons(client: *mut Client, focused: c_int) {
    let st = state();
    updatenumlockmask();
    let modifiers = [
        0,
        xlib::LockMask,
        st.numlockmask,
        st.numlockmask | xlib::LockMask,
    ];
    xlib::XUngrabButton(
        st.dpy,
        xlib::AnyButton as c_uint,
        xlib::AnyModifier,
        (*client).win,
    );
    if focused == 0 {
        xlib::XGrabButton(
            st.dpy,
            xlib::AnyButton as c_uint,
            xlib::AnyModifier,
            (*client).win,
            xlib::False,
            (xlib::ButtonPressMask | xlib::ButtonReleaseMask) as c_uint,
            xlib::GrabModeSync,
            xlib::GrabModeSync,
            0,
            0,
        );
    }
    for button in config::buttons().iter() {
        if button.click == CLK_CLIENT_WIN {
            for modifier in modifiers {
                xlib::XGrabButton(
                    st.dpy,
                    button.button,
                    button.mask | modifier,
                    (*client).win,
                    xlib::False,
                    (xlib::ButtonPressMask | xlib::ButtonReleaseMask) as c_uint,
                    xlib::GrabModeAsync,
                    xlib::GrabModeSync,
                    0,
                    0,
                );
            }
        }
    }
}

unsafe fn grabkeys() {
    let st = state();
    updatenumlockmask();
    let modifiers = [
        0,
        xlib::LockMask,
        st.numlockmask,
        st.numlockmask | xlib::LockMask,
    ];
    xlib::XUngrabKey(st.dpy, xlib::AnyKey, xlib::AnyModifier, st.root);
    for key in config::keys().iter() {
        let code = xlib::XKeysymToKeycode(st.dpy, key.keysym as c_ulong);
        if code != 0 {
            for modifier in modifiers {
                xlib::XGrabKey(
                    st.dpy,
                    code as c_int,
                    key.mod_ | modifier,
                    st.root,
                    xlib::True,
                    xlib::GrabModeAsync,
                    xlib::GrabModeAsync,
                );
            }
        }
    }
}

pub unsafe fn incnmaster(arg: *const Arg) {
    let st = state();
    (*st.selmon).nmaster = util::max_i32((*st.selmon).nmaster + (*arg).i, 0);
    arrange(st.selmon);
}

unsafe fn isuniquegeom(
    unique: &[xinerama::XineramaScreenInfo],
    info: &xinerama::XineramaScreenInfo,
) -> bool {
    !unique.iter().any(|item| {
        item.x_org == info.x_org
            && item.y_org == info.y_org
            && item.width == info.width
            && item.height == info.height
    })
}

unsafe fn keypress(event: *mut xlib::XEvent) {
    let st = state();
    let ev = &(*event).key;
    let keysym = xlib::XKeycodeToKeysym(st.dpy, ev.keycode as u8, 0) as c_uint;
    for key in config::keys().iter() {
        if key.keysym == keysym && cleanmask(key.mod_) == cleanmask(ev.state) {
            if let Some(func) = key.func {
                func(&key.arg);
            }
        }
    }
}

pub unsafe fn killclient(_arg: *const Arg) {
    let st = state();
    if (*st.selmon).sel.is_null() {
        return;
    }
    if sendevent((*st.selmon).sel, st.wmatom[WM_DELETE]) == 0 {
        xlib::XGrabServer(st.dpy);
        xlib::XSetErrorHandler(Some(xerrordummy));
        xlib::XSetCloseDownMode(st.dpy, xlib::DestroyAll);
        xlib::XKillClient(st.dpy, (*(*st.selmon).sel).win);
        xlib::XSync(st.dpy, xlib::False);
        xlib::XSetErrorHandler(Some(xerror));
        xlib::XUngrabServer(st.dpy);
    }
}

unsafe fn manage(window: xlib::Window, wa: *mut xlib::XWindowAttributes) {
    let st = state();
    let client = boxed_zeroed::<Client>();
    let mut transient: xlib::Window = 0;
    let mut term = ptr::null_mut();
    let mut t = ptr::null_mut();
    let mut wc: xlib::XWindowChanges = mem::zeroed();

    (*client).win = window;
    (*client).pid = winpid(window);
    (*client).x = (*wa).x;
    (*client).oldx = (*wa).x;
    (*client).y = (*wa).y;
    (*client).oldy = (*wa).y;
    (*client).w = (*wa).width;
    (*client).oldw = (*wa).width;
    (*client).h = (*wa).height;
    (*client).oldh = (*wa).height;
    (*client).oldbw = (*wa).border_width;

    updatetitle(client);
    if xlib::XGetTransientForHint(st.dpy, window, &mut transient) != 0 {
        t = wintoclient(transient);
    }

    if !t.is_null() {
        (*client).mon = (*t).mon;
        (*client).tags = (*t).tags;
    } else {
        (*client).mon = st.selmon;
        applyrules(client);
        term = termforwin(client);
    }

    if (*client).x + width(client) > (*(*client).mon).mx + (*(*client).mon).mw {
        (*client).x = (*(*client).mon).mx + (*(*client).mon).mw - width(client);
    }
    if (*client).y + height(client) > (*(*client).mon).my + (*(*client).mon).mh {
        (*client).y = (*(*client).mon).my + (*(*client).mon).mh - height(client);
    }
    (*client).x = util::max_i32((*client).x, (*(*client).mon).mx);
    (*client).y = util::max_i32(
        (*client).y,
        if (*(*client).mon).by == (*(*client).mon).my
            && (*client).x + (*client).w / 2 >= (*(*client).mon).wx
            && (*client).x + (*client).w / 2 < (*(*client).mon).wx + (*(*client).mon).ww
        {
            st.bh
        } else {
            (*(*client).mon).my
        },
    );
    (*client).bw = config::borderpx() as c_int;

    wc.border_width = (*client).bw;
    xlib::XConfigureWindow(st.dpy, window, xlib::CWBorderWidth as c_uint, &mut wc);
    xlib::XSetWindowBorder(
        st.dpy,
        window,
        (*st.scheme[SCHEME_NORM].add(COL_BORDER)).pixel,
    );
    configure(client);
    updatewindowtype(client);
    updatesizehints(client);
    updatewmhints(client);
    xlib::XSelectInput(
        st.dpy,
        window,
        xlib::EnterWindowMask
            | xlib::FocusChangeMask
            | xlib::PropertyChangeMask
            | xlib::StructureNotifyMask,
    );
    grabbuttons(client, 0);
    if (*client).isfloating == 0 {
        (*client).isfloating = (transient != 0 || (*client).isfixed != 0) as c_int;
        (*client).oldstate = (*client).isfloating;
    }
    if (*client).isfloating != 0 {
        xlib::XRaiseWindow(st.dpy, (*client).win);
    }
    attach(client);
    attachstack(client);
    xlib::XChangeProperty(
        st.dpy,
        st.root,
        st.netatom[NET_CLIENT_LIST],
        xlib::XA_WINDOW,
        32,
        xlib::PropModeAppend,
        (&(*client).win as *const xlib::Window).cast(),
        1,
    );
    xlib::XMoveResizeWindow(
        st.dpy,
        (*client).win,
        (*client).x + 2 * st.sw,
        (*client).y,
        (*client).w as c_uint,
        (*client).h as c_uint,
    );
    setclientstate(client, ffi::NORMAL_STATE);
    if (*client).mon == st.selmon {
        unfocus((*st.selmon).sel, 0);
    }
    (*(*client).mon).sel = client;
    arrange((*client).mon);
    xlib::XMapWindow(st.dpy, (*client).win);
    if !term.is_null() {
        swallow(term, client);
    }
    focus(ptr::null_mut());
}

unsafe fn mappingnotify(event: *mut xlib::XEvent) {
    let ev = &(*event).mapping;
    xlib::XRefreshKeyboardMapping(ev as *const _ as *mut _);
    if ev.request == xlib::MappingKeyboard {
        grabkeys();
    }
}

unsafe fn maprequest(event: *mut xlib::XEvent) {
    let st = state();
    let ev = &(*event).map_request;
    let mut wa: xlib::XWindowAttributes = mem::zeroed();
    if xlib::XGetWindowAttributes(st.dpy, ev.window, &mut wa) == 0 || wa.override_redirect != 0 {
        return;
    }
    if wintoclient(ev.window).is_null() {
        manage(ev.window, &mut wa);
    }
}

pub unsafe fn monocle(monitor: *mut Monitor) {
    let mut n = 0;
    let mut client = (*monitor).clients;
    while !client.is_null() {
        if isvisible(client) {
            n += 1;
        }
        client = (*client).next;
    }
    if n > 0 {
        let symbol = format!("[{}]", n);
        util::copy_bytes_to_cstr(&mut (*monitor).ltsymbol, symbol.as_bytes());
    }
    client = nexttiled((*monitor).clients);
    while !client.is_null() {
        resize(
            client,
            (*monitor).wx,
            (*monitor).wy,
            (*monitor).ww - 2 * (*client).bw,
            (*monitor).wh - 2 * (*client).bw,
            0,
        );
        client = nexttiled((*client).next);
    }
}

unsafe fn motionnotify(event: *mut xlib::XEvent) {
    let st = state();
    let ev = &(*event).motion;
    if ev.window != st.root {
        return;
    }
    let monitor = recttomon(ev.x_root, ev.y_root, 1, 1);
    if monitor != st.motion_mon && !st.motion_mon.is_null() {
        unfocus((*st.selmon).sel, 1);
        st.selmon = monitor;
        focus(ptr::null_mut());
    }
    st.motion_mon = monitor;
}

pub unsafe fn movemouse(_arg: *const Arg) {
    let st = state();
    let client = (*st.selmon).sel;
    if client.is_null() || (*client).isfullscreen != 0 {
        return;
    }
    restack(st.selmon);
    let ocx = (*client).x;
    let ocy = (*client).y;
    if xlib::XGrabPointer(
        st.dpy,
        st.root,
        xlib::False,
        (xlib::ButtonPressMask | xlib::ButtonReleaseMask | xlib::PointerMotionMask) as c_uint,
        xlib::GrabModeAsync,
        xlib::GrabModeAsync,
        0,
        (*st.cursor[CUR_MOVE]).cursor,
        xlib::CurrentTime,
    ) != xlib::GrabSuccess
    {
        return;
    }

    let mut x = 0;
    let mut y = 0;
    if getrootptr(&mut x, &mut y) == 0 {
        return;
    }
    let mut ev: xlib::XEvent = mem::zeroed();
    let mut lasttime: xlib::Time = 0;
    loop {
        xlib::XMaskEvent(
            st.dpy,
            xlib::ButtonPressMask
                | xlib::ButtonReleaseMask
                | xlib::PointerMotionMask
                | xlib::ExposureMask
                | xlib::SubstructureRedirectMask,
            &mut ev,
        );
        match ev.get_type() {
            xlib::ConfigureRequest | xlib::Expose | xlib::MapRequest => handle_event(&mut ev),
            xlib::MotionNotify => {
                let motion = ev.motion;
                if motion.time.saturating_sub(lasttime) <= (1000 / 60) as c_ulong {
                    continue;
                }
                lasttime = motion.time;

                let mut nx = ocx + (motion.x - x);
                let mut ny = ocy + (motion.y - y);
                if ((*st.selmon).wx - nx).abs() < config::snap() as c_int {
                    nx = (*st.selmon).wx;
                } else if (((*st.selmon).wx + (*st.selmon).ww) - (nx + width(client))).abs()
                    < config::snap() as c_int
                {
                    nx = (*st.selmon).wx + (*st.selmon).ww - width(client);
                }
                if ((*st.selmon).wy - ny).abs() < config::snap() as c_int {
                    ny = (*st.selmon).wy;
                } else if (((*st.selmon).wy + (*st.selmon).wh) - (ny + height(client))).abs()
                    < config::snap() as c_int
                {
                    ny = (*st.selmon).wy + (*st.selmon).wh - height(client);
                }
                if (*client).isfloating == 0
                    && layout_has_arrange(st.selmon)
                    && ((nx - (*client).x).abs() > config::snap() as c_int
                        || (ny - (*client).y).abs() > config::snap() as c_int)
                {
                    togglefloating(ptr::null());
                }
                if !layout_has_arrange(st.selmon) || (*client).isfloating != 0 {
                    resize(client, nx, ny, (*client).w, (*client).h, 1);
                }
            }
            xlib::ButtonRelease => break,
            _ => {}
        }
        if ev.get_type() == xlib::ButtonRelease {
            break;
        }
    }
    xlib::XUngrabPointer(st.dpy, xlib::CurrentTime);
    let monitor = recttomon((*client).x, (*client).y, (*client).w, (*client).h);
    if monitor != st.selmon {
        sendmon(client, monitor);
        st.selmon = monitor;
        focus(ptr::null_mut());
    }
}

unsafe fn nexttiled(mut client: *mut Client) -> *mut Client {
    while !client.is_null() && ((*client).isfloating != 0 || !isvisible(client)) {
        client = (*client).next;
    }
    client
}

unsafe fn pop(client: *mut Client) {
    detach(client);
    attach(client);
    focus(client);
    arrange((*client).mon);
}

unsafe fn propertynotify(event: *mut xlib::XEvent) {
    let st = state();
    let ev = &(*event).property;
    if ev.window == st.root && ev.atom == xlib::XA_WM_NAME {
        updatestatus();
    } else if ev.state == xlib::PropertyDelete {
        return;
    } else {
        let client = wintoclient(ev.window);
        if client.is_null() {
            return;
        }
        match ev.atom {
            atom if atom == xlib::XA_WM_TRANSIENT_FOR => {
                let mut trans = 0;
                if (*client).isfloating == 0
                    && xlib::XGetTransientForHint(st.dpy, (*client).win, &mut trans) != 0
                    && {
                        (*client).isfloating = (!wintoclient(trans).is_null()) as c_int;
                        (*client).isfloating != 0
                    }
                {
                    arrange((*client).mon);
                }
            }
            atom if atom == xlib::XA_WM_NORMAL_HINTS => updatesizehints(client),
            atom if atom == xlib::XA_WM_HINTS => {
                updatewmhints(client);
                drawbars();
            }
            _ => {}
        }
        if ev.atom == xlib::XA_WM_NAME || ev.atom == st.netatom[NET_WM_NAME] {
            updatetitle(client);
            if client == (*(*client).mon).sel {
                drawbar((*client).mon);
            }
        }
        if ev.atom == st.netatom[NET_WM_WINDOW_TYPE] {
            updatewindowtype(client);
        }
    }
}

pub unsafe fn quit(_arg: *const Arg) {
    state().running = false;
}

unsafe fn recttomon(x: c_int, y: c_int, w: c_int, h: c_int) -> *mut Monitor {
    let st = state();
    let mut result = st.selmon;
    let mut area = 0;
    let mut monitor = st.mons;
    while !monitor.is_null() {
        let current = intersect(x, y, w, h, monitor);
        if current > area {
            area = current;
            result = monitor;
        }
        monitor = (*monitor).next;
    }
    result
}

unsafe fn resize(
    client: *mut Client,
    mut x: c_int,
    mut y: c_int,
    mut w: c_int,
    mut h: c_int,
    interact: c_int,
) {
    if applysizehints(client, &mut x, &mut y, &mut w, &mut h, interact) != 0 {
        resizeclient(client, x, y, w, h);
    }
}

unsafe fn resizeclient(client: *mut Client, x: c_int, y: c_int, w: c_int, h: c_int) {
    let st = state();
    let mut wc: xlib::XWindowChanges = mem::zeroed();
    (*client).oldx = (*client).x;
    (*client).x = x;
    wc.x = x;
    (*client).oldy = (*client).y;
    (*client).y = y;
    wc.y = y;
    (*client).oldw = (*client).w;
    (*client).w = w;
    wc.width = w;
    (*client).oldh = (*client).h;
    (*client).h = h;
    wc.height = h;
    wc.border_width = (*client).bw;

    if ((nexttiled((*(*client).mon).clients) == client && nexttiled((*client).next).is_null())
        || is_monocle_layout((*client).mon))
        && (*client).isfullscreen == 0
        && (*client).isfloating == 0
    {
        (*client).w = wc.width + (*client).bw * 2;
        (*client).h = wc.height + (*client).bw * 2;
        wc.width += (*client).bw * 2;
        wc.height += (*client).bw * 2;
        wc.border_width = 0;
    }

    xlib::XConfigureWindow(
        st.dpy,
        (*client).win,
        (xlib::CWX | xlib::CWY | xlib::CWWidth | xlib::CWHeight | xlib::CWBorderWidth) as c_uint,
        &mut wc,
    );
    configure(client);
    xlib::XSync(st.dpy, xlib::False);
}

pub unsafe fn resizemouse(_arg: *const Arg) {
    let st = state();
    let client = (*st.selmon).sel;
    if client.is_null() || (*client).isfullscreen != 0 {
        return;
    }
    restack(st.selmon);
    let ocx = (*client).x;
    let ocy = (*client).y;
    if xlib::XGrabPointer(
        st.dpy,
        st.root,
        xlib::False,
        (xlib::ButtonPressMask | xlib::ButtonReleaseMask | xlib::PointerMotionMask) as c_uint,
        xlib::GrabModeAsync,
        xlib::GrabModeAsync,
        0,
        (*st.cursor[CUR_RESIZE]).cursor,
        xlib::CurrentTime,
    ) != xlib::GrabSuccess
    {
        return;
    }
    xlib::XWarpPointer(
        st.dpy,
        0,
        (*client).win,
        0,
        0,
        0,
        0,
        (*client).w + (*client).bw - 1,
        (*client).h + (*client).bw - 1,
    );

    let mut ev: xlib::XEvent = mem::zeroed();
    let mut lasttime: c_ulong = 0;
    loop {
        xlib::XMaskEvent(
            st.dpy,
            xlib::ButtonPressMask
                | xlib::ButtonReleaseMask
                | xlib::PointerMotionMask
                | xlib::ExposureMask
                | xlib::SubstructureRedirectMask,
            &mut ev,
        );
        match ev.get_type() {
            xlib::ConfigureRequest | xlib::Expose | xlib::MapRequest => handle_event(&mut ev),
            xlib::MotionNotify => {
                let motion = ev.motion;
                if motion.time.saturating_sub(lasttime) <= (1000 / 60) as c_ulong {
                    continue;
                }
                lasttime = motion.time;
                let nw = util::max_i32(motion.x - ocx - 2 * (*client).bw + 1, 1);
                let nh = util::max_i32(motion.y - ocy - 2 * (*client).bw + 1, 1);
                if (*(*client).mon).wx + nw >= (*st.selmon).wx
                    && (*(*client).mon).wx + nw <= (*st.selmon).wx + (*st.selmon).ww
                    && (*(*client).mon).wy + nh >= (*st.selmon).wy
                    && (*(*client).mon).wy + nh <= (*st.selmon).wy + (*st.selmon).wh
                    && (*client).isfloating == 0
                    && layout_has_arrange(st.selmon)
                    && ((nw - (*client).w).abs() > config::snap() as c_int
                        || (nh - (*client).h).abs() > config::snap() as c_int)
                {
                    togglefloating(ptr::null());
                }
                if !layout_has_arrange(st.selmon) || (*client).isfloating != 0 {
                    resize(client, (*client).x, (*client).y, nw, nh, 1);
                }
            }
            xlib::ButtonRelease => break,
            _ => {}
        }
        if ev.get_type() == xlib::ButtonRelease {
            break;
        }
    }
    xlib::XWarpPointer(
        st.dpy,
        0,
        (*client).win,
        0,
        0,
        0,
        0,
        (*client).w + (*client).bw - 1,
        (*client).h + (*client).bw - 1,
    );
    xlib::XUngrabPointer(st.dpy, xlib::CurrentTime);
    while xlib::XCheckMaskEvent(st.dpy, xlib::EnterWindowMask, &mut ev) != 0 {}
    let monitor = recttomon((*client).x, (*client).y, (*client).w, (*client).h);
    if monitor != st.selmon {
        sendmon(client, monitor);
        st.selmon = monitor;
        focus(ptr::null_mut());
    }
}

unsafe fn restack(monitor: *mut Monitor) {
    let st = state();
    let mut ev: xlib::XEvent = mem::zeroed();
    let mut wc: xlib::XWindowChanges = mem::zeroed();

    drawbar(monitor);
    if (*monitor).sel.is_null() {
        return;
    }
    if (*(*monitor).sel).isfloating != 0 || !layout_has_arrange(monitor) {
        xlib::XRaiseWindow(st.dpy, (*(*monitor).sel).win);
    }
    if layout_has_arrange(monitor) {
        wc.stack_mode = xlib::Below;
        wc.sibling = (*monitor).barwin;
        let mut client = (*monitor).stack;
        while !client.is_null() {
            if (*client).isfloating == 0 && isvisible(client) {
                xlib::XConfigureWindow(
                    st.dpy,
                    (*client).win,
                    (xlib::CWSibling | xlib::CWStackMode) as c_uint,
                    &mut wc,
                );
                wc.sibling = (*client).win;
            }
            client = (*client).snext;
        }
    }
    xlib::XSync(st.dpy, xlib::False);
    while xlib::XCheckMaskEvent(st.dpy, xlib::EnterWindowMask, &mut ev) != 0 {}
}

unsafe fn run() {
    let st = state();
    let mut event: xlib::XEvent = mem::zeroed();
    xlib::XSync(st.dpy, xlib::False);
    while st.running && xlib::XNextEvent(st.dpy, &mut event) == 0 {
        handle_event(&mut event);
    }
}

unsafe fn scan() {
    let st = state();
    let mut wins: *mut xlib::Window = ptr::null_mut();
    let mut num = 0;
    let mut d1 = 0;
    let mut d2 = 0;
    let mut wa: xlib::XWindowAttributes = mem::zeroed();

    if xlib::XQueryTree(st.dpy, st.root, &mut d1, &mut d2, &mut wins, &mut num) != 0 {
        for index in 0..num {
            let window = *wins.add(index as usize);
            if xlib::XGetWindowAttributes(st.dpy, window, &mut wa) == 0
                || wa.override_redirect != 0
                || xlib::XGetTransientForHint(st.dpy, window, &mut d1) != 0
            {
                continue;
            }
            if wa.map_state == xlib::IsViewable || getstate(window) == ffi::ICONIC_STATE {
                manage(window, &mut wa);
            }
        }

        for index in 0..num {
            let window = *wins.add(index as usize);
            if xlib::XGetWindowAttributes(st.dpy, window, &mut wa) == 0 {
                continue;
            }
            if xlib::XGetTransientForHint(st.dpy, window, &mut d1) != 0
                && (wa.map_state == xlib::IsViewable || getstate(window) == ffi::ICONIC_STATE)
            {
                manage(window, &mut wa);
            }
        }
        if !wins.is_null() {
            xlib::XFree(wins.cast());
        }
    }
}

unsafe fn sendmon(client: *mut Client, monitor: *mut Monitor) {
    if (*client).mon == monitor {
        return;
    }
    unfocus(client, 1);
    detach(client);
    detachstack(client);
    (*client).mon = monitor;
    (*client).tags = (*monitor).tagset[(*monitor).seltags as usize];
    attach(client);
    attachstack(client);
    focus(ptr::null_mut());
    arrange(ptr::null_mut());
}

unsafe fn setclientstate(client: *mut Client, state_value: c_long) {
    let st = state();
    let data = [state_value, 0];
    xlib::XChangeProperty(
        st.dpy,
        (*client).win,
        st.wmatom[WM_STATE],
        st.wmatom[WM_STATE],
        32,
        xlib::PropModeReplace,
        data.as_ptr().cast(),
        2,
    );
}

unsafe fn sendevent(client: *mut Client, proto: xlib::Atom) -> c_int {
    let st = state();
    let mut n = 0;
    let mut protocols: *mut xlib::Atom = ptr::null_mut();
    let mut exists = 0;
    if xlib::XGetWMProtocols(st.dpy, (*client).win, &mut protocols, &mut n) != 0 {
        let slice = std::slice::from_raw_parts(protocols, n as usize);
        exists = slice.iter().any(|atom| *atom == proto) as c_int;
        xlib::XFree(protocols.cast());
    }

    if exists != 0 {
        let mut event: xlib::XEvent = mem::zeroed();
        event.client_message.type_ = xlib::ClientMessage;
        event.client_message.window = (*client).win;
        event.client_message.message_type = st.wmatom[WM_PROTOCOLS];
        event.client_message.format = 32;
        event.client_message.data = xlib::ClientMessageData::new();
        event.client_message.data.as_longs_mut()[0] = proto as c_long;
        event.client_message.data.as_longs_mut()[1] = xlib::CurrentTime as c_long;
        xlib::XSendEvent(
            st.dpy,
            (*client).win,
            xlib::False,
            xlib::NoEventMask,
            &mut event,
        );
    }
    exists
}

unsafe fn setfocus(client: *mut Client) {
    let st = state();
    if (*client).neverfocus == 0 {
        xlib::XSetInputFocus(
            st.dpy,
            (*client).win,
            xlib::RevertToPointerRoot,
            xlib::CurrentTime,
        );
        xlib::XChangeProperty(
            st.dpy,
            st.root,
            st.netatom[NET_ACTIVE_WINDOW],
            xlib::XA_WINDOW,
            32,
            xlib::PropModeReplace,
            (&(*client).win as *const xlib::Window).cast(),
            1,
        );
    }
    sendevent(client, st.wmatom[WM_TAKE_FOCUS]);
}

unsafe fn setfullscreen(client: *mut Client, fullscreen: c_int) {
    let st = state();
    if fullscreen != 0 && (*client).isfullscreen == 0 {
        xlib::XChangeProperty(
            st.dpy,
            (*client).win,
            st.netatom[NET_WM_STATE],
            xlib::XA_ATOM,
            32,
            xlib::PropModeReplace,
            (&st.netatom[NET_WM_FULLSCREEN] as *const xlib::Atom).cast(),
            1,
        );
        (*client).isfullscreen = 1;
        (*client).oldstate = (*client).isfloating;
        (*client).oldbw = (*client).bw;
        (*client).bw = 0;
        (*client).isfloating = 1;
        resizeclient(
            client,
            (*(*client).mon).mx,
            (*(*client).mon).my,
            (*(*client).mon).mw,
            (*(*client).mon).mh,
        );
        xlib::XRaiseWindow(st.dpy, (*client).win);
    } else if fullscreen == 0 && (*client).isfullscreen != 0 {
        xlib::XChangeProperty(
            st.dpy,
            (*client).win,
            st.netatom[NET_WM_STATE],
            xlib::XA_ATOM,
            32,
            xlib::PropModeReplace,
            ptr::null(),
            0,
        );
        (*client).isfullscreen = 0;
        (*client).isfloating = (*client).oldstate;
        (*client).bw = (*client).oldbw;
        (*client).x = (*client).oldx;
        (*client).y = (*client).oldy;
        (*client).w = (*client).oldw;
        (*client).h = (*client).oldh;
        resizeclient(client, (*client).x, (*client).y, (*client).w, (*client).h);
        arrange((*client).mon);
    }
}

pub unsafe fn setlayout(arg: *const Arg) {
    let st = state();
    if arg.is_null() || (*arg).v.is_null() || (*arg).v != selected_layout(st.selmon).cast() {
        (*st.selmon).sellt ^= 1;
    }
    if !arg.is_null() && !(*arg).v.is_null() {
        (*st.selmon).lt[(*st.selmon).sellt as usize] = (*arg).v.cast();
    }
    util::copy_cstr(
        &mut (*st.selmon).ltsymbol,
        (*selected_layout(st.selmon)).symbol,
    );
    if !(*st.selmon).sel.is_null() {
        arrange(st.selmon);
    } else {
        drawbar(st.selmon);
    }
}

pub unsafe fn setmfact(arg: *const Arg) {
    let st = state();
    if arg.is_null() || !layout_has_arrange(st.selmon) {
        return;
    }
    let f = if (*arg).f < 1.0 {
        (*arg).f + (*st.selmon).mfact
    } else {
        (*arg).f - 1.0
    };
    if !(0.1..=0.9).contains(&f) {
        return;
    }
    (*st.selmon).mfact = f;
    arrange(st.selmon);
}

unsafe fn setup() {
    let st = state();
    sigchld(0);

    st.screen = xlib::XDefaultScreen(st.dpy);
    st.sw = xlib::XDisplayWidth(st.dpy, st.screen);
    st.sh = xlib::XDisplayHeight(st.dpy, st.screen);
    st.root = xlib::XRootWindow(st.dpy, st.screen);
    st.drw = drw::drw_create(st.dpy, st.screen, st.root, st.sw as c_uint, st.sh as c_uint);
    if drw::drw_fontset_create(st.drw, config::fonts()).is_null() {
        util::die("no fonts could be loaded.");
    }
    st.lrpad = (*(*st.drw).fonts).h as c_int;
    st.bh = (*(*st.drw).fonts).h as c_int + 2;
    updategeom();

    let utf8string = xlib::XInternAtom(st.dpy, b"UTF8_STRING\0".as_ptr().cast(), xlib::False);
    st.wmatom[WM_PROTOCOLS] =
        xlib::XInternAtom(st.dpy, b"WM_PROTOCOLS\0".as_ptr().cast(), xlib::False);
    st.wmatom[WM_DELETE] =
        xlib::XInternAtom(st.dpy, b"WM_DELETE_WINDOW\0".as_ptr().cast(), xlib::False);
    st.wmatom[WM_STATE] = xlib::XInternAtom(st.dpy, b"WM_STATE\0".as_ptr().cast(), xlib::False);
    st.wmatom[WM_TAKE_FOCUS] =
        xlib::XInternAtom(st.dpy, b"WM_TAKE_FOCUS\0".as_ptr().cast(), xlib::False);
    st.netatom[NET_ACTIVE_WINDOW] =
        xlib::XInternAtom(st.dpy, b"_NET_ACTIVE_WINDOW\0".as_ptr().cast(), xlib::False);
    st.netatom[NET_SUPPORTED] =
        xlib::XInternAtom(st.dpy, b"_NET_SUPPORTED\0".as_ptr().cast(), xlib::False);
    st.netatom[NET_WM_NAME] =
        xlib::XInternAtom(st.dpy, b"_NET_WM_NAME\0".as_ptr().cast(), xlib::False);
    st.netatom[NET_WM_STATE] =
        xlib::XInternAtom(st.dpy, b"_NET_WM_STATE\0".as_ptr().cast(), xlib::False);
    st.netatom[NET_WM_CHECK] = xlib::XInternAtom(
        st.dpy,
        b"_NET_SUPPORTING_WM_CHECK\0".as_ptr().cast(),
        xlib::False,
    );
    st.netatom[NET_WM_FULLSCREEN] = xlib::XInternAtom(
        st.dpy,
        b"_NET_WM_STATE_FULLSCREEN\0".as_ptr().cast(),
        xlib::False,
    );
    st.netatom[NET_WM_WINDOW_TYPE] = xlib::XInternAtom(
        st.dpy,
        b"_NET_WM_WINDOW_TYPE\0".as_ptr().cast(),
        xlib::False,
    );
    st.netatom[NET_WM_WINDOW_TYPE_DIALOG] = xlib::XInternAtom(
        st.dpy,
        b"_NET_WM_WINDOW_TYPE_DIALOG\0".as_ptr().cast(),
        xlib::False,
    );
    st.netatom[NET_CLIENT_LIST] =
        xlib::XInternAtom(st.dpy, b"_NET_CLIENT_LIST\0".as_ptr().cast(), xlib::False);

    st.cursor[CUR_NORMAL] = drw::drw_cur_create(st.drw, ffi::XC_LEFT_PTR);
    st.cursor[CUR_RESIZE] = drw::drw_cur_create(st.drw, ffi::XC_SIZING);
    st.cursor[CUR_MOVE] = drw::drw_cur_create(st.drw, ffi::XC_FLEUR);

    st.scheme = config::colors()
        .iter()
        .map(|colors| drw::drw_scm_create(st.drw, colors.as_ref()))
        .collect();

    updatebars();
    updatestatus();

    st.wmcheckwin = xlib::XCreateSimpleWindow(st.dpy, st.root, 0, 0, 1, 1, 0, 0, 0);
    xlib::XChangeProperty(
        st.dpy,
        st.wmcheckwin,
        st.netatom[NET_WM_CHECK],
        xlib::XA_WINDOW,
        32,
        xlib::PropModeReplace,
        (&st.wmcheckwin as *const xlib::Window).cast(),
        1,
    );
    xlib::XChangeProperty(
        st.dpy,
        st.wmcheckwin,
        st.netatom[NET_WM_NAME],
        utf8string,
        8,
        xlib::PropModeReplace,
        b"dwm\0".as_ptr(),
        3,
    );
    xlib::XChangeProperty(
        st.dpy,
        st.root,
        st.netatom[NET_WM_CHECK],
        xlib::XA_WINDOW,
        32,
        xlib::PropModeReplace,
        (&st.wmcheckwin as *const xlib::Window).cast(),
        1,
    );
    xlib::XChangeProperty(
        st.dpy,
        st.root,
        st.netatom[NET_SUPPORTED],
        xlib::XA_ATOM,
        32,
        xlib::PropModeReplace,
        st.netatom.as_ptr().cast(),
        NET_LAST as c_int,
    );
    xlib::XDeleteProperty(st.dpy, st.root, st.netatom[NET_CLIENT_LIST]);

    let mut wa: xlib::XSetWindowAttributes = mem::zeroed();
    wa.cursor = (*st.cursor[CUR_NORMAL]).cursor;
    wa.event_mask = xlib::SubstructureRedirectMask
        | xlib::SubstructureNotifyMask
        | xlib::ButtonPressMask
        | xlib::PointerMotionMask
        | xlib::EnterWindowMask
        | xlib::LeaveWindowMask
        | xlib::StructureNotifyMask
        | xlib::PropertyChangeMask;
    xlib::XChangeWindowAttributes(st.dpy, st.root, xlib::CWEventMask | xlib::CWCursor, &mut wa);
    xlib::XSelectInput(st.dpy, st.root, wa.event_mask);
    grabkeys();
    focus(ptr::null_mut());
}

unsafe fn seturgent(client: *mut Client, urg: c_int) {
    let st = state();
    (*client).isurgent = urg;
    let wmh = xlib::XGetWMHints(st.dpy, (*client).win);
    if wmh.is_null() {
        return;
    }
    if urg != 0 {
        (*wmh).flags |= xlib::XUrgencyHint;
    } else {
        (*wmh).flags &= !xlib::XUrgencyHint;
    }
    xlib::XSetWMHints(st.dpy, (*client).win, wmh);
    xlib::XFree(wmh.cast());
}

unsafe fn showhide(client: *mut Client) {
    let st = state();
    if client.is_null() {
        return;
    }
    if isvisible(client) {
        xlib::XMoveWindow(st.dpy, (*client).win, (*client).x, (*client).y);
        if (!layout_has_arrange((*client).mon) || (*client).isfloating != 0)
            && (*client).isfullscreen == 0
        {
            resize(
                client,
                (*client).x,
                (*client).y,
                (*client).w,
                (*client).h,
                0,
            );
        }
        showhide((*client).snext);
    } else {
        showhide((*client).snext);
        xlib::XMoveWindow(st.dpy, (*client).win, -2 * width(client), (*client).y);
    }
}

unsafe extern "C" fn sigchld(_unused: c_int) {
    if libc::signal(libc::SIGCHLD, sigchld as usize) == libc::SIG_ERR {
        util::die_perror("can't install SIGCHLD handler:");
    }
    while libc::waitpid(-1, ptr::null_mut(), libc::WNOHANG) > 0 {}
}

pub unsafe fn spawn(arg: *const Arg) {
    let st = state();
    if libc::fork() == 0 {
        if !st.dpy.is_null() {
            libc::close(xlib::XConnectionNumber(st.dpy));
        }
        libc::setsid();
        let argv = (*arg).v as *const *const c_char;
        libc::execvp(*argv, argv.cast());
        eprintln!(
            "dwm: execvp {} failed: {}",
            CStr::from_ptr(*argv).to_string_lossy(),
            std::io::Error::last_os_error()
        );
        libc::_exit(libc::EXIT_SUCCESS);
    }
}

pub unsafe fn tag(arg: *const Arg) {
    let st = state();
    if !(*st.selmon).sel.is_null() && (*arg).ui & tagmask() != 0 {
        (*(*st.selmon).sel).tags = (*arg).ui & tagmask();
        focus(ptr::null_mut());
        arrange(st.selmon);
    }
}

pub unsafe fn tagmon(arg: *const Arg) {
    let st = state();
    if (*st.selmon).sel.is_null() || (*st.mons).next.is_null() {
        return;
    }
    sendmon((*st.selmon).sel, dirtomon((*arg).i));
}

pub unsafe fn tile(monitor: *mut Monitor) {
    let mut n = 0;
    let mut client = nexttiled((*monitor).clients);
    while !client.is_null() {
        n += 1;
        client = nexttiled((*client).next);
    }
    if n == 0 {
        return;
    }

    let mut g = 0;
    let mw = if n > (*monitor).nmaster as c_uint {
        if (*monitor).nmaster != 0 {
            g = config::gappx() as c_int;
            (((*monitor).ww - g) as f32 * (*monitor).mfact) as c_int
        } else {
            0
        }
    } else {
        (*monitor).ww
    };

    let mut i = 0u32;
    let mut my = 0;
    let mut ty = 0;
    client = nexttiled((*monitor).clients);
    while !client.is_null() {
        if i < (*monitor).nmaster as u32 {
            let r = std::cmp::min(n, (*monitor).nmaster as u32) - i;
            let h = ((*monitor).wh - my - config::gappx() as c_int * (r as c_int - 1)) / r as c_int;
            resize(
                client,
                (*monitor).wx,
                (*monitor).wy + my,
                mw - 2 * (*client).bw,
                h - 2 * (*client).bw,
                0,
            );
            my += height(client) + config::gappx() as c_int;
        } else {
            let r = n - i;
            let h = ((*monitor).wh - ty - config::gappx() as c_int * (r as c_int - 1)) / r as c_int;
            resize(
                client,
                (*monitor).wx + mw + g,
                (*monitor).wy + ty,
                (*monitor).ww - mw - g - 2 * (*client).bw,
                h - 2 * (*client).bw,
                0,
            );
            ty += height(client) + config::gappx() as c_int;
        }
        i += 1;
        client = nexttiled((*client).next);
    }
}

pub unsafe fn togglebar(_arg: *const Arg) {
    let st = state();
    (*st.selmon).showbar = if (*st.selmon).showbar == 0 { 1 } else { 0 };
    updatebarpos(st.selmon);
    xlib::XMoveResizeWindow(
        st.dpy,
        (*st.selmon).barwin,
        (*st.selmon).wx,
        (*st.selmon).by,
        (*st.selmon).ww as c_uint,
        st.bh as c_uint,
    );
    arrange(st.selmon);
}

pub unsafe fn togglefloating(_arg: *const Arg) {
    let st = state();
    if (*st.selmon).sel.is_null() || (*(*st.selmon).sel).isfullscreen != 0 {
        return;
    }
    (*(*st.selmon).sel).isfloating =
        if (*(*st.selmon).sel).isfloating == 0 || (*(*st.selmon).sel).isfixed != 0 {
            1
        } else {
            0
        };
    if (*(*st.selmon).sel).isfloating != 0 {
        resize(
            (*st.selmon).sel,
            (*(*st.selmon).sel).x,
            (*(*st.selmon).sel).y,
            (*(*st.selmon).sel).w,
            (*(*st.selmon).sel).h,
            0,
        );
    }
    arrange(st.selmon);
}

pub unsafe fn toggletag(arg: *const Arg) {
    let st = state();
    if (*st.selmon).sel.is_null() {
        return;
    }
    let newtags = (*(*st.selmon).sel).tags ^ ((*arg).ui & tagmask());
    if newtags != 0 {
        (*(*st.selmon).sel).tags = newtags;
        focus(ptr::null_mut());
        arrange(st.selmon);
    }
}

pub unsafe fn toggleview(arg: *const Arg) {
    let st = state();
    let newtagset = (*st.selmon).tagset[(*st.selmon).seltags as usize] ^ ((*arg).ui & tagmask());
    if newtagset != 0 {
        (*st.selmon).tagset[(*st.selmon).seltags as usize] = newtagset;
        focus(ptr::null_mut());
        arrange(st.selmon);
    }
}

unsafe fn unfocus(client: *mut Client, setfocus: c_int) {
    let st = state();
    if client.is_null() {
        return;
    }
    grabbuttons(client, 0);
    xlib::XSetWindowBorder(
        st.dpy,
        (*client).win,
        (*st.scheme[SCHEME_NORM].add(COL_BORDER)).pixel,
    );
    if setfocus != 0 {
        xlib::XSetInputFocus(
            st.dpy,
            st.root,
            xlib::RevertToPointerRoot,
            xlib::CurrentTime,
        );
        xlib::XDeleteProperty(st.dpy, st.root, st.netatom[NET_ACTIVE_WINDOW]);
    }
}

unsafe fn unmanage(client: *mut Client, destroyed: c_int) {
    let st = state();
    let monitor = (*client).mon;
    let mut wc: xlib::XWindowChanges = mem::zeroed();

    if !(*client).swallowing.is_null() {
        unswallow(client);
        return;
    }

    let swallowing = swallowingclient((*client).win);
    if !swallowing.is_null() {
        drop(Box::from_raw((*swallowing).swallowing));
        (*swallowing).swallowing = ptr::null_mut();
        arrange(monitor);
        focus(ptr::null_mut());
        return;
    }

    detach(client);
    detachstack(client);
    if destroyed == 0 {
        wc.border_width = (*client).oldbw;
        xlib::XGrabServer(st.dpy);
        xlib::XSetErrorHandler(Some(xerrordummy));
        xlib::XConfigureWindow(
            st.dpy,
            (*client).win,
            xlib::CWBorderWidth as c_uint,
            &mut wc,
        );
        xlib::XUngrabButton(
            st.dpy,
            xlib::AnyButton as c_uint,
            xlib::AnyModifier,
            (*client).win,
        );
        setclientstate(client, ffi::WITHDRAWN_STATE);
        xlib::XSync(st.dpy, xlib::False);
        xlib::XSetErrorHandler(Some(xerror));
        xlib::XUngrabServer(st.dpy);
    }
    drop(Box::from_raw(client));
    arrange(monitor);
    focus(ptr::null_mut());
    updateclientlist();
}

unsafe fn unmapnotify(event: *mut xlib::XEvent) {
    let ev = &(*event).unmap;
    let client = wintoclient(ev.window);
    if !client.is_null() {
        if ev.send_event != 0 {
            setclientstate(client, ffi::WITHDRAWN_STATE);
        } else {
            unmanage(client, 0);
        }
    }
}

unsafe fn updatebars() {
    let st = state();
    let mut wa: xlib::XSetWindowAttributes = mem::zeroed();
    wa.override_redirect = xlib::True;
    wa.background_pixmap = xlib::ParentRelative as c_ulong;
    wa.event_mask = xlib::ButtonPressMask | xlib::ExposureMask;
    let mut ch = xlib::XClassHint {
        res_name: b"dwm\0".as_ptr().cast_mut().cast(),
        res_class: b"dwm\0".as_ptr().cast_mut().cast(),
    };
    let mut monitor = st.mons;
    while !monitor.is_null() {
        if (*monitor).barwin == 0 {
            (*monitor).barwin = xlib::XCreateWindow(
                st.dpy,
                st.root,
                (*monitor).wx,
                (*monitor).by,
                (*monitor).ww as c_uint,
                st.bh as c_uint,
                0,
                xlib::XDefaultDepth(st.dpy, st.screen) as c_int,
                xlib::CopyFromParent as c_uint,
                xlib::XDefaultVisual(st.dpy, st.screen),
                (xlib::CWOverrideRedirect | xlib::CWBackPixmap | xlib::CWEventMask) as c_ulong,
                &mut wa,
            );
            xlib::XDefineCursor(st.dpy, (*monitor).barwin, (*st.cursor[CUR_NORMAL]).cursor);
            xlib::XMapRaised(st.dpy, (*monitor).barwin);
            xlib::XSetClassHint(st.dpy, (*monitor).barwin, &mut ch);
        }
        monitor = (*monitor).next;
    }
}

unsafe fn updatebarpos(monitor: *mut Monitor) {
    let st = state();
    (*monitor).wy = (*monitor).my;
    (*monitor).wh = (*monitor).mh;
    if (*monitor).showbar != 0 {
        (*monitor).wh -= st.bh;
        (*monitor).by = if (*monitor).topbar != 0 {
            (*monitor).wy
        } else {
            (*monitor).wy + (*monitor).wh
        };
        if (*monitor).topbar != 0 {
            (*monitor).wy += st.bh;
        }
    } else {
        (*monitor).by = -st.bh;
    }
}

unsafe fn updateclientlist() {
    let st = state();
    xlib::XDeleteProperty(st.dpy, st.root, st.netatom[NET_CLIENT_LIST]);
    let mut monitor = st.mons;
    while !monitor.is_null() {
        let mut client = (*monitor).clients;
        while !client.is_null() {
            xlib::XChangeProperty(
                st.dpy,
                st.root,
                st.netatom[NET_CLIENT_LIST],
                xlib::XA_WINDOW,
                32,
                xlib::PropModeAppend,
                (&(*client).win as *const xlib::Window).cast(),
                1,
            );
            client = (*client).next;
        }
        monitor = (*monitor).next;
    }
}

unsafe fn updategeom() -> c_int {
    let st = state();
    let mut dirty = 0;

    if xinerama::XineramaIsActive(st.dpy) != 0 {
        let mut nn = 0;
        let info = xinerama::XineramaQueryScreens(st.dpy, &mut nn);
        if !info.is_null() {
            let info_slice = std::slice::from_raw_parts(info, nn as usize);
            let mut unique = Vec::new();
            for item in info_slice.iter() {
                if isuniquegeom(&unique, item) {
                    unique.push(*item);
                }
            }
            xlib::XFree(info.cast());

            let mut n = 0;
            let mut monitor = st.mons;
            while !monitor.is_null() {
                n += 1;
                monitor = (*monitor).next;
            }

            if n <= unique.len() {
                for _ in 0..(unique.len() - n) {
                    let new_monitor = createmon();
                    if st.mons.is_null() {
                        st.mons = new_monitor;
                    } else {
                        let mut tail = st.mons;
                        while !(*tail).next.is_null() {
                            tail = (*tail).next;
                        }
                        (*tail).next = new_monitor;
                    }
                }

                monitor = st.mons;
                for (index, geom) in unique.iter().enumerate() {
                    if monitor.is_null() {
                        break;
                    }
                    if index >= n
                        || geom.x_org as c_int != (*monitor).mx
                        || geom.y_org as c_int != (*monitor).my
                        || geom.width as c_int != (*monitor).mw
                        || geom.height as c_int != (*monitor).mh
                    {
                        dirty = 1;
                        (*monitor).num = index as c_int;
                        (*monitor).mx = geom.x_org as c_int;
                        (*monitor).wx = geom.x_org as c_int;
                        (*monitor).my = geom.y_org as c_int;
                        (*monitor).wy = geom.y_org as c_int;
                        (*monitor).mw = geom.width as c_int;
                        (*monitor).ww = geom.width as c_int;
                        (*monitor).mh = geom.height as c_int;
                        (*monitor).wh = geom.height as c_int;
                        updatebarpos(monitor);
                    }
                    monitor = (*monitor).next;
                }
            } else {
                for _ in unique.len()..n {
                    let mut last = st.mons;
                    while !(*last).next.is_null() {
                        last = (*last).next;
                    }
                    while !(*last).clients.is_null() {
                        let client = (*last).clients;
                        dirty = 1;
                        (*last).clients = (*client).next;
                        detachstack(client);
                        (*client).mon = st.mons;
                        attach(client);
                        attachstack(client);
                    }
                    if last == st.selmon {
                        st.selmon = st.mons;
                    }
                    cleanupmon(last);
                }
            }
        }
    } else {
        if st.mons.is_null() {
            st.mons = createmon();
        }
        if (*st.mons).mw != st.sw || (*st.mons).mh != st.sh {
            dirty = 1;
            (*st.mons).mw = st.sw;
            (*st.mons).ww = st.sw;
            (*st.mons).mh = st.sh;
            (*st.mons).wh = st.sh;
            updatebarpos(st.mons);
        }
    }

    if dirty != 0 {
        st.selmon = st.mons;
        st.selmon = wintomon(st.root);
    }
    dirty
}

unsafe fn updatenumlockmask() {
    let st = state();
    st.numlockmask = 0;
    let modmap = xlib::XGetModifierMapping(st.dpy);
    for i in 0..8 {
        for j in 0..(*modmap).max_keypermod {
            if *(*modmap)
                .modifiermap
                .add((i * (*modmap).max_keypermod + j) as usize)
                == xlib::XKeysymToKeycode(st.dpy, x11::keysym::XK_Num_Lock as c_ulong)
            {
                st.numlockmask = 1 << i;
            }
        }
    }
    xlib::XFreeModifiermap(modmap);
}

unsafe fn updatesizehints(client: *mut Client) {
    let st = state();
    let mut msize = 0;
    let mut size: xlib::XSizeHints = mem::zeroed();
    if xlib::XGetWMNormalHints(st.dpy, (*client).win, &mut size, &mut msize) == 0 {
        size.flags = xlib::PSize;
    }
    if size.flags & xlib::PBaseSize != 0 {
        (*client).basew = size.base_width;
        (*client).baseh = size.base_height;
    } else if size.flags & xlib::PMinSize != 0 {
        (*client).basew = size.min_width;
        (*client).baseh = size.min_height;
    } else {
        (*client).basew = 0;
        (*client).baseh = 0;
    }
    if size.flags & xlib::PResizeInc != 0 {
        (*client).incw = size.width_inc;
        (*client).inch = size.height_inc;
    } else {
        (*client).incw = 0;
        (*client).inch = 0;
    }
    if size.flags & xlib::PMaxSize != 0 {
        (*client).maxw = size.max_width;
        (*client).maxh = size.max_height;
    } else {
        (*client).maxw = 0;
        (*client).maxh = 0;
    }
    if size.flags & xlib::PMinSize != 0 {
        (*client).minw = size.min_width;
        (*client).minh = size.min_height;
    } else if size.flags & xlib::PBaseSize != 0 {
        (*client).minw = size.base_width;
        (*client).minh = size.base_height;
    } else {
        (*client).minw = 0;
        (*client).minh = 0;
    }
    if size.flags & xlib::PAspect != 0 {
        (*client).mina = size.min_aspect.y as f32 / size.min_aspect.x as f32;
        (*client).maxa = size.max_aspect.x as f32 / size.max_aspect.y as f32;
    } else {
        (*client).maxa = 0.0;
        (*client).mina = 0.0;
    }
    (*client).isfixed = ((*client).maxw != 0
        && (*client).maxh != 0
        && (*client).maxw == (*client).minw
        && (*client).maxh == (*client).minh) as c_int;
}

unsafe fn updatestatus() {
    let st = state();
    if gettextprop(st.root, xlib::XA_WM_NAME, &mut st.stext) == 0 {
        let fallback = format!("dwm-{}", config::VERSION);
        util::copy_bytes_to_cstr(&mut st.stext, fallback.as_bytes());
    }
    drawbar(st.selmon);
}

unsafe fn updatetitle(client: *mut Client) {
    let st = state();
    if gettextprop((*client).win, st.netatom[NET_WM_NAME], &mut (*client).name) == 0 {
        gettextprop((*client).win, xlib::XA_WM_NAME, &mut (*client).name);
    }
    if (*client).name[0] == 0 {
        util::copy_cstr(&mut (*client).name, broken_ptr());
    }
}

unsafe fn updatewindowtype(client: *mut Client) {
    let st = state();
    let state_atom = getatomprop(client, st.netatom[NET_WM_STATE]);
    let wtype = getatomprop(client, st.netatom[NET_WM_WINDOW_TYPE]);
    if state_atom == st.netatom[NET_WM_FULLSCREEN] {
        setfullscreen(client, 1);
    }
    if wtype == st.netatom[NET_WM_WINDOW_TYPE_DIALOG] {
        (*client).isfloating = 1;
    }
}

unsafe fn updatewmhints(client: *mut Client) {
    let st = state();
    let wmh = xlib::XGetWMHints(st.dpy, (*client).win);
    if wmh.is_null() {
        return;
    }
    if client == (*st.selmon).sel && (*wmh).flags & xlib::XUrgencyHint != 0 {
        (*wmh).flags &= !xlib::XUrgencyHint;
        xlib::XSetWMHints(st.dpy, (*client).win, wmh);
    } else {
        (*client).isurgent = ((*wmh).flags & xlib::XUrgencyHint != 0) as c_int;
    }
    if (*wmh).flags & xlib::InputHint != 0 {
        (*client).neverfocus = ((*wmh).input == 0) as c_int;
    } else {
        (*client).neverfocus = 0;
    }
    xlib::XFree(wmh.cast());
}

pub unsafe fn view(arg: *const Arg) {
    let st = state();
    if ((*arg).ui & tagmask()) == (*st.selmon).tagset[(*st.selmon).seltags as usize] {
        return;
    }
    (*st.selmon).seltags ^= 1;
    if (*arg).ui & tagmask() != 0 {
        (*st.selmon).tagset[(*st.selmon).seltags as usize] = (*arg).ui & tagmask();
    }
    focus(ptr::null_mut());
    arrange(st.selmon);
}

pub unsafe fn view_adjacent(arg: *const Arg) {
    let st = state();
    let curtags = (*st.selmon).tagset[(*st.selmon).seltags as usize];
    let mut seltag = 0i32;
    for (index, _) in config::tags().iter().enumerate() {
        if curtags & (1 << index) != 0 {
            seltag = index as i32;
            break;
        }
    }
    seltag = (seltag + (*arg).i) % config::tags().len() as i32;
    if seltag < 0 {
        seltag += config::tags().len() as i32;
    }
    let arg = Arg { ui: 1 << seltag };
    view(&arg);
}

unsafe fn winpid(window: xlib::Window) -> pid_t {
    let st = state();
    let spec = ffi::xcb_res_client_id_spec_t {
        client: window as u32,
        mask: ffi::XCB_RES_CLIENT_ID_MASK_LOCAL_CLIENT_PID,
    };
    let mut error: *mut ffi::xcb_generic_error_t = ptr::null_mut();
    let cookie = ffi::xcb_res_query_client_ids(st.xcon, 1, &spec);
    let reply = ffi::xcb_res_query_client_ids_reply(st.xcon, cookie, &mut error);
    if reply.is_null() {
        return 0;
    }

    let mut result = 0;
    let mut iter = ffi::xcb_res_query_client_ids_ids_iterator(reply);
    while iter.rem != 0 {
        let value = &*iter.data;
        if value.spec.mask & ffi::XCB_RES_CLIENT_ID_MASK_LOCAL_CLIENT_PID != 0 {
            let pid = ffi::xcb_res_client_id_value_value(iter.data);
            result = *pid as pid_t;
            break;
        }
        ffi::xcb_res_client_id_value_next(&mut iter);
    }
    libc::free(reply.cast());
    if !error.is_null() {
        libc::free(error.cast());
    }
    if result == -1 {
        0
    } else {
        result
    }
}

unsafe fn getparentprocess(pid: pid_t) -> pid_t {
    #[cfg(target_os = "linux")]
    {
        let path = format!("/proc/{pid}/stat");
        if let Ok(content) = fs::read_to_string(path) {
            if let Some((_, tail)) = content.rsplit_once(") ") {
                let mut parts = tail.split_whitespace();
                let _state = parts.next();
                if let Some(ppid) = parts.next() {
                    return ppid.parse::<pid_t>().unwrap_or(0);
                }
            }
        }
        0
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = pid;
        0
    }
}

unsafe fn isdescprocess(parent: pid_t, mut child: pid_t) -> c_int {
    while parent != child && child != 0 {
        child = getparentprocess(child);
    }
    child as c_int
}

unsafe fn termforwin(window_client: *const Client) -> *mut Client {
    let st = state();
    if (*window_client).pid == 0 || (*window_client).isterminal != 0 {
        return ptr::null_mut();
    }
    let mut monitor = st.mons;
    while !monitor.is_null() {
        let mut client = (*monitor).clients;
        while !client.is_null() {
            if (*client).isterminal != 0
                && (*client).swallowing.is_null()
                && (*client).pid != 0
                && isdescprocess((*client).pid, (*window_client).pid) != 0
            {
                return client;
            }
            client = (*client).next;
        }
        monitor = (*monitor).next;
    }
    ptr::null_mut()
}

unsafe fn swallowingclient(window: xlib::Window) -> *mut Client {
    let st = state();
    let mut monitor = st.mons;
    while !monitor.is_null() {
        let mut client = (*monitor).clients;
        while !client.is_null() {
            if !(*client).swallowing.is_null() && (*(*client).swallowing).win == window {
                return client;
            }
            client = (*client).next;
        }
        monitor = (*monitor).next;
    }
    ptr::null_mut()
}

unsafe fn wintoclient(window: xlib::Window) -> *mut Client {
    let st = state();
    let mut monitor = st.mons;
    while !monitor.is_null() {
        let mut client = (*monitor).clients;
        while !client.is_null() {
            if (*client).win == window {
                return client;
            }
            client = (*client).next;
        }
        monitor = (*monitor).next;
    }
    ptr::null_mut()
}

unsafe fn wintomon(window: xlib::Window) -> *mut Monitor {
    let st = state();
    let mut x = 0;
    let mut y = 0;
    if window == st.root && getrootptr(&mut x, &mut y) != 0 {
        return recttomon(x, y, 1, 1);
    }
    let mut monitor = st.mons;
    while !monitor.is_null() {
        if window == (*monitor).barwin {
            return monitor;
        }
        monitor = (*monitor).next;
    }
    let client = wintoclient(window);
    if !client.is_null() {
        (*client).mon
    } else {
        st.selmon
    }
}

unsafe extern "C" fn xerror(display: *mut xlib::Display, ee: *mut xlib::XErrorEvent) -> c_int {
    if (*ee).error_code == xlib::BadWindow
        || ((*ee).request_code == X_SET_INPUT_FOCUS && (*ee).error_code == xlib::BadMatch)
        || ((*ee).request_code == X_POLY_TEXT8 && (*ee).error_code == xlib::BadDrawable)
        || ((*ee).request_code == X_POLY_FILL_RECTANGLE && (*ee).error_code == xlib::BadDrawable)
        || ((*ee).request_code == X_POLY_SEGMENT && (*ee).error_code == xlib::BadDrawable)
        || ((*ee).request_code == X_CONFIGURE_WINDOW && (*ee).error_code == xlib::BadMatch)
        || ((*ee).request_code == X_GRAB_BUTTON && (*ee).error_code == xlib::BadAccess)
        || ((*ee).request_code == X_GRAB_KEY && (*ee).error_code == xlib::BadAccess)
        || ((*ee).request_code == X_COPY_AREA && (*ee).error_code == xlib::BadDrawable)
    {
        return 0;
    }
    eprintln!(
        "dwm: fatal error: request code={}, error code={}",
        (*ee).request_code,
        (*ee).error_code
    );
    if let Some(handler) = state().xerrorxlib {
        handler(display, ee)
    } else {
        0
    }
}

unsafe extern "C" fn xerrordummy(
    _display: *mut xlib::Display,
    _ee: *mut xlib::XErrorEvent,
) -> c_int {
    0
}

unsafe extern "C" fn xerrorstart(
    _display: *mut xlib::Display,
    _ee: *mut xlib::XErrorEvent,
) -> c_int {
    util::die("dwm: another window manager is already running");
}

pub unsafe fn zoom(_arg: *const Arg) {
    let st = state();
    let mut client = (*st.selmon).sel;
    if !layout_has_arrange(st.selmon) || (!client.is_null() && (*client).isfloating != 0) {
        return;
    }
    if client == nexttiled((*st.selmon).clients) {
        if client.is_null() {
            return;
        }
        client = nexttiled((*client).next);
        if client.is_null() {
            return;
        }
    }
    pop(client);
}

pub unsafe fn movestack(arg: *const Arg) {
    let st = state();
    if (*st.selmon).sel.is_null() {
        return;
    }
    let mut client = ptr::null_mut();
    let mut p: *mut Client = ptr::null_mut();
    let mut pc: *mut Client = ptr::null_mut();
    if (*arg).i > 0 {
        client = (*(*st.selmon).sel).next;
        while !client.is_null() && (!isvisible(client) || (*client).isfloating != 0) {
            client = (*client).next;
        }
        if client.is_null() {
            client = (*st.selmon).clients;
            while !client.is_null() && (!isvisible(client) || (*client).isfloating != 0) {
                client = (*client).next;
            }
        }
    } else {
        let mut i = (*st.selmon).clients;
        while !i.is_null() && i != (*st.selmon).sel {
            if isvisible(i) && (*i).isfloating == 0 {
                client = i;
            }
            i = (*i).next;
        }
        if client.is_null() {
            while !i.is_null() {
                if isvisible(i) && (*i).isfloating == 0 {
                    client = i;
                }
                i = (*i).next;
            }
        }
    }

    let mut i = (*st.selmon).clients;
    while !i.is_null() && (p.is_null() || pc.is_null()) {
        if (*i).next == (*st.selmon).sel {
            p = i;
        }
        if (*i).next == client {
            pc = i;
        }
        i = (*i).next;
    }

    if !client.is_null() && client != (*st.selmon).sel {
        let temp = if (*(*st.selmon).sel).next == client {
            (*st.selmon).sel
        } else {
            (*(*st.selmon).sel).next
        };
        (*(*st.selmon).sel).next = if (*client).next == (*st.selmon).sel {
            client
        } else {
            (*client).next
        };
        (*client).next = temp;

        if !p.is_null() && p != client {
            (*p).next = client;
        }
        if !pc.is_null() && pc != (*st.selmon).sel {
            (*pc).next = (*st.selmon).sel;
        }

        if (*st.selmon).sel == (*st.selmon).clients {
            (*st.selmon).clients = client;
        } else if client == (*st.selmon).clients {
            (*st.selmon).clients = (*st.selmon).sel;
        }
        arrange(st.selmon);
    }
}

pub unsafe fn main_entry() {
    init_state();

    let args: Vec<String> = env::args().collect();
    if args.len() == 2 && args[1] == "-v" {
        util::die(format!("dwm-{}", config::VERSION));
    } else if args.len() != 1 {
        util::die("usage: dwm [-v]");
    }

    config::load();

    if libc::setlocale(libc::LC_CTYPE, b"\0".as_ptr().cast()).is_null()
        || xlib::XSupportsLocale() == 0
    {
        eprintln!("warning: no locale support");
    }

    let st = state();
    st.dpy = xlib::XOpenDisplay(ptr::null());
    if st.dpy.is_null() {
        util::die("dwm: cannot open display");
    }
    st.xcon = xlib_xcb::XGetXCBConnection(st.dpy);
    if st.xcon.is_null() {
        util::die("dwm: cannot get xcb connection");
    }

    checkotherwm();
    setup();
    scan();
    run();
    cleanup();
    xlib::XCloseDisplay(st.dpy);
    config::unload();
    destroy_state();
}
