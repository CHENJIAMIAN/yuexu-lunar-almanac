use std::{
    env,
    ffi::{OsStr, OsString, c_void},
    mem::{size_of, zeroed},
    os::windows::ffi::{OsStrExt, OsStringExt},
    path::PathBuf,
    ptr::{null, null_mut},
    slice,
};

use anyhow::{Context, Result, bail};
use chrono::NaiveDate;

use crate::{
    apply_selected_theme,
    calendar::{CustomTheme, LayoutSettings, Palette, Theme},
    export_theme, load_custom_theme, render_svg_pixels,
};

const PREVIEW_WIDTH: u32 = 1280;
const PREVIEW_HEIGHT: u32 = 720;
const SETTINGS_WINDOW_WIDTH: i32 = 1500;
const SETTINGS_WINDOW_HEIGHT: i32 = 1040;
const WINDOW_CLASS: &str = "YueXuNativeSettingsWindow";

const WM_DESTROY: u32 = 0x0002;
const WM_PAINT: u32 = 0x000F;
const WM_CLOSE: u32 = 0x0010;
const WM_ERASEBKGND: u32 = 0x0014;
const WM_GETMINMAXINFO: u32 = 0x0024;
const WM_MOUSEMOVE: u32 = 0x0200;
const WM_LBUTTONDOWN: u32 = 0x0201;
const WM_LBUTTONUP: u32 = 0x0202;
const WM_COMMAND: u32 = 0x0111;
const WM_INITDIALOG: u32 = 0x0110;
const WM_TIMER: u32 = 0x0113;
const WM_NCCREATE: u32 = 0x0081;
const WM_NCDESTROY: u32 = 0x0082;
const GWLP_USERDATA: i32 = -21;
const CS_HREDRAW: u32 = 0x0002;
const CS_VREDRAW: u32 = 0x0001;
const WS_OVERLAPPEDWINDOW: u32 = 0x00CF0000;
const CW_USEDEFAULT: i32 = 0x8000_0000u32 as i32;
const SW_SHOW: i32 = 5;
const DT_CENTER: u32 = 0x0000_0001;
const DT_VCENTER: u32 = 0x0000_0004;
const DT_SINGLELINE: u32 = 0x0000_0020;
const DT_END_ELLIPSIS: u32 = 0x0000_8000;
const DT_LEFT: u32 = 0x0000_0000;
const TRANSPARENT: i32 = 1;
const FW_NORMAL: i32 = 400;
const FW_SEMIBOLD: i32 = 600;
const DIB_RGB_COLORS: u32 = 0;
const BI_RGB: u32 = 0;
const SRCCOPY: u32 = 0x00CC0020;
const HALFTONE: i32 = 4;
const OFN_OVERWRITEPROMPT: u32 = 0x0000_0002;
const OFN_HIDEREADONLY: u32 = 0x0000_0004;
const OFN_NOCHANGEDIR: u32 = 0x0000_0008;
const OFN_PATHMUSTEXIST: u32 = 0x0000_0800;
const OFN_FILEMUSTEXIST: u32 = 0x0000_1000;
const CC_RGBINIT: u32 = 0x0000_0001;
const CC_FULLOPEN: u32 = 0x0000_0002;
const CC_ENABLEHOOK: u32 = 0x0000_0010;
const IMAGE_ICON: u32 = 1;
const LR_LOADFROMFILE: u32 = 0x0000_0010;
const EN_CHANGE: usize = 0x0300;
const COLOR_RED: i32 = 706;
const COLOR_GREEN: i32 = 707;
const COLOR_BLUE: i32 = 708;
const COLOR_PREVIEW_TIMER_ID: usize = 0x59_43;
const COLOR_PREVIEW_DELAY_MS: u32 = 60;

type Handle = isize;
type Hwnd = Handle;
type Hdc = Handle;
type Hbitmap = Handle;
type Hfont = Handle;
type Hinstance = Handle;

#[repr(C)]
#[derive(Clone, Copy, Default)]
struct Rect {
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
}

impl Rect {
    fn new(left: i32, top: i32, right: i32, bottom: i32) -> Self {
        Self {
            left,
            top,
            right,
            bottom,
        }
    }

    fn width(self) -> i32 {
        self.right - self.left
    }

    fn height(self) -> i32 {
        self.bottom - self.top
    }

    fn contains(self, x: i32, y: i32) -> bool {
        x >= self.left && x < self.right && y >= self.top && y < self.bottom
    }
}

#[repr(C)]
#[derive(Default)]
struct Point {
    x: i32,
    y: i32,
}

#[repr(C)]
#[derive(Default)]
struct PaintStruct {
    hdc: Hdc,
    f_erase: i32,
    rc_paint: Rect,
    f_restore: i32,
    f_inc_update: i32,
    rgb_reserved: [u8; 32],
}

#[repr(C)]
struct Msg {
    hwnd: Hwnd,
    message: u32,
    w_param: usize,
    l_param: isize,
    time: u32,
    point: Point,
    l_private: u32,
}

#[repr(C)]
struct MinMaxInfo {
    reserved: Point,
    max_size: Point,
    max_position: Point,
    min_track_size: Point,
    max_track_size: Point,
}

#[repr(C)]
struct WndClassExW {
    cb_size: u32,
    style: u32,
    window_proc: Option<unsafe extern "system" fn(Hwnd, u32, usize, isize) -> isize>,
    cls_extra: i32,
    wnd_extra: i32,
    instance: Hinstance,
    icon: Handle,
    cursor: Handle,
    background: Handle,
    menu_name: *const u16,
    class_name: *const u16,
    icon_small: Handle,
}

#[repr(C)]
struct CreateStructW {
    create_params: *mut c_void,
    instance: Hinstance,
    menu: Handle,
    parent: Hwnd,
    height: i32,
    width: i32,
    y: i32,
    x: i32,
    style: i32,
    name: *const u16,
    class: *const u16,
    ex_style: u32,
}

#[repr(C)]
struct BitmapInfoHeader {
    size: u32,
    width: i32,
    height: i32,
    planes: u16,
    bit_count: u16,
    compression: u32,
    size_image: u32,
    x_pels_per_meter: i32,
    y_pels_per_meter: i32,
    clr_used: u32,
    clr_important: u32,
}

#[repr(C)]
#[derive(Default)]
struct RgbQuad {
    blue: u8,
    green: u8,
    red: u8,
    reserved: u8,
}

#[repr(C)]
struct BitmapInfo {
    header: BitmapInfoHeader,
    colors: [RgbQuad; 1],
}

#[repr(C)]
struct ChooseColorW {
    size: u32,
    owner: Hwnd,
    instance: Hinstance,
    result: u32,
    custom_colors: *mut u32,
    flags: u32,
    custom_data: isize,
    hook: Option<unsafe extern "system" fn(Hwnd, u32, usize, isize) -> isize>,
    template_name: *const u16,
}

#[repr(C)]
struct OpenFileNameW {
    size: u32,
    owner: Hwnd,
    instance: Hinstance,
    filter: *const u16,
    custom_filter: *mut u16,
    max_custom_filter: u32,
    filter_index: u32,
    file: *mut u16,
    max_file: u32,
    file_title: *mut u16,
    max_file_title: u32,
    initial_dir: *const u16,
    title: *const u16,
    flags: u32,
    file_offset: u16,
    file_extension: u16,
    default_ext: *const u16,
    custom_data: isize,
    hook: *const c_void,
    template_name: *const u16,
    reserved: *mut c_void,
    reserved_value: u32,
    flags_ex: u32,
}

#[derive(Clone, Copy)]
enum Action {
    Dark,
    Light,
    Custom,
    AdjustMargin(MarginField),
    Edit(PaletteField),
    Import,
    Export,
    Apply,
    PreviousYear,
    NextYear,
    Close,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum MarginField {
    Left,
    Right,
    Top,
    Bottom,
}

impl MarginField {
    const ALL: [(Self, &'static str); 4] = [
        (Self::Left, "左侧安全区"),
        (Self::Right, "右侧边距"),
        (Self::Top, "上边距"),
        (Self::Bottom, "下边距"),
    ];

    fn range(self) -> (u8, u8) {
        match self {
            Self::Left => (
                LayoutSettings::MIN_LEFT_MARGIN,
                LayoutSettings::MAX_LEFT_MARGIN,
            ),
            Self::Right => (
                LayoutSettings::MIN_RIGHT_MARGIN,
                LayoutSettings::MAX_RIGHT_MARGIN,
            ),
            Self::Top => (
                LayoutSettings::MIN_TOP_MARGIN,
                LayoutSettings::MAX_TOP_MARGIN,
            ),
            Self::Bottom => (
                LayoutSettings::MIN_BOTTOM_MARGIN,
                LayoutSettings::MAX_BOTTOM_MARGIN,
            ),
        }
    }

    fn value(self, layout: LayoutSettings) -> u8 {
        match self {
            Self::Left => layout.left_margin,
            Self::Right => layout.right_margin,
            Self::Top => layout.top_margin,
            Self::Bottom => layout.bottom_margin,
        }
    }

    fn set(self, layout: &mut LayoutSettings, value: u8) {
        match self {
            Self::Left => layout.left_margin = value,
            Self::Right => layout.right_margin = value,
            Self::Top => layout.top_margin = value,
            Self::Bottom => layout.bottom_margin = value,
        }
    }
}

#[derive(Clone, Copy)]
enum PaletteField {
    Paper,
    Gutter,
    Card,
    CurrentCard,
    Ink,
    Soft,
    Muted,
    Accent,
    AccentSoft,
    Line,
    Footer,
}

impl PaletteField {
    const ALL: [(Self, &'static str); 11] = [
        (Self::Paper, "主背景"),
        (Self::Gutter, "图标区"),
        (Self::Card, "月卡"),
        (Self::CurrentCard, "当月卡"),
        (Self::Ink, "正文"),
        (Self::Soft, "辅助文字"),
        (Self::Muted, "次级文字"),
        (Self::Accent, "今日高亮"),
        (Self::AccentSoft, "周末"),
        (Self::Line, "分隔线"),
        (Self::Footer, "页脚"),
    ];

    fn color(self, palette: &Palette) -> &str {
        match self {
            Self::Paper => &palette.paper,
            Self::Gutter => &palette.gutter,
            Self::Card => &palette.card,
            Self::CurrentCard => &palette.current_card,
            Self::Ink => &palette.ink,
            Self::Soft => &palette.soft,
            Self::Muted => &palette.muted,
            Self::Accent => &palette.accent,
            Self::AccentSoft => &palette.accent_soft,
            Self::Line => &palette.line,
            Self::Footer => &palette.footer,
        }
    }

    fn color_mut(self, palette: &mut Palette) -> &mut String {
        match self {
            Self::Paper => &mut palette.paper,
            Self::Gutter => &mut palette.gutter,
            Self::Card => &mut palette.card,
            Self::CurrentCard => &mut palette.current_card,
            Self::Ink => &mut palette.ink,
            Self::Soft => &mut palette.soft,
            Self::Muted => &mut palette.muted,
            Self::Accent => &mut palette.accent,
            Self::AccentSoft => &mut palette.accent_soft,
            Self::Line => &mut palette.line,
            Self::Footer => &mut palette.footer,
        }
    }
}

#[derive(Clone, Copy)]
struct HitTarget {
    rect: Rect,
    action: Action,
}

struct SettingsWindow {
    year: i32,
    today: NaiveDate,
    theme: Theme,
    saved_custom_theme: Option<CustomTheme>,
    layout: LayoutSettings,
    preview_bitmap: Hbitmap,
    preview_width: i32,
    preview_height: i32,
    font_title: Hfont,
    font_body: Hfont,
    font_small: Hfont,
    hits: Vec<HitTarget>,
    custom_colors: [u32; 16],
    status: String,
    dirty: bool,
    scale: f32,
    active_margin: Option<MarginField>,
}

struct ColorPreviewContext {
    state: *mut SettingsWindow,
    owner: Hwnd,
    field: PaletteField,
    original_theme: Theme,
    original_saved_custom_theme: Option<CustomTheme>,
    original_status: String,
    original_dirty: bool,
    dialog: Hwnd,
    last_color: u32,
}

impl SettingsWindow {
    fn new(
        year: i32,
        theme: Theme,
        today: NaiveDate,
        saved_custom_theme: Option<CustomTheme>,
        layout: LayoutSettings,
        scale: f32,
    ) -> Self {
        Self {
            year,
            today,
            theme,
            saved_custom_theme,
            layout: layout.normalized(),
            preview_bitmap: 0,
            preview_width: PREVIEW_WIDTH as i32,
            preview_height: PREVIEW_HEIGHT as i32,
            font_title: 0,
            font_body: 0,
            font_small: 0,
            hits: Vec::new(),
            custom_colors: [0; 16],
            status: "选择主题或颜色，然后应用到桌面。".to_owned(),
            dirty: false,
            scale,
            active_margin: None,
        }
    }

    unsafe fn initialize(&mut self, _hwnd: Hwnd) {
        self.font_title = create_font(24, FW_SEMIBOLD, self.scale);
        self.font_body = create_font(14, FW_NORMAL, self.scale);
        self.font_small = create_font(11, FW_NORMAL, self.scale);
        if let Err(error) = self.refresh_preview() {
            self.status = format!("预览生成失败：{error:#}");
        }
    }

    fn refresh_preview(&mut self) -> Result<()> {
        let svg = crate::calendar::wallpaper_svg_with_layout(
            PREVIEW_WIDTH,
            PREVIEW_HEIGHT,
            self.year,
            &self.theme,
            self.today,
            self.layout,
        );
        let pixels = render_svg_pixels(&svg, PREVIEW_WIDTH, PREVIEW_HEIGHT)?;
        let bitmap = unsafe { bitmap_from_rgba(&pixels, PREVIEW_WIDTH, PREVIEW_HEIGHT)? };
        if self.preview_bitmap != 0 {
            unsafe {
                DeleteObject(self.preview_bitmap);
            }
        }
        self.preview_bitmap = bitmap;
        Ok(())
    }

    fn theme_name(&self) -> &str {
        match &self.theme {
            Theme::Dark => "深色",
            Theme::Light => "浅色",
            Theme::Custom(theme) => &theme.name,
        }
    }

    fn px(&self, value: i32) -> i32 {
        scale_px(value, self.scale)
    }

    fn ensure_custom(&mut self) -> &mut CustomTheme {
        if !matches!(self.theme, Theme::Custom(_)) {
            let mut custom = self
                .saved_custom_theme
                .clone()
                .unwrap_or_else(|| self.theme.exportable());
            if self.saved_custom_theme.is_none() {
                custom.name = "自定义主题".to_owned();
            }
            self.theme = Theme::custom(custom).expect("内置调色板必须有效");
        }
        match &mut self.theme {
            Theme::Custom(theme) => theme.as_mut(),
            Theme::Dark | Theme::Light => unreachable!(),
        }
    }

    fn remember_custom_theme(&mut self) {
        if let Some(theme) = self.theme.custom_theme() {
            self.saved_custom_theme = Some(theme.clone());
        }
    }

    fn activate(&mut self, hwnd: Hwnd, action: Action) {
        let result = match action {
            Action::Dark => self.select_theme(Theme::Dark),
            Action::Light => self.select_theme(Theme::Light),
            Action::Custom => {
                self.ensure_custom();
                self.remember_custom_theme();
                self.dirty = true;
                self.status = "已进入自定义主题。".to_owned();
                self.refresh_preview()
            }
            Action::AdjustMargin(_) => Ok(()),
            Action::Edit(field) => self.edit_color(hwnd, field),
            Action::Import => self.import_theme(hwnd),
            Action::Export => self.export_theme(hwnd),
            Action::Apply => self.apply(),
            Action::PreviousYear => self.change_year(-1),
            Action::NextYear => self.change_year(1),
            Action::Close => {
                unsafe {
                    DestroyWindow(hwnd);
                }
                Ok(())
            }
        };
        if let Err(error) = result {
            self.status = format!("操作失败：{error:#}");
        }
        unsafe {
            InvalidateRect(hwnd, null(), 0);
        }
    }

    fn select_theme(&mut self, theme: Theme) -> Result<()> {
        self.remember_custom_theme();
        self.theme = theme;
        self.dirty = true;
        self.status = "主题已更新，尚未应用到桌面。".to_owned();
        self.refresh_preview()
    }

    fn preview_color(&mut self, field: PaletteField, color: u32) -> Result<()> {
        *field.color_mut(&mut self.ensure_custom().palette) = hex_from_colorref(color);
        self.dirty = true;
        self.status = "配色预览中，尚未应用到桌面。".to_owned();
        self.refresh_preview()
    }

    fn set_margin_from_position(
        &mut self,
        field: MarginField,
        row: Rect,
        pointer_x: i32,
    ) -> Result<()> {
        let value = margin_value_from_position(field, row, pointer_x, self.scale);
        if field.value(self.layout) == value {
            return Ok(());
        }
        field.set(&mut self.layout, value);
        self.layout = self.layout.normalized();
        self.dirty = true;
        self.status = "版式边距预览中，尚未应用到桌面。".to_owned();
        self.refresh_preview()
    }

    fn restore_color_preview(&mut self, context: &ColorPreviewContext) -> Result<()> {
        self.theme = context.original_theme.clone();
        self.saved_custom_theme = context.original_saved_custom_theme.clone();
        self.status = context.original_status.clone();
        self.dirty = context.original_dirty;
        self.refresh_preview()
    }

    fn edit_color(&mut self, hwnd: Hwnd, field: PaletteField) -> Result<()> {
        let palette = self.theme.exportable().palette;
        let initial = colorref_from_hex(field.color(&palette));
        let state = self as *mut SettingsWindow;
        let mut context = ColorPreviewContext {
            state,
            owner: hwnd,
            field,
            original_theme: self.theme.clone(),
            original_saved_custom_theme: self.saved_custom_theme.clone(),
            original_status: self.status.clone(),
            original_dirty: self.dirty,
            dialog: 0,
            last_color: initial,
        };
        let custom_colors = self.custom_colors.as_mut_ptr();
        let selected = choose_color(hwnd, initial, custom_colors, &mut context);
        if let Some(color) = selected {
            if color == initial {
                self.restore_color_preview(&context)?;
            } else {
                self.preview_color(field, color)?;
                self.remember_custom_theme();
                self.status = "配色预览已更新，尚未应用到桌面。".to_owned();
            }
        } else {
            self.restore_color_preview(&context)?;
        }
        Ok(())
    }

    fn import_theme(&mut self, hwnd: Hwnd) -> Result<()> {
        let Some(path) = select_theme_file(hwnd, false) else {
            return Ok(());
        };
        self.theme = Theme::custom(load_custom_theme(&path)?).map_err(anyhow::Error::msg)?;
        self.remember_custom_theme();
        self.dirty = true;
        self.status = "已导入主题，尚未应用到桌面。".to_owned();
        self.refresh_preview()
    }

    fn export_theme(&mut self, hwnd: Hwnd) -> Result<()> {
        let Some(path) = select_theme_file(hwnd, true) else {
            return Ok(());
        };
        export_theme(&path, &self.theme, true)?;
        self.status = format!("已导出：{}", path.display());
        Ok(())
    }

    fn apply(&mut self) -> Result<()> {
        apply_selected_theme(&self.theme, self.layout)?;
        self.dirty = false;
        self.status = "已应用到桌面；后续自动更新会沿用此主题。".to_owned();
        Ok(())
    }

    fn change_year(&mut self, offset: i32) -> Result<()> {
        let year = (self.year + offset).clamp(1900, 2100);
        if year == self.year {
            return Ok(());
        }
        self.year = year;
        self.status = "仅切换预览年份；桌面始终生成当前年份。".to_owned();
        self.refresh_preview()
    }

    unsafe fn paint(&mut self, hwnd: Hwnd) {
        let mut paint = PaintStruct::default();
        let hdc = unsafe { BeginPaint(hwnd, &mut paint) };
        if hdc == 0 {
            return;
        }
        let mut client = Rect::default();
        unsafe {
            GetClientRect(hwnd, &mut client);
        }
        self.hits.clear();
        unsafe {
            paint_surface(hdc, self, client);
        }
        unsafe {
            EndPaint(hwnd, &paint);
        }
    }

    unsafe fn destroy_resources(&mut self) {
        for object in [
            self.preview_bitmap,
            self.font_title,
            self.font_body,
            self.font_small,
        ] {
            if object != 0 {
                unsafe {
                    DeleteObject(object);
                }
            }
        }
        self.preview_bitmap = 0;
    }
}

pub(crate) fn open_settings(
    year: i32,
    theme: Theme,
    today: NaiveDate,
    saved_custom_theme: Option<CustomTheme>,
    layout: LayoutSettings,
) -> Result<()> {
    if !(1900..=2100).contains(&year) {
        bail!("日历年份仅支持 1900-2100");
    }
    let scale = settings_window_scale(
        unsafe { GetDpiForSystem() },
        unsafe { GetSystemMetrics(0) },
        unsafe { GetSystemMetrics(1) },
    );
    let instance = unsafe { GetModuleHandleW(null()) };
    if instance == 0 {
        return Err(std::io::Error::last_os_error()).context("获取应用实例失败");
    }
    let class_name = wide(WINDOW_CLASS);
    let cursor = unsafe { LoadCursorW(0, 32512usize as *const u16) };
    let icon = load_app_icon();
    let window_class = WndClassExW {
        cb_size: size_of::<WndClassExW>() as u32,
        style: CS_HREDRAW | CS_VREDRAW,
        window_proc: Some(window_proc),
        cls_extra: 0,
        wnd_extra: 0,
        instance,
        icon,
        cursor,
        background: 0,
        menu_name: null(),
        class_name: class_name.as_ptr(),
        icon_small: icon,
    };
    if unsafe { RegisterClassExW(&window_class) } == 0 {
        return Err(std::io::Error::last_os_error()).context("注册原生设置窗口失败");
    }

    let title = wide("月序 · 桌面日历设置");
    let state = Box::new(SettingsWindow::new(
        year,
        theme,
        today,
        saved_custom_theme,
        layout,
        scale,
    ));
    let state_ptr = Box::into_raw(state);
    let hwnd = unsafe {
        CreateWindowExW(
            0,
            class_name.as_ptr(),
            title.as_ptr(),
            WS_OVERLAPPEDWINDOW,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            (SETTINGS_WINDOW_WIDTH as f32 * scale).round() as i32,
            (SETTINGS_WINDOW_HEIGHT as f32 * scale).round() as i32,
            0,
            0,
            instance,
            state_ptr.cast::<c_void>(),
        )
    };
    if hwnd == 0 {
        unsafe {
            drop(Box::from_raw(state_ptr));
        }
        return Err(std::io::Error::last_os_error()).context("创建原生设置窗口失败");
    }
    unsafe {
        ShowWindow(hwnd, SW_SHOW);
        UpdateWindow(hwnd);
    }

    let mut message: Msg = unsafe { zeroed() };
    loop {
        let result = unsafe { GetMessageW(&mut message, 0, 0, 0) };
        if result == 0 {
            break;
        }
        if result == -1 {
            return Err(std::io::Error::last_os_error()).context("读取窗口消息失败");
        }
        unsafe {
            TranslateMessage(&message);
            DispatchMessageW(&message);
        }
    }
    Ok(())
}

unsafe extern "system" fn window_proc(
    hwnd: Hwnd,
    message: u32,
    w_param: usize,
    l_param: isize,
) -> isize {
    match message {
        WM_NCCREATE => {
            let create = unsafe { &*(l_param as *const CreateStructW) };
            let state = create.create_params.cast::<SettingsWindow>();
            unsafe {
                SetWindowLongPtrW(hwnd, GWLP_USERDATA, state as isize);
                (*state).initialize(hwnd);
            }
            1
        }
        WM_PAINT => {
            if let Some(state) = unsafe { state_mut(hwnd) } {
                unsafe {
                    state.paint(hwnd);
                }
                0
            } else {
                unsafe { DefWindowProcW(hwnd, message, w_param, l_param) }
            }
        }
        WM_GETMINMAXINFO => {
            if let Some(state) = unsafe { state_mut(hwnd) } {
                let info = unsafe { &mut *(l_param as *mut MinMaxInfo) };
                info.min_track_size.x = state.px(1120);
                info.min_track_size.y = state.px(960);
                0
            } else {
                unsafe { DefWindowProcW(hwnd, message, w_param, l_param) }
            }
        }
        WM_ERASEBKGND => 1,
        WM_LBUTTONDOWN => {
            if let Some(state) = unsafe { state_mut(hwnd) } {
                let (x, y) = mouse_position(l_param);
                if let Some((field, row)) = margin_target_at(&state.hits, x, y) {
                    state.active_margin = Some(field);
                    if let Err(error) = state.set_margin_from_position(field, row, x) {
                        state.status = format!("预览生成失败：{error:#}");
                    }
                    unsafe {
                        SetCapture(hwnd);
                        InvalidateRect(hwnd, null(), 0);
                    }
                }
            }
            0
        }
        WM_MOUSEMOVE => {
            if let Some(state) = unsafe { state_mut(hwnd) }
                && let Some(field) = state.active_margin
            {
                let (x, _) = mouse_position(l_param);
                if let Some(row) = margin_row(&state.hits, field) {
                    if let Err(error) = state.set_margin_from_position(field, row, x) {
                        state.status = format!("预览生成失败：{error:#}");
                    }
                    unsafe {
                        InvalidateRect(hwnd, null(), 0);
                    }
                }
            }
            0
        }
        WM_LBUTTONUP => {
            if let Some(state) = unsafe { state_mut(hwnd) } {
                let (x, y) = mouse_position(l_param);
                if let Some(field) = state.active_margin.take() {
                    if let Some(row) = margin_row(&state.hits, field)
                        && let Err(error) = state.set_margin_from_position(field, row, x)
                    {
                        state.status = format!("预览生成失败：{error:#}");
                    }
                    unsafe {
                        ReleaseCapture();
                        InvalidateRect(hwnd, null(), 0);
                    }
                } else if let Some(action) = state
                    .hits
                    .iter()
                    .rev()
                    .find(|target| target.rect.contains(x, y))
                    .map(|target| target.action)
                {
                    state.activate(hwnd, action);
                }
            }
            0
        }
        WM_CLOSE => {
            unsafe {
                DestroyWindow(hwnd);
            }
            0
        }
        WM_DESTROY => {
            unsafe {
                PostQuitMessage(0);
            }
            0
        }
        WM_NCDESTROY => {
            let pointer = unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) } as *mut SettingsWindow;
            if !pointer.is_null() {
                unsafe {
                    (*pointer).destroy_resources();
                    drop(Box::from_raw(pointer));
                    SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
                }
            }
            unsafe { DefWindowProcW(hwnd, message, w_param, l_param) }
        }
        _ => unsafe { DefWindowProcW(hwnd, message, w_param, l_param) },
    }
}

unsafe fn state_mut(hwnd: Hwnd) -> Option<&'static mut SettingsWindow> {
    let pointer = unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) } as *mut SettingsWindow;
    if pointer.is_null() {
        None
    } else {
        Some(unsafe { &mut *pointer })
    }
}

fn mouse_position(l_param: isize) -> (i32, i32) {
    (
        (l_param as u32 & 0xFFFF) as i16 as i32,
        ((l_param as u32 >> 16) & 0xFFFF) as i16 as i32,
    )
}

fn margin_target_at(hits: &[HitTarget], x: i32, y: i32) -> Option<(MarginField, Rect)> {
    hits.iter().rev().find_map(|target| match target.action {
        Action::AdjustMargin(field) if target.rect.contains(x, y) => Some((field, target.rect)),
        _ => None,
    })
}

fn margin_row(hits: &[HitTarget], field: MarginField) -> Option<Rect> {
    hits.iter().rev().find_map(|target| match target.action {
        Action::AdjustMargin(candidate) if candidate == field => Some(target.rect),
        _ => None,
    })
}

unsafe fn paint_surface(hdc: Hdc, state: &mut SettingsWindow, client: Rect) {
    let scale = state.scale;
    let s = |value: i32| scale_px(value, scale);
    let margin = s(24);
    let side_width = s(318).min((client.width() - margin * 3).max(s(260)));
    let side = Rect::new(margin, margin, margin + side_width, client.bottom - margin);
    let preview_left = side.right + margin;
    let preview_width = (client.right - preview_left - margin).max(s(320));
    let available_height = (client.height() - s(150)).max(s(260));
    let preview_height = ((preview_width * 9) / 16).min(available_height);
    let preview = Rect::new(
        preview_left,
        s(78),
        preview_left + (preview_height * 16) / 9,
        s(78) + preview_height,
    );

    unsafe {
        fill_rect(hdc, client, rgb(15, 20, 18));
        fill_rect(hdc, side, rgb(25, 33, 29));
        frame_rect(hdc, side, rgb(66, 79, 70));
    }

    unsafe {
        draw_text(
            hdc,
            state.font_title,
            "月序",
            Rect::new(
                side.left + s(22),
                side.top + s(20),
                side.right - s(22),
                side.top + s(54),
            ),
            rgb(241, 234, 223),
            DT_LEFT | DT_VCENTER | DT_SINGLELINE,
        );
        draw_text(
            hdc,
            state.font_small,
            "LUNAR ALMANAC",
            Rect::new(
                side.left + s(23),
                side.top + s(56),
                side.right - s(22),
                side.top + s(76),
            ),
            rgb(151, 164, 153),
            DT_LEFT | DT_VCENTER | DT_SINGLELINE,
        );
        draw_text(
            hdc,
            state.font_small,
            "主题",
            Rect::new(
                side.left + s(22),
                side.top + s(102),
                side.right - s(22),
                side.top + s(124),
            ),
            rgb(157, 171, 160),
            DT_LEFT | DT_VCENTER | DT_SINGLELINE,
        );
    }

    let theme_y = side.top + s(132);
    let button_gap = s(7);
    let theme_width = (side.width() - s(44) - button_gap * 2) / 3;
    let selections = [
        ("深色", Action::Dark, matches!(state.theme, Theme::Dark)),
        ("浅色", Action::Light, matches!(state.theme, Theme::Light)),
        (
            "自定义",
            Action::Custom,
            matches!(state.theme, Theme::Custom(_)),
        ),
    ];
    for (index, (label, action, selected)) in selections.iter().enumerate() {
        let left = side.left + s(22) + index as i32 * (theme_width + button_gap);
        let rect = Rect::new(left, theme_y, left + theme_width, theme_y + s(34));
        unsafe {
            draw_button(hdc, state.font_small, label, rect, *selected, false);
        }
        state.hits.push(HitTarget {
            rect,
            action: *action,
        });
    }

    let mut layout_y = theme_y + s(52);
    unsafe {
        draw_text(
            hdc,
            state.font_small,
            "版式边距",
            Rect::new(
                side.left + s(22),
                layout_y,
                side.right - s(22),
                layout_y + s(22),
            ),
            rgb(157, 171, 160),
            DT_LEFT | DT_VCENTER | DT_SINGLELINE,
        );
    }
    layout_y += s(28);
    for (field, label) in MarginField::ALL {
        let rect = Rect::new(
            side.left + s(22),
            layout_y,
            side.right - s(22),
            layout_y + s(34),
        );
        unsafe {
            draw_margin_row(
                hdc,
                state.font_small,
                label,
                rect,
                field,
                field.value(state.layout),
                scale,
            );
        }
        state.hits.push(HitTarget {
            rect,
            action: Action::AdjustMargin(field),
        });
        layout_y += s(40);
    }

    unsafe {
        draw_text(
            hdc,
            state.font_small,
            "自定义配色",
            Rect::new(
                side.left + s(22),
                layout_y + s(6),
                side.right - s(22),
                layout_y + s(28),
            ),
            rgb(157, 171, 160),
            DT_LEFT | DT_VCENTER | DT_SINGLELINE,
        );
    }
    let palette = state.theme.exportable().palette;
    let mut color_y = layout_y + s(34);
    for (field, label) in PaletteField::ALL {
        let rect = Rect::new(
            side.left + s(22),
            color_y,
            side.right - s(22),
            color_y + s(27),
        );
        unsafe {
            draw_color_row(
                hdc,
                state.font_small,
                label,
                rect,
                colorref_from_hex(field.color(&palette)),
                scale,
            );
        }
        state.hits.push(HitTarget {
            rect,
            action: Action::Edit(field),
        });
        color_y += s(30);
    }

    let command_y = color_y + s(11);
    let command_width = (side.width() - s(51)) / 2;
    let import_rect = Rect::new(
        side.left + s(22),
        command_y,
        side.left + s(22) + command_width,
        command_y + s(34),
    );
    let export_rect = Rect::new(
        import_rect.right + s(7),
        command_y,
        side.right - s(22),
        command_y + s(34),
    );
    unsafe {
        draw_button(hdc, state.font_small, "导入主题", import_rect, false, false);
        draw_button(hdc, state.font_small, "导出主题", export_rect, false, false);
    }
    state.hits.extend([
        HitTarget {
            rect: import_rect,
            action: Action::Import,
        },
        HitTarget {
            rect: export_rect,
            action: Action::Export,
        },
    ]);

    let apply_rect = Rect::new(
        side.left + s(22),
        command_y + s(50),
        side.right - s(22),
        command_y + s(92),
    );
    let close_rect = Rect::new(
        side.left + s(22),
        command_y + s(100),
        side.right - s(22),
        command_y + s(134),
    );
    unsafe {
        draw_button(hdc, state.font_body, "应用到桌面", apply_rect, true, true);
        draw_button(hdc, state.font_small, "关闭", close_rect, false, false);
    }
    state.hits.extend([
        HitTarget {
            rect: apply_rect,
            action: Action::Apply,
        },
        HitTarget {
            rect: close_rect,
            action: Action::Close,
        },
    ]);

    unsafe {
        draw_text(
            hdc,
            state.font_title,
            &format!("{} 年全年日历", state.year),
            Rect::new(preview.left, s(22), preview.right - s(120), s(56)),
            rgb(241, 234, 223),
            DT_LEFT | DT_VCENTER | DT_SINGLELINE,
        );
        draw_text(
            hdc,
            state.font_small,
            state.theme_name(),
            Rect::new(preview.left, s(53), preview.right - s(120), s(72)),
            rgb(151, 164, 153),
            DT_LEFT | DT_VCENTER | DT_SINGLELINE | DT_END_ELLIPSIS,
        );
    }
    let previous = Rect::new(preview.right - s(82), s(27), preview.right - s(47), s(61));
    let next = Rect::new(preview.right - s(40), s(27), preview.right - s(5), s(61));
    unsafe {
        draw_button(hdc, state.font_body, "‹", previous, false, false);
        draw_button(hdc, state.font_body, "›", next, false, false);
        fill_rect(hdc, preview, rgb(16, 23, 20));
    }
    state.hits.extend([
        HitTarget {
            rect: previous,
            action: Action::PreviousYear,
        },
        HitTarget {
            rect: next,
            action: Action::NextYear,
        },
    ]);
    if state.preview_bitmap != 0 {
        unsafe {
            draw_bitmap(
                hdc,
                state.preview_bitmap,
                state.preview_width,
                state.preview_height,
                preview,
            );
        }
    }
    unsafe {
        frame_rect(hdc, preview, rgb(72, 87, 78));
        draw_text(
            hdc,
            state.font_body,
            &state.status,
            Rect::new(
                preview.left,
                preview.bottom + s(18),
                client.right - margin,
                preview.bottom + s(44),
            ),
            if state.dirty {
                rgb(223, 137, 102)
            } else {
                rgb(157, 171, 160)
            },
            DT_LEFT | DT_VCENTER | DT_SINGLELINE | DT_END_ELLIPSIS,
        );
    }
}

unsafe fn draw_bitmap(hdc: Hdc, bitmap: Hbitmap, width: i32, height: i32, target: Rect) {
    let memory = unsafe { CreateCompatibleDC(hdc) };
    if memory == 0 {
        return;
    }
    let old = unsafe { SelectObject(memory, bitmap) };
    unsafe {
        SetStretchBltMode(hdc, HALFTONE);
        StretchBlt(
            hdc,
            target.left,
            target.top,
            target.width(),
            target.height(),
            memory,
            0,
            0,
            width,
            height,
            SRCCOPY,
        );
        SelectObject(memory, old);
        DeleteDC(memory);
    }
}

unsafe fn draw_button(
    hdc: Hdc,
    font: Hfont,
    label: &str,
    rect: Rect,
    selected: bool,
    primary: bool,
) {
    let fill = if primary {
        rgb(223, 107, 84)
    } else if selected {
        rgb(56, 73, 62)
    } else {
        rgb(31, 42, 36)
    };
    let border = if primary {
        rgb(223, 107, 84)
    } else {
        rgb(83, 101, 89)
    };
    let text_color = if primary {
        rgb(255, 248, 238)
    } else {
        rgb(233, 226, 212)
    };
    unsafe {
        fill_rect(hdc, rect, fill);
        frame_rect(hdc, rect, border);
        draw_text(
            hdc,
            font,
            label,
            rect,
            text_color,
            DT_CENTER | DT_VCENTER | DT_SINGLELINE,
        );
    }
}

unsafe fn draw_color_row(hdc: Hdc, font: Hfont, label: &str, rect: Rect, color: u32, scale: f32) {
    let s = |value: i32| scale_px(value, scale);
    unsafe {
        fill_rect(hdc, rect, rgb(29, 39, 34));
        frame_rect(hdc, rect, rgb(63, 78, 69));
        draw_text(
            hdc,
            font,
            label,
            Rect::new(rect.left + s(10), rect.top, rect.right - s(44), rect.bottom),
            rgb(224, 230, 222),
            DT_LEFT | DT_VCENTER | DT_SINGLELINE,
        );
        let chip = Rect::new(
            rect.right - s(33),
            rect.top + s(5),
            rect.right - s(8),
            rect.bottom - s(5),
        );
        fill_rect(hdc, chip, color);
        frame_rect(hdc, chip, rgb(224, 230, 222));
    }
}

unsafe fn draw_margin_row(
    hdc: Hdc,
    font: Hfont,
    label: &str,
    rect: Rect,
    field: MarginField,
    value: u8,
    scale: f32,
) {
    let s = |value: i32| scale_px(value, scale);
    let track = margin_track_rect(rect, scale);
    let (minimum, maximum) = field.range();
    let clamped = value.clamp(minimum, maximum);
    let span = i32::from(maximum - minimum).max(1);
    let offset = i32::from(clamped - minimum);
    let handle_x = track.left + (track.width().max(1) - 1) * offset / span;
    let handle = Rect::new(
        handle_x - s(4),
        rect.top + s(9),
        handle_x + s(5),
        rect.bottom - s(8),
    );
    unsafe {
        fill_rect(hdc, rect, rgb(29, 39, 34));
        frame_rect(hdc, rect, rgb(63, 78, 69));
        draw_text(
            hdc,
            font,
            label,
            Rect::new(rect.left + s(10), rect.top, rect.left + s(100), rect.bottom),
            rgb(224, 230, 222),
            DT_LEFT | DT_VCENTER | DT_SINGLELINE,
        );
        draw_text(
            hdc,
            font,
            &format!("{clamped}%"),
            Rect::new(rect.left + s(82), rect.top, track.left - s(6), rect.bottom),
            rgb(157, 171, 160),
            DT_CENTER | DT_VCENTER | DT_SINGLELINE,
        );
        fill_rect(hdc, track, rgb(83, 101, 89));
        fill_rect(
            hdc,
            Rect::new(track.left, track.top, handle_x + 1, track.bottom),
            rgb(223, 107, 84),
        );
        fill_rect(hdc, handle, rgb(241, 234, 223));
        frame_rect(hdc, handle, rgb(223, 107, 84));
    }
}

fn margin_track_rect(rect: Rect, scale: f32) -> Rect {
    let s = |value: i32| scale_px(value, scale);
    let left = rect.left + s(112);
    let right = (rect.right - s(11)).max(left + 1);
    let center = (rect.top + rect.bottom) / 2;
    Rect::new(left, center - s(2).max(1), right, center + s(3).max(2))
}

fn margin_value_from_position(field: MarginField, row: Rect, pointer_x: i32, scale: f32) -> u8 {
    let track = margin_track_rect(row, scale);
    let span = (track.width() - 1).max(1);
    let offset = (pointer_x - track.left).clamp(0, span);
    let (minimum, maximum) = field.range();
    let range = i32::from(maximum - minimum);
    let value = i32::from(minimum) + (range * offset + span / 2) / span;
    value as u8
}

unsafe fn fill_rect(hdc: Hdc, rect: Rect, color: u32) {
    let brush = unsafe { CreateSolidBrush(color) };
    if brush != 0 {
        unsafe {
            FillRect(hdc, &rect, brush);
            DeleteObject(brush);
        }
    }
}

unsafe fn frame_rect(hdc: Hdc, rect: Rect, color: u32) {
    let brush = unsafe { CreateSolidBrush(color) };
    if brush != 0 {
        unsafe {
            FrameRect(hdc, &rect, brush);
            DeleteObject(brush);
        }
    }
}

unsafe fn draw_text(hdc: Hdc, font: Hfont, text: &str, rect: Rect, color: u32, flags: u32) {
    let value = wide(text);
    let old = unsafe { SelectObject(hdc, font) };
    unsafe {
        SetBkMode(hdc, TRANSPARENT);
        SetTextColor(hdc, color);
        DrawTextW(hdc, value.as_ptr(), (value.len() - 1) as i32, &rect, flags);
        SelectObject(hdc, old);
    }
}

unsafe fn bitmap_from_rgba(data: &[u8], width: u32, height: u32) -> Result<Hbitmap> {
    let mut info = BitmapInfo {
        header: BitmapInfoHeader {
            size: size_of::<BitmapInfoHeader>() as u32,
            width: width as i32,
            height: -(height as i32),
            planes: 1,
            bit_count: 32,
            compression: BI_RGB,
            size_image: width * height * 4,
            x_pels_per_meter: 0,
            y_pels_per_meter: 0,
            clr_used: 0,
            clr_important: 0,
        },
        colors: [RgbQuad::default()],
    };
    let mut bits = null_mut::<c_void>();
    let bitmap = unsafe { CreateDIBSection(0, &mut info, DIB_RGB_COLORS, &mut bits, 0, 0) };
    if bitmap == 0 || bits.is_null() {
        return Err(std::io::Error::last_os_error()).context("创建预览位图失败");
    }
    let destination = unsafe { slice::from_raw_parts_mut(bits.cast::<u8>(), data.len()) };
    for (source, target) in data.chunks_exact(4).zip(destination.chunks_exact_mut(4)) {
        target[0] = source[2];
        target[1] = source[1];
        target[2] = source[0];
        target[3] = 255;
    }
    Ok(bitmap)
}

fn select_theme_file(hwnd: Hwnd, save: bool) -> Option<PathBuf> {
    let filter = wide("月序主题 (*.json)\0*.json\0所有文件 (*.*)\0*.*\0\0");
    let title = wide(if save {
        "导出月序主题"
    } else {
        "导入月序主题"
    });
    let extension = wide("json");
    let mut file = [0u16; 32_768];
    let mut dialog = OpenFileNameW {
        size: size_of::<OpenFileNameW>() as u32,
        owner: hwnd,
        instance: 0,
        filter: filter.as_ptr(),
        custom_filter: null_mut(),
        max_custom_filter: 0,
        filter_index: 1,
        file: file.as_mut_ptr(),
        max_file: file.len() as u32,
        file_title: null_mut(),
        max_file_title: 0,
        initial_dir: null(),
        title: title.as_ptr(),
        flags: OFN_HIDEREADONLY
            | OFN_NOCHANGEDIR
            | OFN_PATHMUSTEXIST
            | if save {
                OFN_OVERWRITEPROMPT
            } else {
                OFN_FILEMUSTEXIST
            },
        file_offset: 0,
        file_extension: 0,
        default_ext: extension.as_ptr(),
        custom_data: 0,
        hook: null(),
        template_name: null(),
        reserved: null_mut(),
        reserved_value: 0,
        flags_ex: 0,
    };
    let selected = unsafe {
        if save {
            GetSaveFileNameW(&mut dialog)
        } else {
            GetOpenFileNameW(&mut dialog)
        }
    };
    if selected == 0 {
        return None;
    }
    let length = file.iter().position(|value| *value == 0)?;
    Some(PathBuf::from(OsString::from_wide(&file[..length])))
}

fn choose_color(
    hwnd: Hwnd,
    initial: u32,
    custom_colors: *mut u32,
    context: &mut ColorPreviewContext,
) -> Option<u32> {
    let mut dialog = ChooseColorW {
        size: size_of::<ChooseColorW>() as u32,
        owner: hwnd,
        instance: 0,
        result: initial,
        custom_colors,
        flags: CC_RGBINIT | CC_FULLOPEN | CC_ENABLEHOOK,
        custom_data: (context as *mut ColorPreviewContext) as isize,
        hook: Some(choose_color_hook),
        template_name: null(),
    };
    let accepted = unsafe { ChooseColorW(&mut dialog) } != 0;
    if context.dialog != 0 {
        unsafe {
            KillTimer(context.dialog, COLOR_PREVIEW_TIMER_ID);
        }
    }
    accepted.then_some(dialog.result)
}

unsafe extern "system" fn choose_color_hook(
    hwnd: Hwnd,
    message: u32,
    w_param: usize,
    l_param: isize,
) -> isize {
    match message {
        WM_INITDIALOG => {
            let dialog = unsafe { &*(l_param as *const ChooseColorW) };
            let context = dialog.custom_data as *mut ColorPreviewContext;
            if !context.is_null() {
                unsafe {
                    SetWindowLongPtrW(hwnd, GWLP_USERDATA, context as isize);
                    (*context).dialog = hwnd;
                }
            }
        }
        WM_COMMAND if color_component_changed(w_param) => unsafe {
            SetTimer(hwnd, COLOR_PREVIEW_TIMER_ID, COLOR_PREVIEW_DELAY_MS, null());
        },
        WM_TIMER if w_param == COLOR_PREVIEW_TIMER_ID => unsafe {
            KillTimer(hwnd, COLOR_PREVIEW_TIMER_ID);
            refresh_color_preview_from_dialog(hwnd);
        },
        WM_DESTROY | WM_NCDESTROY => unsafe {
            KillTimer(hwnd, COLOR_PREVIEW_TIMER_ID);
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
        },
        _ => {}
    }
    0
}

fn color_component_changed(w_param: usize) -> bool {
    let control = (w_param & 0xFFFF) as i32;
    let notification = (w_param >> 16) & 0xFFFF;
    notification == EN_CHANGE && matches!(control, COLOR_RED | COLOR_GREEN | COLOR_BLUE)
}

unsafe fn refresh_color_preview_from_dialog(hwnd: Hwnd) {
    let context = unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) } as *mut ColorPreviewContext;
    if context.is_null() {
        return;
    }
    let Some(color) = (unsafe { color_from_dialog(hwnd) }) else {
        return;
    };
    let context = unsafe { &mut *context };
    if color == context.last_color || context.state.is_null() {
        return;
    }
    context.last_color = color;
    let state = unsafe { &mut *context.state };
    if let Err(error) = state.preview_color(context.field, color) {
        state.status = format!("预览生成失败：{error:#}");
    }
    unsafe {
        InvalidateRect(context.owner, null(), 0);
    }
}

unsafe fn color_from_dialog(hwnd: Hwnd) -> Option<u32> {
    let mut converted = 0;
    let red = unsafe { GetDlgItemInt(hwnd, COLOR_RED, &mut converted, 0) };
    if converted == 0 || red > u32::from(u8::MAX) {
        return None;
    }
    let green = unsafe { GetDlgItemInt(hwnd, COLOR_GREEN, &mut converted, 0) };
    if converted == 0 || green > u32::from(u8::MAX) {
        return None;
    }
    let blue = unsafe { GetDlgItemInt(hwnd, COLOR_BLUE, &mut converted, 0) };
    if converted == 0 || blue > u32::from(u8::MAX) {
        return None;
    }
    Some(rgb(red as u8, green as u8, blue as u8))
}

fn create_font(size: i32, weight: i32, scale: f32) -> Hfont {
    let face = wide("Microsoft YaHei UI");
    let height = -scale_px(size, scale).max(1);
    unsafe {
        CreateFontW(
            height,
            0,
            0,
            0,
            weight,
            0,
            0,
            0,
            134,
            0,
            0,
            5,
            0,
            face.as_ptr(),
        )
    }
}

fn scale_px(value: i32, scale: f32) -> i32 {
    (value as f32 * scale).round() as i32
}

fn settings_window_scale(dpi: u32, screen_width: i32, screen_height: i32) -> f32 {
    let available_width = (screen_width - 96).max(1) as f32;
    let available_height = (screen_height - 120).max(1) as f32;
    (dpi.max(96) as f32 / 96.0)
        .min(available_width / SETTINGS_WINDOW_WIDTH as f32)
        .min(available_height / SETTINGS_WINDOW_HEIGHT as f32)
        .max(0.35)
}

fn load_app_icon() -> Handle {
    let mut candidates = Vec::new();
    if let Ok(executable) = env::current_exe()
        && let Some(directory) = executable.parent()
    {
        candidates.push(directory.join("YueXu.ico"));
    }
    candidates.push(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets/YueXu.ico"));
    for candidate in candidates {
        if candidate.is_file() {
            let path = wide(&candidate.to_string_lossy());
            let icon = unsafe { LoadImageW(0, path.as_ptr(), IMAGE_ICON, 0, 0, LR_LOADFROMFILE) };
            if icon != 0 {
                return icon;
            }
        }
    }
    0
}

fn wide(value: &str) -> Vec<u16> {
    OsStr::new(value).encode_wide().chain(Some(0)).collect()
}

fn rgb(red: u8, green: u8, blue: u8) -> u32 {
    u32::from(red) | (u32::from(green) << 8) | (u32::from(blue) << 16)
}

fn colorref_from_hex(value: &str) -> u32 {
    let red = u8::from_str_radix(&value[1..3], 16).unwrap_or(0);
    let green = u8::from_str_radix(&value[3..5], 16).unwrap_or(0);
    let blue = u8::from_str_radix(&value[5..7], 16).unwrap_or(0);
    rgb(red, green, blue)
}

fn hex_from_colorref(value: u32) -> String {
    format!(
        "#{:02X}{:02X}{:02X}",
        value & 0xFF,
        (value >> 8) & 0xFF,
        (value >> 16) & 0xFF
    )
}

#[link(name = "user32")]
unsafe extern "system" {
    fn RegisterClassExW(window_class: *const WndClassExW) -> u16;
    fn CreateWindowExW(
        ex_style: u32,
        class_name: *const u16,
        window_name: *const u16,
        style: u32,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        parent: Hwnd,
        menu: Handle,
        instance: Hinstance,
        parameter: *mut c_void,
    ) -> Hwnd;
    fn DefWindowProcW(hwnd: Hwnd, message: u32, w_param: usize, l_param: isize) -> isize;
    fn ShowWindow(hwnd: Hwnd, command: i32) -> i32;
    fn UpdateWindow(hwnd: Hwnd) -> i32;
    fn DestroyWindow(hwnd: Hwnd) -> i32;
    fn SetCapture(hwnd: Hwnd) -> Hwnd;
    fn ReleaseCapture() -> i32;
    fn GetMessageW(message: *mut Msg, hwnd: Hwnd, minimum: u32, maximum: u32) -> i32;
    fn TranslateMessage(message: *const Msg) -> i32;
    fn DispatchMessageW(message: *const Msg) -> isize;
    fn PostQuitMessage(code: i32);
    fn BeginPaint(hwnd: Hwnd, paint: *mut PaintStruct) -> Hdc;
    fn EndPaint(hwnd: Hwnd, paint: *const PaintStruct) -> i32;
    fn GetClientRect(hwnd: Hwnd, rect: *mut Rect) -> i32;
    fn FillRect(hdc: Hdc, rect: *const Rect, brush: Handle) -> i32;
    fn FrameRect(hdc: Hdc, rect: *const Rect, brush: Handle) -> i32;
    fn InvalidateRect(hwnd: Hwnd, rect: *const Rect, erase: i32) -> i32;
    fn SetWindowLongPtrW(hwnd: Hwnd, index: i32, value: isize) -> isize;
    fn GetWindowLongPtrW(hwnd: Hwnd, index: i32) -> isize;
    fn GetDlgItemInt(hwnd: Hwnd, item: i32, translated: *mut i32, signed: i32) -> u32;
    fn SetTimer(hwnd: Hwnd, identifier: usize, elapsed: u32, callback: *const c_void) -> usize;
    fn KillTimer(hwnd: Hwnd, identifier: usize) -> i32;
    fn LoadCursorW(instance: Hinstance, cursor_name: *const u16) -> Handle;
    fn LoadImageW(
        instance: Hinstance,
        name: *const u16,
        image_type: u32,
        desired_width: i32,
        desired_height: i32,
        flags: u32,
    ) -> Handle;
    fn GetDpiForSystem() -> u32;
    fn GetSystemMetrics(index: i32) -> i32;
    fn DrawTextW(hdc: Hdc, text: *const u16, count: i32, rect: *const Rect, format: u32) -> i32;
}

#[link(name = "gdi32")]
unsafe extern "system" {
    fn CreateSolidBrush(color: u32) -> Handle;
    fn DeleteObject(object: Handle) -> i32;
    fn CreateCompatibleDC(hdc: Hdc) -> Hdc;
    fn DeleteDC(hdc: Hdc) -> i32;
    fn SelectObject(hdc: Hdc, object: Handle) -> Handle;
    fn StretchBlt(
        destination: Hdc,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        source: Hdc,
        source_x: i32,
        source_y: i32,
        source_width: i32,
        source_height: i32,
        raster_operation: u32,
    ) -> i32;
    fn SetStretchBltMode(hdc: Hdc, mode: i32) -> i32;
    fn SetBkMode(hdc: Hdc, mode: i32) -> i32;
    fn SetTextColor(hdc: Hdc, color: u32) -> u32;
    fn CreateFontW(
        height: i32,
        width: i32,
        escapement: i32,
        orientation: i32,
        weight: i32,
        italic: u32,
        underline: u32,
        strike_out: u32,
        char_set: u32,
        out_precision: u32,
        clip_precision: u32,
        quality: u32,
        pitch_and_family: u32,
        face_name: *const u16,
    ) -> Hfont;
    fn CreateDIBSection(
        hdc: Hdc,
        info: *mut BitmapInfo,
        usage: u32,
        bits: *mut *mut c_void,
        section: Handle,
        offset: u32,
    ) -> Hbitmap;
}

#[link(name = "kernel32")]
unsafe extern "system" {
    fn GetModuleHandleW(module_name: *const u16) -> Hinstance;
}

#[link(name = "comdlg32")]
unsafe extern "system" {
    fn ChooseColorW(dialog: *mut ChooseColorW) -> i32;
    fn GetOpenFileNameW(dialog: *mut OpenFileNameW) -> i32;
    fn GetSaveFileNameW(dialog: *mut OpenFileNameW) -> i32;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preserves_rgb_when_converting_between_windows_and_hex_colors() {
        let source = "#12ABCD";
        assert_eq!(hex_from_colorref(colorref_from_hex(source)), source);
    }

    #[test]
    fn restores_saved_custom_theme_after_switching_back_from_builtin_theme() {
        let saved = CustomTheme {
            name: "雾青".to_owned(),
            palette: Theme::Light.exportable().palette,
        };
        let mut state = SettingsWindow::new(
            2026,
            Theme::Dark,
            NaiveDate::from_ymd_opt(2026, 8, 5).unwrap(),
            Some(saved),
            LayoutSettings::default(),
            1.0,
        );

        state.ensure_custom();

        assert_eq!(state.theme_name(), "雾青");
    }

    #[test]
    fn keeps_the_initial_window_inside_a_small_screen() {
        let scale = settings_window_scale(96, 800, 600);

        assert!(SETTINGS_WINDOW_WIDTH as f32 * scale <= 704.0);
        assert!(SETTINGS_WINDOW_HEIGHT as f32 * scale <= 480.0);
    }

    #[test]
    fn maps_margin_pointer_positions_to_the_slider_range() {
        let row = Rect::new(0, 0, 280, 34);
        let track = margin_track_rect(row, 1.0);

        assert_eq!(
            margin_value_from_position(MarginField::Left, row, track.left, 1.0),
            LayoutSettings::MIN_LEFT_MARGIN
        );
        assert_eq!(
            margin_value_from_position(MarginField::Left, row, track.right, 1.0),
            LayoutSettings::MAX_LEFT_MARGIN
        );
    }

    #[test]
    fn color_preview_stays_pending_until_apply() {
        let mut state = SettingsWindow::new(
            2026,
            Theme::Dark,
            NaiveDate::from_ymd_opt(2026, 8, 5).unwrap(),
            None,
            LayoutSettings::default(),
            1.0,
        );

        state
            .preview_color(PaletteField::Accent, rgb(18, 171, 205))
            .unwrap();

        assert!(matches!(&state.theme, Theme::Custom(_)));
        assert!(state.dirty);
        assert_eq!(state.status, "配色预览中，尚未应用到桌面。");
        unsafe {
            state.destroy_resources();
        }
    }
}
