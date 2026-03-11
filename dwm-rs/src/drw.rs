use crate::ffi;
use crate::util;
use libc::{c_char, c_int, c_long, c_uint};
use std::ffi::CStr;
use std::mem;
use std::ptr;
use x11::xft;
use x11::xlib;
use x11::xrender;

const UTF_INVALID: c_long = 0xFFFD;
const UTF_SIZ: usize = 4;
const UTF_BYTE: [u8; UTF_SIZ + 1] = [0x80, 0, 0xC0, 0xE0, 0xF0];
const UTF_MASK: [u8; UTF_SIZ + 1] = [0xC0, 0x80, 0xE0, 0xF0, 0xF8];
const UTF_MIN: [c_long; UTF_SIZ + 1] = [0, 0, 0x80, 0x800, 0x10000];
const UTF_MAX: [c_long; UTF_SIZ + 1] = [0x10FFFF, 0x7F, 0x7FF, 0xFFFF, 0x10FFFF];

pub type Clr = xft::XftColor;

#[repr(C)]
pub struct Cur {
    pub cursor: xlib::Cursor,
}

#[repr(C)]
pub struct Fnt {
    pub dpy: *mut xlib::Display,
    pub h: c_uint,
    pub xfont: *mut xft::XftFont,
    pub pattern: *mut xft::FcPattern,
    pub next: *mut Fnt,
}

#[repr(C)]
pub struct Drw {
    pub w: c_uint,
    pub h: c_uint,
    pub dpy: *mut xlib::Display,
    pub screen: c_int,
    pub root: xlib::Window,
    pub drawable: xlib::Drawable,
    pub gc: xlib::GC,
    pub scheme: *mut Clr,
    pub fonts: *mut Fnt,
}

unsafe fn utf8decodebyte(c: c_char, index: &mut usize) -> c_long {
    for current in 0..=UTF_SIZ {
        if ((c as u8) & UTF_MASK[current]) == UTF_BYTE[current] {
            *index = current;
            return ((c as u8) & !UTF_MASK[current]) as c_long;
        }
    }

    0
}

unsafe fn utf8validate(codepoint: &mut c_long, mut index: usize) -> usize {
    if !util::between(*codepoint, UTF_MIN[index], UTF_MAX[index])
        || util::between(*codepoint, 0xD800, 0xDFFF)
    {
        *codepoint = UTF_INVALID;
    }

    while *codepoint > UTF_MAX[index] {
        index += 1;
    }

    index
}

unsafe fn utf8decode(text: *const c_char, codepoint: &mut c_long, clen: usize) -> usize {
    *codepoint = UTF_INVALID;
    if clen == 0 {
        return 0;
    }

    let mut len = 0usize;
    let mut decoded = utf8decodebyte(*text, &mut len);
    if !util::between(len as i64, 1, UTF_SIZ as i64) {
        return 1;
    }

    let mut i = 1usize;
    let mut j = 1usize;
    while i < clen && j < len {
        let mut kind = 0usize;
        decoded = (decoded << 6) | utf8decodebyte(*text.add(i), &mut kind);
        if kind != 0 {
            return j;
        }

        i += 1;
        j += 1;
    }

    if j < len {
        return 0;
    }

    *codepoint = decoded;
    utf8validate(codepoint, len)
}

unsafe fn boxed_zeroed<T>() -> *mut T {
    Box::into_raw(Box::new(mem::zeroed()))
}

unsafe fn xfont_create(
    drw: *mut Drw,
    fontname: *const c_char,
    fontpattern: *mut xft::FcPattern,
) -> *mut Fnt {
    let xfont: *mut xft::XftFont;
    let mut pattern: *mut xft::FcPattern = ptr::null_mut();

    if !fontname.is_null() {
        xfont = xft::XftFontOpenName((*drw).dpy, (*drw).screen, fontname);
        if xfont.is_null() {
            eprintln!(
                "error, cannot load font from name: '{}'",
                CStr::from_ptr(fontname).to_string_lossy()
            );
            return ptr::null_mut();
        }

        pattern = ffi::FcNameParse(fontname.cast());
        if pattern.is_null() {
            eprintln!(
                "error, cannot parse font name to pattern: '{}'",
                CStr::from_ptr(fontname).to_string_lossy()
            );
            xft::XftFontClose((*drw).dpy, xfont);
            return ptr::null_mut();
        }
    } else if !fontpattern.is_null() {
        xfont = xft::XftFontOpenPattern((*drw).dpy, fontpattern);
        if xfont.is_null() {
            eprintln!("error, cannot load font from pattern.");
            return ptr::null_mut();
        }
    } else {
        util::die("no font specified.");
    }

    let mut iscol: ffi::FcBool = 0;
    if ffi::FcPatternGetBool(
        (*xfont).pattern,
        ffi::FC_COLOR.as_ptr().cast(),
        0,
        &mut iscol,
    ) == xft::FcResult::Match as c_int
        && iscol != 0
    {
        xft::XftFontClose((*drw).dpy, xfont);
        return ptr::null_mut();
    }

    let font = boxed_zeroed::<Fnt>();
    (*font).xfont = xfont;
    (*font).pattern = pattern;
    (*font).h = ((*xfont).ascent + (*xfont).descent) as c_uint;
    (*font).dpy = (*drw).dpy;
    font
}

unsafe fn xfont_free(font: *mut Fnt) {
    if font.is_null() {
        return;
    }

    if !(*font).pattern.is_null() {
        ffi::FcPatternDestroy((*font).pattern);
    }
    xft::XftFontClose((*font).dpy, (*font).xfont);
    drop(Box::from_raw(font));
}

pub unsafe fn drw_create(
    dpy: *mut xlib::Display,
    screen: c_int,
    root: xlib::Window,
    w: c_uint,
    h: c_uint,
) -> *mut Drw {
    let drw = boxed_zeroed::<Drw>();
    (*drw).dpy = dpy;
    (*drw).screen = screen;
    (*drw).root = root;
    (*drw).w = w;
    (*drw).h = h;
    (*drw).drawable =
        xlib::XCreatePixmap(dpy, root, w, h, xlib::XDefaultDepth(dpy, screen) as c_uint);
    (*drw).gc = xlib::XCreateGC(dpy, root, 0, ptr::null_mut());
    xlib::XSetLineAttributes(
        dpy,
        (*drw).gc,
        1,
        xlib::LineSolid,
        xlib::CapButt,
        xlib::JoinMiter,
    );

    drw
}

pub unsafe fn drw_resize(drw: *mut Drw, w: c_uint, h: c_uint) {
    if drw.is_null() {
        return;
    }

    (*drw).w = w;
    (*drw).h = h;
    if (*drw).drawable != 0 {
        xlib::XFreePixmap((*drw).dpy, (*drw).drawable);
    }
    (*drw).drawable = xlib::XCreatePixmap(
        (*drw).dpy,
        (*drw).root,
        w,
        h,
        xlib::XDefaultDepth((*drw).dpy, (*drw).screen) as c_uint,
    );
}

pub unsafe fn drw_free(drw: *mut Drw) {
    if drw.is_null() {
        return;
    }

    xlib::XFreePixmap((*drw).dpy, (*drw).drawable);
    xlib::XFreeGC((*drw).dpy, (*drw).gc);
    drop(Box::from_raw(drw));
}

pub unsafe fn drw_fontset_create(drw: *mut Drw, fonts: &[*const c_char]) -> *mut Fnt {
    if drw.is_null() || fonts.is_empty() {
        return ptr::null_mut();
    }

    let mut ret: *mut Fnt = ptr::null_mut();
    for fontname in fonts.iter().rev().copied() {
        let cur = xfont_create(drw, fontname, ptr::null_mut());
        if !cur.is_null() {
            (*cur).next = ret;
            ret = cur;
        }
    }

    (*drw).fonts = ret;
    ret
}

pub unsafe fn drw_fontset_free(font: *mut Fnt) {
    if font.is_null() {
        return;
    }

    drw_fontset_free((*font).next);
    xfont_free(font);
}

pub unsafe fn drw_clr_create(drw: *mut Drw, dest: *mut Clr, clrname: *const c_char) {
    if drw.is_null() || dest.is_null() || clrname.is_null() {
        return;
    }

    if xft::XftColorAllocName(
        (*drw).dpy,
        xlib::XDefaultVisual((*drw).dpy, (*drw).screen),
        xlib::XDefaultColormap((*drw).dpy, (*drw).screen),
        clrname,
        dest,
    ) == 0
    {
        util::die(format!(
            "error, cannot allocate color '{}'",
            CStr::from_ptr(clrname).to_string_lossy()
        ));
    }
}

pub unsafe fn drw_scm_create(drw: *mut Drw, clrnames: &[*const c_char]) -> *mut Clr {
    if drw.is_null() || clrnames.len() < 2 {
        return ptr::null_mut();
    }

    let mut ret: Box<[Clr]> = vec![mem::zeroed::<Clr>(); clrnames.len()].into_boxed_slice();
    for (index, clrname) in clrnames.iter().copied().enumerate() {
        drw_clr_create(drw, &mut ret[index], clrname);
    }

    Box::into_raw(ret) as *mut Clr
}

#[allow(dead_code)]
pub unsafe fn drw_setfontset(drw: *mut Drw, set: *mut Fnt) {
    if !drw.is_null() {
        (*drw).fonts = set;
    }
}

pub unsafe fn drw_setscheme(drw: *mut Drw, scm: *mut Clr) {
    if !drw.is_null() {
        (*drw).scheme = scm;
    }
}

pub unsafe fn drw_rect(
    drw: *mut Drw,
    x: c_int,
    y: c_int,
    w: c_uint,
    h: c_uint,
    filled: c_int,
    invert: c_int,
) {
    if drw.is_null() || (*drw).scheme.is_null() {
        return;
    }

    let pixel = if invert != 0 {
        (*(*drw).scheme.add(1)).pixel
    } else {
        (*(*drw).scheme.add(0)).pixel
    };
    xlib::XSetForeground((*drw).dpy, (*drw).gc, pixel);
    if filled != 0 {
        xlib::XFillRectangle((*drw).dpy, (*drw).drawable, (*drw).gc, x, y, w, h);
    } else {
        xlib::XDrawRectangle(
            (*drw).dpy,
            (*drw).drawable,
            (*drw).gc,
            x,
            y,
            w.saturating_sub(1),
            h.saturating_sub(1),
        );
    }
}

pub unsafe fn drw_text(
    drw: *mut Drw,
    mut x: c_int,
    y: c_int,
    mut w: c_uint,
    h: c_uint,
    lpad: c_uint,
    mut text: *const c_char,
    invert: c_int,
) -> c_int {
    let mut buf = [0u8; 1024];
    let mut ew: c_uint = 0;
    let render = x != 0 || y != 0 || w != 0 || h != 0;

    if drw.is_null()
        || text.is_null()
        || (*drw).fonts.is_null()
        || (render && (*drw).scheme.is_null())
    {
        return 0;
    }

    let mut d: *mut xft::XftDraw = ptr::null_mut();
    if !render {
        w = !w;
    } else {
        let pixel = if invert != 0 {
            (*(*drw).scheme.add(0)).pixel
        } else {
            (*(*drw).scheme.add(1)).pixel
        };
        xlib::XSetForeground((*drw).dpy, (*drw).gc, pixel);
        xlib::XFillRectangle((*drw).dpy, (*drw).drawable, (*drw).gc, x, y, w, h);
        d = xft::XftDrawCreate(
            (*drw).dpy,
            (*drw).drawable,
            xlib::XDefaultVisual((*drw).dpy, (*drw).screen),
            xlib::XDefaultColormap((*drw).dpy, (*drw).screen),
        );
        x += lpad as c_int;
        w = w.saturating_sub(lpad);
    }

    let mut usedfont = (*drw).fonts;
    loop {
        let mut utf8strlen = 0usize;
        let utf8str = text;
        let mut nextfont: *mut Fnt = ptr::null_mut();
        while *text != 0 {
            let mut utf8codepoint: c_long = 0;
            let utf8charlen = utf8decode(text, &mut utf8codepoint, UTF_SIZ);
            let mut curfont = (*drw).fonts;
            let mut charexists = false;
            while !curfont.is_null() {
                charexists = charexists
                    || xft::XftCharExists((*drw).dpy, (*curfont).xfont, utf8codepoint as c_uint)
                        != 0;
                if charexists {
                    if curfont == usedfont {
                        utf8strlen += utf8charlen;
                        text = text.add(utf8charlen);
                    } else {
                        nextfont = curfont;
                    }
                    break;
                }
                curfont = (*curfont).next;
            }

            if !charexists || !nextfont.is_null() {
                break;
            }
        }

        if utf8strlen != 0 {
            drw_font_getexts(
                usedfont,
                utf8str,
                utf8strlen as c_uint,
                &mut ew,
                ptr::null_mut(),
            );
            let mut len = utf8strlen.min(buf.len() - 1);
            while len != 0 && ew > w {
                len -= 1;
                drw_font_getexts(usedfont, utf8str, len as c_uint, &mut ew, ptr::null_mut());
            }

            if len != 0 {
                ptr::copy_nonoverlapping(utf8str.cast::<u8>(), buf.as_mut_ptr(), len);
                buf[len] = 0;
                if len < utf8strlen {
                    let start = len.saturating_sub(3);
                    for index in start..len {
                        buf[index] = b'.';
                    }
                }

                if render {
                    let ty =
                        y + (h as c_int - (*usedfont).h as c_int) / 2 + (*(*usedfont).xfont).ascent;
                    let color_index = if invert != 0 { 1 } else { 0 };
                    xft::XftDrawStringUtf8(
                        d,
                        (*drw).scheme.add(color_index),
                        (*usedfont).xfont,
                        x,
                        ty,
                        buf.as_ptr(),
                        len as c_int,
                    );
                }

                x += ew as c_int;
                w = w.saturating_sub(ew);
            }
        }

        if *text == 0 {
            break;
        } else if !nextfont.is_null() {
            usedfont = nextfont;
        } else {
            let mut utf8codepoint: c_long = 0;
            utf8decode(text, &mut utf8codepoint, UTF_SIZ);

            let fccharset = ffi::FcCharSetCreate();
            ffi::FcCharSetAddChar(fccharset, utf8codepoint as u32);

            if (*(*drw).fonts).pattern.is_null() {
                util::die("the first font in the cache must be loaded from a font string.");
            }

            let fcpattern = ffi::FcPatternDuplicate((*(*drw).fonts).pattern);
            ffi::FcPatternAddCharSet(fcpattern, ffi::FC_CHARSET.as_ptr().cast(), fccharset);
            ffi::FcPatternAddBool(fcpattern, ffi::FC_SCALABLE.as_ptr().cast(), ffi::FC_TRUE);
            ffi::FcPatternAddBool(fcpattern, ffi::FC_COLOR.as_ptr().cast(), ffi::FC_FALSE);
            ffi::FcConfigSubstitute(ptr::null_mut(), fcpattern, ffi::FC_MATCH_PATTERN);
            ffi::FcDefaultSubstitute(fcpattern);

            let mut result: xft::FcResult = mem::zeroed();
            let match_pattern =
                xft::XftFontMatch((*drw).dpy, (*drw).screen, fcpattern, &mut result);

            ffi::FcCharSetDestroy(fccharset);
            ffi::FcPatternDestroy(fcpattern);

            if !match_pattern.is_null() {
                usedfont = xfont_create(drw, ptr::null(), match_pattern);
                if !usedfont.is_null()
                    && xft::XftCharExists((*drw).dpy, (*usedfont).xfont, utf8codepoint as c_uint)
                        != 0
                {
                    let mut curfont = (*drw).fonts;
                    while !(*curfont).next.is_null() {
                        curfont = (*curfont).next;
                    }
                    (*curfont).next = usedfont;
                } else {
                    xfont_free(usedfont);
                    usedfont = (*drw).fonts;
                }
            }
        }
    }

    if !d.is_null() {
        xft::XftDrawDestroy(d);
    }

    x + if render { w as c_int } else { 0 }
}

pub unsafe fn drw_map(drw: *mut Drw, win: xlib::Window, x: c_int, y: c_int, w: c_uint, h: c_uint) {
    if drw.is_null() {
        return;
    }

    xlib::XCopyArea(
        (*drw).dpy,
        (*drw).drawable,
        win,
        (*drw).gc,
        x,
        y,
        w,
        h,
        x,
        y,
    );
    xlib::XSync((*drw).dpy, xlib::False);
}

pub unsafe fn drw_fontset_getwidth(drw: *mut Drw, text: *const c_char) -> c_uint {
    if drw.is_null() || text.is_null() || (*drw).fonts.is_null() {
        return 0;
    }

    drw_text(drw, 0, 0, 0, 0, 0, text, 0) as c_uint
}

pub unsafe fn drw_font_getexts(
    font: *mut Fnt,
    text: *const c_char,
    len: c_uint,
    w: *mut c_uint,
    h: *mut c_uint,
) {
    if font.is_null() || text.is_null() {
        return;
    }

    let mut ext: xrender::XGlyphInfo = mem::zeroed();
    xft::XftTextExtentsUtf8(
        (*font).dpy,
        (*font).xfont,
        text.cast(),
        len as c_int,
        &mut ext,
    );
    if !w.is_null() {
        *w = ext.xOff as c_uint;
    }
    if !h.is_null() {
        *h = (*font).h;
    }
}

pub unsafe fn drw_cur_create(drw: *mut Drw, shape: c_uint) -> *mut Cur {
    if drw.is_null() {
        return ptr::null_mut();
    }

    let cur = boxed_zeroed::<Cur>();
    (*cur).cursor = xlib::XCreateFontCursor((*drw).dpy, shape);
    cur
}

pub unsafe fn drw_cur_free(drw: *mut Drw, cursor: *mut Cur) {
    if cursor.is_null() {
        return;
    }

    xlib::XFreeCursor((*drw).dpy, (*cursor).cursor);
    drop(Box::from_raw(cursor));
}
