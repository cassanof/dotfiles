use crate::types::{
    ActionFn, Arg, ArrangeFn, Button, Key, Layout, Rule, CLK_CLIENT_WIN, CLK_LT_SYMBOL,
    CLK_ROOT_WIN, CLK_STATUS_TEXT, CLK_TAG_BAR, CLK_WIN_TITLE,
};
use crate::util;
use libc::{c_char, c_float, c_int, c_uint, c_void};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::ffi::CString;
use std::fs;
use std::path::{Path, PathBuf};
use std::ptr;
use x11::xlib;

pub const VERSION: &str = "6.2";

const LOCAL_CONFIG_PATH: &str = "dwm.toml";
const SYSTEM_CONFIG_PATH: &str = "/etc/dwm.toml";

static mut CONFIG: *mut Config = ptr::null_mut();

pub struct Config {
    pub borderpx: c_uint,
    pub gappx: c_uint,
    pub snap: c_uint,
    pub swallowfloating: c_int,
    pub showbar: c_int,
    pub topbar: c_int,
    pub mfact: c_float,
    pub nmaster: c_int,
    pub resizehints: c_int,
    pub fonts: Box<[*const c_char]>,
    pub colors: [Box<[*const c_char]>; 2],
    pub tags: Box<[*const c_char]>,
    pub rules: Box<[Rule]>,
    pub layouts: Box<[Layout]>,
    pub monocle_layout_index: usize,
    pub keys: Box<[Key]>,
    pub buttons: Box<[Button]>,
    _commands: BTreeMap<String, Box<[*const c_char]>>,
    _cstrings: Vec<CString>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawConfig {
    borderpx: c_uint,
    gappx: c_uint,
    snap: c_uint,
    swallowfloating: bool,
    showbar: bool,
    topbar: bool,
    fonts: Vec<String>,
    colors: RawColors,
    tags: Vec<String>,
    #[serde(default)]
    rules: Vec<RawRule>,
    mfact: c_float,
    nmaster: c_int,
    resizehints: bool,
    modkey: String,
    layouts: Vec<RawLayout>,
    commands: BTreeMap<String, Vec<String>>,
    keys: Vec<RawKeyBinding>,
    buttons: Vec<RawButtonBinding>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawColors {
    normal: [String; 3],
    selected: [String; 3],
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawRule {
    class: Option<String>,
    instance: Option<String>,
    title: Option<String>,
    #[serde(default)]
    tags: Vec<usize>,
    #[serde(default)]
    isfloating: bool,
    #[serde(default)]
    isterminal: bool,
    #[serde(default)]
    noswallow: c_int,
    #[serde(default = "default_monitor")]
    monitor: c_int,
}

#[derive(Clone, Copy, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
enum ArrangeKind {
    Tile,
    Monocle,
    None,
    Floating,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawLayout {
    symbol: String,
    arrange: ArrangeKind,
}

#[derive(Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
enum ActionName {
    Spawn,
    Togglebar,
    Focusstack,
    Incnmaster,
    Setmfact,
    Zoom,
    View,
    ViewAdjacent,
    Movestack,
    Killclient,
    Setlayout,
    Togglefloating,
    Focusmon,
    Tagmon,
    Toggleview,
    Tag,
    Toggletag,
    Quit,
    Movemouse,
    Resizemouse,
}

#[derive(Clone, Copy, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
enum ClickTarget {
    TagBar,
    LayoutSymbol,
    StatusText,
    WindowTitle,
    ClientWin,
    RootWin,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawKeyBinding {
    #[serde(default)]
    modifiers: Vec<String>,
    key: String,
    action: ActionName,
    command: Option<String>,
    layout: Option<usize>,
    int: Option<c_int>,
    float: Option<c_float>,
    tag: Option<usize>,
    #[serde(default)]
    all_tags: bool,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawButtonBinding {
    click: ClickTarget,
    #[serde(default)]
    modifiers: Vec<String>,
    button: String,
    action: ActionName,
    command: Option<String>,
    layout: Option<usize>,
    int: Option<c_int>,
    float: Option<c_float>,
    tag: Option<usize>,
    #[serde(default)]
    all_tags: bool,
    #[serde(default)]
    clicked_tag: bool,
}

#[derive(Clone, Copy)]
struct BindingFields<'a> {
    command: Option<&'a str>,
    layout: Option<usize>,
    int: Option<c_int>,
    float: Option<c_float>,
    tag: Option<usize>,
    all_tags: bool,
    clicked_tag: bool,
}

struct ConfigBuilder {
    cstrings: Vec<CString>,
}

impl ArrangeKind {
    fn arrange_fn(self) -> Option<ArrangeFn> {
        match self {
            Self::Tile => Some(crate::wm::tile),
            Self::Monocle => Some(crate::wm::monocle),
            Self::None | Self::Floating => None,
        }
    }
}

impl ActionName {
    fn action_fn(self) -> ActionFn {
        match self {
            Self::Spawn => crate::wm::spawn,
            Self::Togglebar => crate::wm::togglebar,
            Self::Focusstack => crate::wm::focusstack,
            Self::Incnmaster => crate::wm::incnmaster,
            Self::Setmfact => crate::wm::setmfact,
            Self::Zoom => crate::wm::zoom,
            Self::View => crate::wm::view,
            Self::ViewAdjacent => crate::wm::view_adjacent,
            Self::Movestack => crate::wm::movestack,
            Self::Killclient => crate::wm::killclient,
            Self::Setlayout => crate::wm::setlayout,
            Self::Togglefloating => crate::wm::togglefloating,
            Self::Focusmon => crate::wm::focusmon,
            Self::Tagmon => crate::wm::tagmon,
            Self::Toggleview => crate::wm::toggleview,
            Self::Tag => crate::wm::tag,
            Self::Toggletag => crate::wm::toggletag,
            Self::Quit => crate::wm::quit,
            Self::Movemouse => crate::wm::movemouse,
            Self::Resizemouse => crate::wm::resizemouse,
        }
    }

    fn name(self) -> &'static str {
        match self {
            Self::Spawn => "spawn",
            Self::Togglebar => "togglebar",
            Self::Focusstack => "focusstack",
            Self::Incnmaster => "incnmaster",
            Self::Setmfact => "setmfact",
            Self::Zoom => "zoom",
            Self::View => "view",
            Self::ViewAdjacent => "view_adjacent",
            Self::Movestack => "movestack",
            Self::Killclient => "killclient",
            Self::Setlayout => "setlayout",
            Self::Togglefloating => "togglefloating",
            Self::Focusmon => "focusmon",
            Self::Tagmon => "tagmon",
            Self::Toggleview => "toggleview",
            Self::Tag => "tag",
            Self::Toggletag => "toggletag",
            Self::Quit => "quit",
            Self::Movemouse => "movemouse",
            Self::Resizemouse => "resizemouse",
        }
    }
}

impl ClickTarget {
    fn click(self) -> c_uint {
        match self {
            Self::TagBar => CLK_TAG_BAR,
            Self::LayoutSymbol => CLK_LT_SYMBOL,
            Self::StatusText => CLK_STATUS_TEXT,
            Self::WindowTitle => CLK_WIN_TITLE,
            Self::ClientWin => CLK_CLIENT_WIN,
            Self::RootWin => CLK_ROOT_WIN,
        }
    }
}

impl ConfigBuilder {
    fn new() -> Self {
        Self {
            cstrings: Vec::new(),
        }
    }

    fn build(mut self, raw: RawConfig) -> Result<Config, String> {
        if raw.fonts.is_empty() {
            return Err("fonts must contain at least one entry".into());
        }
        if raw.tags.is_empty() {
            return Err("tags must contain at least one entry".into());
        }
        if raw.tags.len() > 31 {
            return Err("tags supports at most 31 entries".into());
        }
        if raw.layouts.len() < 2 {
            return Err("layouts must contain at least two entries".into());
        }

        let modkey = parse_base_modifier(&raw.modkey)?;
        let fonts = self.build_string_ptr_slice(raw.fonts.iter().map(String::as_str))?;
        let colors = [
            self.build_string_ptr_slice(raw.colors.normal.iter().map(String::as_str))?,
            self.build_string_ptr_slice(raw.colors.selected.iter().map(String::as_str))?,
        ];
        let tags = self.build_string_ptr_slice(raw.tags.iter().map(String::as_str))?;
        let tag_count = tags.len();
        let rules = raw
            .rules
            .into_iter()
            .map(|rule| self.build_rule(rule, tag_count))
            .collect::<Result<Vec<_>, _>>()?
            .into_boxed_slice();
        let (layouts, monocle_layout_index) = self.build_layouts(raw.layouts)?;
        let commands = self.build_commands(raw.commands)?;
        let keys = raw
            .keys
            .into_iter()
            .enumerate()
            .map(|(index, binding)| {
                self.build_key(index, binding, &commands, &layouts, modkey, tag_count)
            })
            .collect::<Result<Vec<_>, _>>()?
            .into_boxed_slice();
        let buttons = raw
            .buttons
            .into_iter()
            .enumerate()
            .map(|(index, binding)| {
                self.build_button(index, binding, &commands, &layouts, modkey, tag_count)
            })
            .collect::<Result<Vec<_>, _>>()?
            .into_boxed_slice();

        Ok(Config {
            borderpx: raw.borderpx,
            gappx: raw.gappx,
            snap: raw.snap,
            swallowfloating: bool_to_c_int(raw.swallowfloating),
            showbar: bool_to_c_int(raw.showbar),
            topbar: bool_to_c_int(raw.topbar),
            mfact: raw.mfact,
            nmaster: raw.nmaster,
            resizehints: bool_to_c_int(raw.resizehints),
            fonts,
            colors,
            tags,
            rules,
            layouts,
            monocle_layout_index,
            keys,
            buttons,
            _commands: commands,
            _cstrings: self.cstrings,
        })
    }

    fn build_rule(&mut self, raw: RawRule, tag_count: usize) -> Result<Rule, String> {
        Ok(Rule {
            class: self.intern_optional(raw.class.as_deref())?,
            instance: self.intern_optional(raw.instance.as_deref())?,
            title: self.intern_optional(raw.title.as_deref())?,
            tags: tags_to_mask(&raw.tags, tag_count)?,
            isfloating: bool_to_c_int(raw.isfloating),
            isterminal: bool_to_c_int(raw.isterminal),
            noswallow: raw.noswallow,
            monitor: raw.monitor,
        })
    }

    fn build_layouts(
        &mut self,
        raw_layouts: Vec<RawLayout>,
    ) -> Result<(Box<[Layout]>, usize), String> {
        let mut monocle_layout_index = None;
        let mut layouts = Vec::with_capacity(raw_layouts.len());

        for (index, layout) in raw_layouts.into_iter().enumerate() {
            if layout.arrange == ArrangeKind::Monocle && monocle_layout_index.is_none() {
                monocle_layout_index = Some(index);
            }

            layouts.push(Layout {
                symbol: self.intern(&layout.symbol)?,
                arrange: layout.arrange.arrange_fn(),
            });
        }

        let monocle_layout_index = monocle_layout_index
            .ok_or_else(|| "layouts must include an arrange = \"monocle\" entry".to_string())?;

        Ok((layouts.into_boxed_slice(), monocle_layout_index))
    }

    fn build_commands(
        &mut self,
        raw_commands: BTreeMap<String, Vec<String>>,
    ) -> Result<BTreeMap<String, Box<[*const c_char]>>, String> {
        let mut commands = BTreeMap::new();

        for (name, argv) in raw_commands {
            if name.trim().is_empty() {
                return Err("command names cannot be empty".into());
            }
            if argv.is_empty() {
                return Err(format!(
                    "command {name:?} must contain at least one argv entry"
                ));
            }

            let mut ptrs = Vec::with_capacity(argv.len() + 1);
            for value in argv {
                ptrs.push(self.intern(&value)?);
            }
            ptrs.push(ptr::null());
            commands.insert(name, ptrs.into_boxed_slice());
        }

        Ok(commands)
    }

    fn build_key(
        &self,
        index: usize,
        raw: RawKeyBinding,
        commands: &BTreeMap<String, Box<[*const c_char]>>,
        layouts: &[Layout],
        modkey: c_uint,
        tag_count: usize,
    ) -> Result<Key, String> {
        let RawKeyBinding {
            modifiers,
            key,
            action,
            command,
            layout,
            int,
            float,
            tag,
            all_tags,
        } = raw;

        let mod_ =
            parse_modifiers(&modifiers, modkey).map_err(|err| format!("keys[{index}]: {err}"))?;
        let keysym = parse_keysym(&key).map_err(|err| format!("keys[{index}]: {err}"))?;
        let arg = build_action_arg(
            action,
            BindingFields {
                command: command.as_deref(),
                layout,
                int,
                float,
                tag,
                all_tags,
                clicked_tag: false,
            },
            commands,
            layouts,
            tag_count,
        )
        .map_err(|err| format!("keys[{index}]: {err}"))?;

        Ok(Key {
            mod_,
            keysym,
            func: Some(action.action_fn()),
            arg,
        })
    }

    fn build_button(
        &self,
        index: usize,
        raw: RawButtonBinding,
        commands: &BTreeMap<String, Box<[*const c_char]>>,
        layouts: &[Layout],
        modkey: c_uint,
        tag_count: usize,
    ) -> Result<Button, String> {
        let RawButtonBinding {
            click,
            modifiers,
            button,
            action,
            command,
            layout,
            int,
            float,
            tag,
            all_tags,
            clicked_tag,
        } = raw;

        if clicked_tag && click != ClickTarget::TagBar {
            return Err(format!(
                "buttons[{index}]: clicked_tag is only valid for tag_bar clicks"
            ));
        }

        let mask = parse_modifiers(&modifiers, modkey)
            .map_err(|err| format!("buttons[{index}]: {err}"))?;
        let button = parse_button(&button).map_err(|err| format!("buttons[{index}]: {err}"))?;
        let arg = build_action_arg(
            action,
            BindingFields {
                command: command.as_deref(),
                layout,
                int,
                float,
                tag,
                all_tags,
                clicked_tag,
            },
            commands,
            layouts,
            tag_count,
        )
        .map_err(|err| format!("buttons[{index}]: {err}"))?;

        Ok(Button {
            click: click.click(),
            mask,
            button,
            func: Some(action.action_fn()),
            arg,
        })
    }

    fn build_string_ptr_slice<'a, I>(&mut self, values: I) -> Result<Box<[*const c_char]>, String>
    where
        I: IntoIterator<Item = &'a str>,
    {
        let mut ptrs = Vec::new();
        for value in values {
            ptrs.push(self.intern(value)?);
        }
        Ok(ptrs.into_boxed_slice())
    }

    fn intern(&mut self, value: &str) -> Result<*const c_char, String> {
        let cstring =
            CString::new(value).map_err(|_| format!("string contains a NUL byte: {value:?}"))?;
        let ptr = cstring.as_ptr();
        self.cstrings.push(cstring);
        Ok(ptr)
    }

    fn intern_optional(&mut self, value: Option<&str>) -> Result<*const c_char, String> {
        match value {
            Some(value) => self.intern(value),
            None => Ok(ptr::null()),
        }
    }
}

pub unsafe fn load() {
    if !CONFIG.is_null() {
        return;
    }

    let (path, contents) = read_config_source().unwrap_or_else(|err| util::die(err));
    let raw: RawConfig = toml::from_str(&contents)
        .unwrap_or_else(|err| util::die(format!("failed to parse {}: {err}", path.display())));
    let config = ConfigBuilder::new()
        .build(raw)
        .unwrap_or_else(|err| util::die(format!("invalid config {}: {err}", path.display())));

    CONFIG = Box::into_raw(Box::new(config));
}

pub unsafe fn unload() {
    if !CONFIG.is_null() {
        drop(Box::from_raw(CONFIG));
        CONFIG = ptr::null_mut();
    }
}

pub fn borderpx() -> c_uint {
    unsafe { config().borderpx }
}

pub fn gappx() -> c_uint {
    unsafe { config().gappx }
}

pub fn snap() -> c_uint {
    unsafe { config().snap }
}

pub fn swallowfloating() -> c_int {
    unsafe { config().swallowfloating }
}

pub fn showbar() -> c_int {
    unsafe { config().showbar }
}

pub fn topbar() -> c_int {
    unsafe { config().topbar }
}

pub fn fonts() -> &'static [*const c_char] {
    unsafe { config().fonts.as_ref() }
}

pub fn colors() -> &'static [Box<[*const c_char]>; 2] {
    unsafe { &config().colors }
}

pub fn tags() -> &'static [*const c_char] {
    unsafe { config().tags.as_ref() }
}

pub fn rules() -> &'static [Rule] {
    unsafe { config().rules.as_ref() }
}

pub fn mfact() -> c_float {
    unsafe { config().mfact }
}

pub fn nmaster() -> c_int {
    unsafe { config().nmaster }
}

pub fn resizehints() -> c_int {
    unsafe { config().resizehints }
}

pub fn layouts() -> &'static [Layout] {
    unsafe { config().layouts.as_ref() }
}

pub fn monocle_layout() -> *const Layout {
    unsafe {
        let config = config();
        &config.layouts[config.monocle_layout_index] as *const Layout
    }
}

pub fn keys() -> &'static [Key] {
    unsafe { config().keys.as_ref() }
}

pub fn buttons() -> &'static [Button] {
    unsafe { config().buttons.as_ref() }
}

unsafe fn config() -> &'static Config {
    if CONFIG.is_null() {
        util::die("dwm config is not loaded");
    }

    &*CONFIG
}

fn build_action_arg(
    action: ActionName,
    fields: BindingFields<'_>,
    commands: &BTreeMap<String, Box<[*const c_char]>>,
    layouts: &[Layout],
    tag_count: usize,
) -> Result<Arg, String> {
    match action {
        ActionName::Spawn => {
            if fields.layout.is_some()
                || fields.int.is_some()
                || fields.float.is_some()
                || fields.tag.is_some()
                || fields.all_tags
                || fields.clicked_tag
            {
                return Err("spawn only supports the command field".into());
            }

            let command = fields
                .command
                .ok_or_else(|| "spawn requires command = \"...\"".to_string())?;
            let argv = commands
                .get(command)
                .ok_or_else(|| format!("unknown command {command:?}"))?;

            Ok(Arg {
                v: argv.as_ptr().cast::<c_void>(),
            })
        }
        ActionName::Setlayout => {
            if fields.command.is_some()
                || fields.int.is_some()
                || fields.float.is_some()
                || fields.tag.is_some()
                || fields.all_tags
                || fields.clicked_tag
            {
                return Err("setlayout only supports the layout field".into());
            }

            match fields.layout {
                Some(index) => {
                    let layout = layouts.get(index).ok_or_else(|| {
                        format!(
                            "layout index {index} is out of range, expected 0..{}",
                            layouts.len()
                        )
                    })?;
                    Ok(Arg {
                        v: (layout as *const Layout).cast::<c_void>(),
                    })
                }
                None => Ok(Arg { i: 0 }),
            }
        }
        ActionName::Focusstack
        | ActionName::Incnmaster
        | ActionName::ViewAdjacent
        | ActionName::Movestack
        | ActionName::Focusmon
        | ActionName::Tagmon => {
            if fields.command.is_some()
                || fields.layout.is_some()
                || fields.float.is_some()
                || fields.tag.is_some()
                || fields.all_tags
                || fields.clicked_tag
            {
                return Err(format!("{} only supports the int field", action.name()));
            }

            let value = fields
                .int
                .ok_or_else(|| format!("{} requires int = ...", action.name()))?;
            Ok(Arg { i: value })
        }
        ActionName::Setmfact => {
            if fields.command.is_some()
                || fields.layout.is_some()
                || fields.int.is_some()
                || fields.tag.is_some()
                || fields.all_tags
                || fields.clicked_tag
            {
                return Err("setmfact only supports the float field".into());
            }

            let value = fields
                .float
                .ok_or_else(|| "setmfact requires float = ...".to_string())?;
            Ok(Arg { f: value })
        }
        ActionName::View => {
            if fields.command.is_some()
                || fields.layout.is_some()
                || fields.int.is_some()
                || fields.float.is_some()
            {
                return Err("view only supports tag, all_tags, or clicked_tag".into());
            }
            if fields.clicked_tag && (fields.tag.is_some() || fields.all_tags) {
                return Err("view cannot combine clicked_tag with tag or all_tags".into());
            }
            if fields.all_tags && fields.tag.is_some() {
                return Err("view cannot combine tag with all_tags".into());
            }

            if fields.clicked_tag {
                return Ok(Arg { i: 0 });
            }
            if fields.all_tags {
                return Ok(Arg { ui: u32::MAX });
            }
            if let Some(tag) = fields.tag {
                return Ok(Arg {
                    ui: tag_mask(tag, tag_count)?,
                });
            }

            Ok(Arg { i: 0 })
        }
        ActionName::Toggleview | ActionName::Tag | ActionName::Toggletag => {
            if fields.command.is_some()
                || fields.layout.is_some()
                || fields.int.is_some()
                || fields.float.is_some()
            {
                return Err(format!(
                    "{} only supports tag, all_tags, or clicked_tag",
                    action.name()
                ));
            }
            if fields.clicked_tag && (fields.tag.is_some() || fields.all_tags) {
                return Err(format!(
                    "{} cannot combine clicked_tag with tag or all_tags",
                    action.name()
                ));
            }
            if fields.all_tags && fields.tag.is_some() {
                return Err(format!(
                    "{} cannot combine tag with all_tags",
                    action.name()
                ));
            }

            if fields.clicked_tag {
                return Ok(Arg { i: 0 });
            }
            if fields.all_tags {
                return Ok(Arg { ui: u32::MAX });
            }
            if let Some(tag) = fields.tag {
                return Ok(Arg {
                    ui: tag_mask(tag, tag_count)?,
                });
            }

            Err(format!(
                "{} requires tag = N, all_tags = true, or clicked_tag = true",
                action.name()
            ))
        }
        ActionName::Togglebar
        | ActionName::Zoom
        | ActionName::Killclient
        | ActionName::Togglefloating
        | ActionName::Quit
        | ActionName::Movemouse
        | ActionName::Resizemouse => {
            if fields.command.is_some()
                || fields.layout.is_some()
                || fields.int.is_some()
                || fields.float.is_some()
                || fields.tag.is_some()
                || fields.all_tags
                || fields.clicked_tag
            {
                return Err(format!("{} does not take extra fields", action.name()));
            }

            Ok(Arg { i: 0 })
        }
    }
}

fn read_config_source() -> Result<(PathBuf, String), String> {
    for path in [Path::new(LOCAL_CONFIG_PATH), Path::new(SYSTEM_CONFIG_PATH)] {
        match fs::read_to_string(path) {
            Ok(contents) => return Ok((path.to_path_buf(), contents)),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {}
            Err(err) => {
                return Err(format!("failed to read {}: {err}", path.display()));
            }
        }
    }

    Err(format!(
        "could not find {LOCAL_CONFIG_PATH} or {SYSTEM_CONFIG_PATH}"
    ))
}

fn parse_keysym(name: &str) -> Result<c_uint, String> {
    let keysym_name =
        CString::new(name).map_err(|_| format!("keysym contains a NUL byte: {name:?}"))?;
    let keysym = unsafe { xlib::XStringToKeysym(keysym_name.as_ptr()) };
    if keysym == 0 {
        return Err(format!("unknown keysym {name:?}"));
    }

    Ok(keysym as c_uint)
}

fn parse_modifiers(names: &[String], modkey: c_uint) -> Result<c_uint, String> {
    let mut mask = 0;
    for name in names {
        let modifier = if normalize_name(name) == "modkey" {
            modkey
        } else {
            parse_base_modifier(name)?
        };
        mask |= modifier;
    }
    Ok(mask)
}

fn parse_base_modifier(name: &str) -> Result<c_uint, String> {
    match normalize_name(name).as_str() {
        "shift" => Ok(xlib::ShiftMask),
        "control" | "ctrl" => Ok(xlib::ControlMask),
        "alt" | "mod1" => Ok(xlib::Mod1Mask),
        "mod2" => Ok(xlib::Mod2Mask),
        "mod3" => Ok(xlib::Mod3Mask),
        "mod4" | "super" | "win" => Ok(xlib::Mod4Mask),
        "mod5" => Ok(xlib::Mod5Mask),
        _ => Err(format!("unknown modifier {name:?}")),
    }
}

fn parse_button(name: &str) -> Result<c_uint, String> {
    match normalize_name(name).as_str() {
        "button1" | "left" => Ok(xlib::Button1),
        "button2" | "middle" => Ok(xlib::Button2),
        "button3" | "right" => Ok(xlib::Button3),
        "button4" => Ok(xlib::Button4),
        "button5" => Ok(xlib::Button5),
        _ => Err(format!("unknown button {name:?}")),
    }
}

fn tag_mask(tag: usize, tag_count: usize) -> Result<c_uint, String> {
    if tag == 0 || tag > tag_count {
        return Err(format!(
            "tag {tag} is out of range, expected 1..={tag_count}"
        ));
    }

    Ok(1u32 << (tag - 1))
}

fn tags_to_mask(tags: &[usize], tag_count: usize) -> Result<c_uint, String> {
    let mut mask = 0;
    for tag in tags {
        mask |= tag_mask(*tag, tag_count)?;
    }
    Ok(mask)
}

fn normalize_name(value: &str) -> String {
    value.trim().to_ascii_lowercase()
}

fn bool_to_c_int(value: bool) -> c_int {
    value as c_int
}

fn default_monitor() -> c_int {
    -1
}
