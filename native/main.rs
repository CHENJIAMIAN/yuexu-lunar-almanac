#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

mod calendar;
mod ui;

use std::{
    env,
    ffi::{OsStr, c_void},
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail};
use calendar::{CustomTheme, Theme};
use chrono::{Datelike, Local, NaiveDate};
use serde::{Deserialize, Serialize};

const APP_NAME: &str = "月序";
const DEFAULT_WIDTH: u32 = 3840;
const DEFAULT_HEIGHT: u32 = 2160;
const WALLPAPER_NAME: &str = "lunar-wallpaper.png";
const VERSION: &str = match option_env!("YUEXU_VERSION") {
    Some(version) => version,
    None => env!("CARGO_PKG_VERSION"),
};

#[derive(Debug)]
struct CliOptions {
    update: bool,
    preview: bool,
    quiet: bool,
    show_version: bool,
    width: u32,
    height: u32,
    year: i32,
    requested_theme: Option<String>,
    theme_file: Option<PathBuf>,
    export_theme: Option<PathBuf>,
    today: NaiveDate,
    output: Option<PathBuf>,
    set_wallpaper: bool,
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct Settings {
    theme: String,
    #[serde(
        rename = "customTheme",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    custom_theme: Option<CustomTheme>,
}

fn main() {
    let quiet = env::args().any(|argument| argument == "--quiet");
    if let Err(error) = run() {
        report_error(&error, quiet);
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let options = parse_cli()?;
    if options.show_version {
        println!("{APP_NAME} {VERSION}");
        return Ok(());
    }

    let theme = resolve_theme(
        options.requested_theme.as_deref(),
        options.theme_file.as_deref(),
    )?;
    if let Some(path) = options.export_theme.as_deref() {
        export_theme(path, &theme, options.quiet)?;
        return Ok(());
    }
    if options.preview {
        return ui::open_settings(
            options.year,
            theme,
            options.today,
            load_settings()?.custom_theme,
        );
    }

    if options.update {
        return render_wallpaper(RenderOptions {
            width: options.width,
            height: options.height,
            year: options.year,
            theme,
            today: options.today,
            output: options.output,
            set_wallpaper: options.set_wallpaper,
            quiet: options.quiet,
        });
    }

    usage();
    Ok(())
}

fn parse_cli() -> Result<CliOptions> {
    let now = Local::now().date_naive();
    let (desktop_width, desktop_height) = desktop_resolution();
    let mut options = CliOptions {
        update: false,
        preview: false,
        quiet: false,
        show_version: false,
        width: desktop_width,
        height: desktop_height,
        year: now.year(),
        requested_theme: None,
        theme_file: None,
        export_theme: None,
        today: now,
        output: None,
        set_wallpaper: true,
    };
    let args = env::args().skip(1).collect::<Vec<_>>();
    if args.is_empty() {
        options.update = true;
        return Ok(options);
    }

    let mut index = 0;
    while index < args.len() {
        let argument = &args[index];
        if argument.eq_ignore_ascii_case("--update") {
            options.update = true;
        } else if argument.eq_ignore_ascii_case("--preview") {
            options.preview = true;
        } else if argument.eq_ignore_ascii_case("--quiet") {
            options.quiet = true;
        } else if argument.eq_ignore_ascii_case("--version") {
            options.show_version = true;
        } else if let Some(value) = argument.strip_prefix("--width=") {
            options.width = parse_u32("宽度", value)?;
        } else if let Some(value) = argument.strip_prefix("--height=") {
            options.height = parse_u32("高度", value)?;
        } else if let Some(value) = argument.strip_prefix("--year=") {
            options.year = parse_i32("年份", value)?;
        } else if let Some(value) = argument.strip_prefix("--theme=") {
            options.requested_theme = Some(value.to_owned());
        } else if let Some(value) = argument.strip_prefix("--theme-file=") {
            options.theme_file = Some(PathBuf::from(value));
        } else if let Some(value) = argument.strip_prefix("--export-theme=") {
            options.export_theme = Some(PathBuf::from(value));
        } else if let Some(value) = argument.strip_prefix("--today=") {
            options.today = parse_date(value)?;
        } else if let Some(value) = argument.strip_prefix("--output=") {
            options.output = Some(PathBuf::from(value));
        } else if let Some(value) = argument.strip_prefix("--set-wallpaper=") {
            options.set_wallpaper = parse_bool(value)?;
        } else if matches!(
            argument.as_str(),
            "--width"
                | "--height"
                | "--year"
                | "--theme"
                | "--theme-file"
                | "--export-theme"
                | "--today"
                | "--output"
                | "--set-wallpaper"
        ) {
            index += 1;
            let value = args.get(index).context("参数缺少取值")?;
            match argument.as_str() {
                "--width" => options.width = parse_u32("宽度", value)?,
                "--height" => options.height = parse_u32("高度", value)?,
                "--year" => options.year = parse_i32("年份", value)?,
                "--theme" => options.requested_theme = Some(value.to_owned()),
                "--theme-file" => options.theme_file = Some(PathBuf::from(value)),
                "--export-theme" => options.export_theme = Some(PathBuf::from(value)),
                "--today" => options.today = parse_date(value)?,
                "--output" => options.output = Some(PathBuf::from(value)),
                "--set-wallpaper" => options.set_wallpaper = parse_bool(value)?,
                _ => unreachable!(),
            }
        } else {
            bail!("未知参数：{argument}");
        }
        index += 1;
    }

    if options.preview && options.update {
        bail!("--preview 与 --update 不能同时使用");
    }
    if options.preview && options.export_theme.is_some() {
        bail!("--preview 与 --export-theme 不能同时使用");
    }
    if options.theme_file.is_some() && options.export_theme.is_some() {
        bail!("--theme-file 与 --export-theme 不能同时使用");
    }
    if !options.preview
        && !options.show_version
        && !options.update
        && options.export_theme.is_none()
    {
        options.update = true;
    }
    Ok(options)
}

fn parse_u32(label: &str, value: &str) -> Result<u32> {
    value
        .parse::<u32>()
        .with_context(|| format!("{label}必须是正整数"))
}

fn parse_i32(label: &str, value: &str) -> Result<i32> {
    value
        .parse::<i32>()
        .with_context(|| format!("{label}必须是整数"))
}

fn parse_bool(value: &str) -> Result<bool> {
    match value {
        "true" | "1" => Ok(true),
        "false" | "0" => Ok(false),
        _ => bail!("--set-wallpaper 仅支持 true 或 false"),
    }
}

fn parse_date(value: &str) -> Result<NaiveDate> {
    NaiveDate::parse_from_str(value, "%Y-%m-%d").context("当天日期格式应为 YYYY-MM-DD")
}

#[cfg(windows)]
fn desktop_resolution() -> (u32, u32) {
    unsafe {
        SetProcessDPIAware();
    }
    sanitize_desktop_resolution(unsafe { GetSystemMetrics(0) }, unsafe {
        GetSystemMetrics(1)
    })
}

#[cfg(not(windows))]
fn desktop_resolution() -> (u32, u32) {
    (DEFAULT_WIDTH, DEFAULT_HEIGHT)
}

fn sanitize_desktop_resolution(width: i32, height: i32) -> (u32, u32) {
    if width >= 800 && height >= 600 {
        (width as u32, height as u32)
    } else {
        (DEFAULT_WIDTH, DEFAULT_HEIGHT)
    }
}

fn resolve_theme(requested: Option<&str>, theme_file: Option<&Path>) -> Result<Theme> {
    let settings = load_settings()?;
    if let Some(path) = theme_file {
        if let Some(value) = requested
            && value != "custom"
        {
            bail!("指定 --theme-file 时，主题只能省略或设为 custom");
        }
        let theme = Theme::custom(load_custom_theme(path)?).map_err(anyhow::Error::msg)?;
        save_selected_theme(&theme, &settings)?;
        return Ok(theme);
    }
    if let Some(value) = requested {
        if value == "custom" {
            let custom = settings
                .custom_theme
                .clone()
                .context("尚未导入自定义主题，请先使用 --theme-file 导入")?;
            let theme = Theme::custom(custom).map_err(anyhow::Error::msg)?;
            save_selected_theme(&theme, &settings)?;
            return Ok(theme);
        }
        let theme = Theme::parse(value).context("主题仅支持 dark、light 或 custom")?;
        save_selected_theme(&theme, &settings)?;
        return Ok(theme);
    }
    if settings.theme == "custom"
        && let Some(custom) = settings.custom_theme
    {
        return Theme::custom(custom).map_err(anyhow::Error::msg);
    }
    Ok(Theme::parse(&settings.theme).unwrap_or(Theme::Dark))
}

pub(crate) fn load_custom_theme(path: &Path) -> Result<CustomTheme> {
    let data = fs::read(path).with_context(|| format!("读取自定义主题失败：{}", path.display()))?;
    let theme = serde_json::from_slice::<CustomTheme>(&data).context("解析自定义主题失败")?;
    theme.validate().map_err(anyhow::Error::msg)?;
    Ok(theme)
}

pub(crate) fn export_theme(path: &Path, theme: &Theme, quiet: bool) -> Result<()> {
    let output = absolute_path(path)?;
    let parent = output.parent().context("自定义主题输出路径没有父目录")?;
    fs::create_dir_all(parent).context("创建自定义主题输出目录失败")?;
    let data = serde_json::to_vec_pretty(&theme.exportable()).context("编码自定义主题失败")?;
    fs::write(&output, data)
        .with_context(|| format!("写入自定义主题失败：{}", output.display()))?;
    if !quiet {
        println!("已导出自定义主题：{}", output.display());
    }
    Ok(())
}

fn save_selected_theme(theme: &Theme, previous: &Settings) -> Result<()> {
    save_settings(&Settings {
        theme: theme.as_str().to_owned(),
        custom_theme: theme
            .custom_theme()
            .cloned()
            .or_else(|| previous.custom_theme.clone()),
    })
}

struct RenderOptions {
    width: u32,
    height: u32,
    year: i32,
    theme: Theme,
    today: NaiveDate,
    output: Option<PathBuf>,
    set_wallpaper: bool,
    quiet: bool,
}

fn render_wallpaper(options: RenderOptions) -> Result<()> {
    if options.width < 800 || options.height < 600 {
        bail!("壁纸尺寸至少需要 800×600");
    }
    if !(1900..=2100).contains(&options.year) {
        bail!("日历年份仅支持 1900-2100");
    }
    let output = match options.output {
        Some(path) => path,
        None => application_data_dir()?.join(WALLPAPER_NAME),
    };
    let output = absolute_path(&output)?;
    let parent = output.parent().context("壁纸输出路径没有父目录")?;
    fs::create_dir_all(parent).context("创建壁纸输出目录失败")?;

    let svg = calendar::wallpaper_svg(
        options.width,
        options.height,
        options.year,
        &options.theme,
        options.today,
    );
    rasterize_svg(&svg, options.width, options.height, &output)?;
    let metadata = fs::metadata(&output).context("读取生成的壁纸失败")?;
    if metadata.len() == 0 {
        bail!("未生成有效壁纸图片");
    }
    if options.set_wallpaper {
        set_desktop_wallpaper(&output)?;
    }
    if !options.quiet {
        println!("已生成：{}", output.display());
        if options.set_wallpaper {
            println!("已设置为 Windows 桌面背景。");
        }
    }
    Ok(())
}

pub(crate) fn apply_selected_theme(theme: &Theme) -> Result<()> {
    let settings = load_settings()?;
    save_selected_theme(theme, &settings)?;
    let now = Local::now().date_naive();
    let (width, height) = desktop_resolution();
    render_wallpaper(RenderOptions {
        width,
        height,
        year: now.year(),
        theme: theme.clone(),
        today: now,
        output: None,
        set_wallpaper: true,
        quiet: true,
    })
}

fn rasterize_svg(svg: &str, width: u32, height: u32, output: &Path) -> Result<()> {
    let pixmap = render_svg_pixmap(svg, width, height)?;
    pixmap.save_png(output).context("写入 PNG 壁纸失败")?;
    Ok(())
}

pub(crate) fn render_svg_pixels(svg: &str, width: u32, height: u32) -> Result<Vec<u8>> {
    Ok(render_svg_pixmap(svg, width, height)?.data().to_vec())
}

fn render_svg_pixmap(svg: &str, width: u32, height: u32) -> Result<resvg::tiny_skia::Pixmap> {
    use resvg::{tiny_skia, usvg};

    let mut options = usvg::Options::default();
    options.fontdb_mut().load_system_fonts();
    let tree = usvg::Tree::from_str(svg, &options).context("解析原生日历版式失败")?;
    let mut pixmap = tiny_skia::Pixmap::new(width, height).context("创建壁纸画布失败")?;
    resvg::render(&tree, tiny_skia::Transform::default(), &mut pixmap.as_mut());
    Ok(pixmap)
}

fn application_data_dir() -> Result<PathBuf> {
    let local_app_data = env::var_os("LOCALAPPDATA").context("未找到 LOCALAPPDATA 目录")?;
    let path = PathBuf::from(local_app_data).join("YueXu");
    fs::create_dir_all(&path).context("创建应用数据目录失败")?;
    Ok(path)
}

fn absolute_path(path: &Path) -> Result<PathBuf> {
    if path.is_absolute() {
        Ok(path.to_path_buf())
    } else {
        Ok(env::current_dir()?.join(path))
    }
}

fn load_settings() -> Result<Settings> {
    let path = application_data_dir()?.join("settings.json");
    match fs::read(&path) {
        Ok(data) => serde_json::from_slice(&data).context("解析设置失败"),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(Settings::default()),
        Err(error) => Err(error).context("读取设置失败"),
    }
}

fn save_settings(settings: &Settings) -> Result<()> {
    let path = application_data_dir()?.join("settings.json");
    let temporary = path.with_extension(format!("json.{}.tmp", std::process::id()));
    let data = serde_json::to_vec_pretty(settings).context("编码设置失败")?;
    fs::write(&temporary, data).context("写入设置失败")?;
    if let Err(error) = replace_settings_file(&temporary, &path) {
        let _ = fs::remove_file(&temporary);
        return Err(error).context("替换旧设置失败");
    }
    Ok(())
}

#[cfg(windows)]
fn replace_settings_file(source: &Path, destination: &Path) -> std::io::Result<()> {
    let source = wide(source.as_os_str());
    let destination = wide(destination.as_os_str());
    if unsafe { MoveFileExW(source.as_ptr(), destination.as_ptr(), 0x0000_0001) } == 0 {
        Err(std::io::Error::last_os_error())
    } else {
        Ok(())
    }
}

#[cfg(not(windows))]
fn replace_settings_file(source: &Path, destination: &Path) -> std::io::Result<()> {
    fs::rename(source, destination)
}

#[cfg(windows)]
fn set_desktop_wallpaper(path: &Path) -> Result<()> {
    let wide_path = wide(path.as_os_str());
    let result = unsafe {
        SystemParametersInfoW(
            0x0014,
            0,
            wide_path.as_ptr().cast::<c_void>().cast_mut(),
            0x0001 | 0x0002,
        )
    };
    if result == 0 {
        return Err(std::io::Error::last_os_error()).context("Windows 拒绝设置壁纸");
    }
    Ok(())
}

#[cfg(not(windows))]
fn set_desktop_wallpaper(_: &Path) -> Result<()> {
    bail!("月序仅支持 Windows")
}

fn usage() {
    eprintln!("{APP_NAME} {VERSION}");
    eprintln!("\n用法：");
    eprintln!("  LunarCalendar.exe --update --quiet");
    eprintln!("  LunarCalendar.exe --preview  # 打开原生设置窗口");
    eprintln!("  LunarCalendar.exe --update --theme light");
    eprintln!("  LunarCalendar.exe --theme-file my-theme.json");
    eprintln!("  LunarCalendar.exe --export-theme my-theme.json");
}

fn report_error(error: &anyhow::Error, quiet: bool) {
    eprintln!("月序更新失败：{error:#}");
    if !quiet {
        #[cfg(windows)]
        show_error(&format!("月序更新失败：\n{error:#}"));
    }
}

#[cfg(windows)]
fn wide(value: &OsStr) -> Vec<u16> {
    use std::os::windows::ffi::OsStrExt;
    value.encode_wide().chain(std::iter::once(0)).collect()
}

#[cfg(windows)]
fn show_error(message: &str) {
    let title = wide(OsStr::new(APP_NAME));
    let message = wide(OsStr::new(message));
    unsafe {
        MessageBoxW(0, message.as_ptr(), title.as_ptr(), 0x0000_0010);
    }
}

#[cfg(windows)]
#[link(name = "user32")]
unsafe extern "system" {
    fn SystemParametersInfoW(
        ui_action: u32,
        ui_param: u32,
        pv_param: *mut c_void,
        f_win_ini: u32,
    ) -> i32;
    fn MessageBoxW(h_wnd: isize, lp_text: *const u16, lp_caption: *const u16, u_type: u32) -> i32;
    fn SetProcessDPIAware() -> i32;
    fn GetSystemMetrics(index: i32) -> i32;
}

#[cfg(windows)]
#[link(name = "kernel32")]
unsafe extern "system" {
    fn MoveFileExW(existing_file_name: *const u16, new_file_name: *const u16, flags: u32) -> i32;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_update_options() {
        assert!(parse_bool("true").unwrap());
        assert!(!parse_bool("false").unwrap());
        assert!(parse_bool("yes").is_err());
    }

    #[test]
    fn accepts_expected_date_format() {
        assert_eq!(
            parse_date("2026-08-04").unwrap(),
            NaiveDate::from_ymd_opt(2026, 8, 4).unwrap()
        );
        assert!(parse_date("2026/08/04").is_err());
    }

    #[test]
    fn uses_screen_resolution_and_falls_back_for_invalid_metrics() {
        assert_eq!(sanitize_desktop_resolution(1920, 1200), (1920, 1200));
        assert_eq!(
            sanitize_desktop_resolution(0, 0),
            (DEFAULT_WIDTH, DEFAULT_HEIGHT)
        );
    }
}
