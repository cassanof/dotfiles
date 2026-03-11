use libc::{c_char, c_int, c_uint, c_void};
use x11::xft::{FcChar32, FcCharSet, FcPattern};
use x11::xlib_xcb::xcb_connection_t;

pub type FcBool = c_int;
pub const FC_FALSE: FcBool = 0;
pub const FC_TRUE: FcBool = 1;
pub const FC_MATCH_PATTERN: c_int = 0;

pub static FC_CHARSET: &[u8] = b"charset\0";
pub static FC_COLOR: &[u8] = b"color\0";
pub static FC_SCALABLE: &[u8] = b"scalable\0";

pub const XC_FLEUR: c_uint = 52;
pub const XC_LEFT_PTR: c_uint = 68;
pub const XC_SIZING: c_uint = 120;

pub const WITHDRAWN_STATE: libc::c_long = 0;
pub const NORMAL_STATE: libc::c_long = 1;
pub const ICONIC_STATE: libc::c_long = 3;

pub const XCB_RES_CLIENT_ID_MASK_LOCAL_CLIENT_PID: u32 = 2;

pub enum FcConfig {}
#[allow(non_camel_case_types)]
pub type xcb_generic_error_t = c_void;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct xcb_res_client_id_spec_t {
    pub client: u32,
    pub mask: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct xcb_res_client_id_value_t {
    pub spec: xcb_res_client_id_spec_t,
    pub length: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct xcb_res_client_id_value_iterator_t {
    pub data: *mut xcb_res_client_id_value_t,
    pub rem: c_int,
    pub index: c_int,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct xcb_res_query_client_ids_cookie_t {
    pub sequence: c_uint,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct xcb_res_query_client_ids_reply_t {
    pub response_type: u8,
    pub pad0: u8,
    pub sequence: u16,
    pub length: u32,
    pub num_ids: u32,
    pub pad1: [u8; 20],
}

#[link(name = "fontconfig")]
extern "C" {
    pub fn FcConfigSubstitute(
        config: *mut FcConfig,
        pattern: *mut FcPattern,
        kind: c_int,
    ) -> FcBool;
    pub fn FcCharSetCreate() -> *mut FcCharSet;
    pub fn FcCharSetDestroy(char_set: *mut FcCharSet);
    pub fn FcCharSetAddChar(char_set: *mut FcCharSet, value: FcChar32) -> FcBool;
    pub fn FcDefaultSubstitute(pattern: *mut FcPattern);
    pub fn FcNameParse(name: *const u8) -> *mut FcPattern;
    pub fn FcPatternDuplicate(pattern: *const FcPattern) -> *mut FcPattern;
    pub fn FcPatternDestroy(pattern: *mut FcPattern);
    pub fn FcPatternAddCharSet(
        pattern: *mut FcPattern,
        object: *const c_char,
        char_set: *const FcCharSet,
    ) -> FcBool;
    pub fn FcPatternAddBool(
        pattern: *mut FcPattern,
        object: *const c_char,
        value: FcBool,
    ) -> FcBool;
    pub fn FcPatternGetBool(
        pattern: *const FcPattern,
        object: *const c_char,
        index: c_int,
        value: *mut FcBool,
    ) -> c_int;
}

#[link(name = "xcb-res")]
extern "C" {
    pub fn xcb_res_query_client_ids(
        connection: *mut xcb_connection_t,
        num_specs: u32,
        specs: *const xcb_res_client_id_spec_t,
    ) -> xcb_res_query_client_ids_cookie_t;
    pub fn xcb_res_query_client_ids_reply(
        connection: *mut xcb_connection_t,
        cookie: xcb_res_query_client_ids_cookie_t,
        error: *mut *mut xcb_generic_error_t,
    ) -> *mut xcb_res_query_client_ids_reply_t;
    pub fn xcb_res_query_client_ids_ids_iterator(
        reply: *const xcb_res_query_client_ids_reply_t,
    ) -> xcb_res_client_id_value_iterator_t;
    pub fn xcb_res_client_id_value_value(value: *const xcb_res_client_id_value_t) -> *mut u32;
    pub fn xcb_res_client_id_value_next(iterator: *mut xcb_res_client_id_value_iterator_t);
}
