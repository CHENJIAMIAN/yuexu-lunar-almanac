#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

mod calendar;

use std::{
    env,
    ffi::{OsStr, c_void},
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail};
use calendar::Theme;
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
    today: NaiveDate,
    output: Option<PathBuf>,
    set_wallpaper: bool,
    protocol: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct Settings {
    theme: String,
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

    if let Some(protocol) = options.protocol {
        return handle_protocol(&protocol, options.quiet);
    }

    let theme = resolve_theme(options.requested_theme.as_deref())?;
    if options.preview {
        return open_preview(options.year, theme, options.today);
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
    let mut options = CliOptions {
        update: false,
        preview: false,
        quiet: false,
        show_version: false,
        width: DEFAULT_WIDTH,
        height: DEFAULT_HEIGHT,
        year: now.year(),
        requested_theme: None,
        today: now,
        output: None,
        set_wallpaper: true,
        protocol: None,
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
                "--today" => options.today = parse_date(value)?,
                "--output" => options.output = Some(PathBuf::from(value)),
                "--set-wallpaper" => options.set_wallpaper = parse_bool(value)?,
                _ => unreachable!(),
            }
        } else if argument.to_ascii_lowercase().starts_with("yuexu://") && args.len() == 1 {
            options.protocol = Some(argument.to_owned());
        } else {
            bail!("未知参数：{argument}");
        }
        index += 1;
    }

    if options.preview && options.update {
        bail!("--preview 与 --update 不能同时使用");
    }
    if !options.preview && !options.show_version && !options.update {
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

fn resolve_theme(requested: Option<&str>) -> Result<Theme> {
    if let Some(value) = requested {
        let theme = Theme::parse(value).context("主题仅支持 dark 或 light")?;
        save_settings(&Settings {
            theme: theme.as_str().to_owned(),
        })?;
        return Ok(theme);
    }
    let settings = load_settings()?;
    Ok(Theme::parse(&settings.theme).unwrap_or(Theme::Dark))
}

fn handle_protocol(raw_url: &str, quiet: bool) -> Result<()> {
    let theme = raw_url
        .strip_prefix("yuexu://theme/")
        .and_then(|value| value.split('?').next())
        .and_then(Theme::parse)
        .context("不支持的月序操作")?;
    save_settings(&Settings {
        theme: theme.as_str().to_owned(),
    })?;
    let now = Local::now().date_naive();
    render_wallpaper(RenderOptions {
        width: DEFAULT_WIDTH,
        height: DEFAULT_HEIGHT,
        year: now.year(),
        theme,
        today: now,
        output: None,
        set_wallpaper: true,
        quiet,
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
        options.theme,
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

fn rasterize_svg(svg: &str, width: u32, height: u32, output: &Path) -> Result<()> {
    use resvg::{tiny_skia, usvg};

    let mut options = usvg::Options::default();
    options.fontdb_mut().load_system_fonts();
    let tree = usvg::Tree::from_str(svg, &options).context("解析原生日历版式失败")?;
    let mut pixmap = tiny_skia::Pixmap::new(width, height).context("创建壁纸画布失败")?;
    resvg::render(&tree, tiny_skia::Transform::default(), &mut pixmap.as_mut());
    pixmap.save_png(output).context("写入 PNG 壁纸失败")?;
    Ok(())
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
    let temporary = path.with_extension("json.tmp");
    let data = serde_json::to_vec_pretty(settings).context("编码设置失败")?;
    fs::write(&temporary, data).context("写入设置失败")?;
    if path.exists() {
        fs::remove_file(&path).context("替换旧设置失败")?;
    }
    fs::rename(&temporary, &path).context("保存设置失败")?;
    Ok(())
}

fn open_preview(year: i32, theme: Theme, today: NaiveDate) -> Result<()> {
    if !(1900..=2100).contains(&year) {
        bail!("日历年份仅支持 1900-2100");
    }
    let preview_dir = application_data_dir()?.join("preview");
    for (relative, content) in preview_assets() {
        let target = preview_dir.join(relative);
        let parent = target.parent().context("预览资源路径无效")?;
        fs::create_dir_all(parent).context("创建预览资源目录失败")?;
        fs::write(target, content).context("写入预览资源失败")?;
    }
    let index = preview_dir.join("index.html");
    let mut url =
        url::Url::from_file_path(index).map_err(|_| anyhow::anyhow!("构造预览地址失败"))?;
    {
        let mut query = url.query_pairs_mut();
        query.append_pair("year", &year.to_string());
        query.append_pair("theme", theme.as_str());
        query.append_pair("today", &today.format("%Y-%m-%d").to_string());
        query.append_pair("native", "1");
    }
    open_url(url.as_str())
}

fn preview_assets() -> [(&'static str, &'static [u8]); 5] {
    [
        ("index.html", include_bytes!("../index.html")),
        ("src/styles.css", include_bytes!("../src/styles.css")),
        ("src/lunar.js", include_bytes!("../src/lunar.js")),
        ("src/calendar.js", include_bytes!("../src/calendar.js")),
        ("src/app.js", include_bytes!("../src/app.js")),
    ]
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

#[cfg(windows)]
fn open_url(url: &str) -> Result<()> {
    let operation = wide(OsStr::new("open"));
    let file = wide(OsStr::new(url));
    let result = unsafe {
        ShellExecuteW(
            0,
            operation.as_ptr(),
            file.as_ptr(),
            std::ptr::null(),
            std::ptr::null(),
            1,
        )
    };
    if result <= 32 {
        bail!("Windows 无法打开预览，ShellExecuteW 返回 {result}");
    }
    Ok(())
}

#[cfg(not(windows))]
fn open_url(_: &str) -> Result<()> {
    bail!("月序仅支持 Windows")
}

fn usage() {
    eprintln!("{APP_NAME} {VERSION}");
    eprintln!("\n用法：");
    eprintln!("  LunarCalendar.exe --update --quiet");
    eprintln!("  LunarCalendar.exe --preview");
    eprintln!("  LunarCalendar.exe --update --theme light");
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
}

#[cfg(windows)]
#[link(name = "shell32")]
unsafe extern "system" {
    fn ShellExecuteW(
        hwnd: isize,
        lp_operation: *const u16,
        lp_file: *const u16,
        lp_parameters: *const u16,
        lp_directory: *const u16,
        n_show_cmd: i32,
    ) -> isize;
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
}
