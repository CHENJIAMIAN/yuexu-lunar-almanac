use std::fmt::Write;

use chrono::{Datelike, NaiveDate, Weekday};
use lunar_rust::{
    lunar::LunarRefHelper,
    solar::{self, SolarRefHelper},
};
use serde::{Deserialize, Serialize};

const MONTH_NAMES: [&str; 12] = [
    "一月",
    "二月",
    "三月",
    "四月",
    "五月",
    "六月",
    "七月",
    "八月",
    "九月",
    "十月",
    "十一月",
    "十二月",
];
const MONTH_EN: [&str; 12] = [
    "JANUARY",
    "FEBRUARY",
    "MARCH",
    "APRIL",
    "MAY",
    "JUNE",
    "JULY",
    "AUGUST",
    "SEPTEMBER",
    "OCTOBER",
    "NOVEMBER",
    "DECEMBER",
];
const WEEK_NAMES: [&str; 7] = ["一", "二", "三", "四", "五", "六", "日"];
const SANS_FONT: &str = "Microsoft YaHei UI, Microsoft YaHei, SimSun, sans-serif";
const SERIF_FONT: &str = "SimSun, Microsoft YaHei, serif";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Theme {
    Dark,
    Light,
    Moonlit,
    Pine,
    Cinnabar,
    Mist,
    Custom(Box<CustomTheme>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuiltinTheme {
    Dark,
    Light,
    Moonlit,
    Pine,
    Cinnabar,
    Mist,
}

impl BuiltinTheme {
    pub const ALL: [(Self, &'static str); 6] = [
        (Self::Dark, "深色"),
        (Self::Light, "浅色"),
        (Self::Moonlit, "月海"),
        (Self::Pine, "松烟"),
        (Self::Cinnabar, "朱砂"),
        (Self::Mist, "雾蓝"),
    ];

    pub fn id(self) -> &'static str {
        match self {
            Self::Dark => "dark",
            Self::Light => "light",
            Self::Moonlit => "moonlit",
            Self::Pine => "pine",
            Self::Cinnabar => "cinnabar",
            Self::Mist => "mist",
        }
    }

    pub fn theme(self) -> Theme {
        match self {
            Self::Dark => Theme::Dark,
            Self::Light => Theme::Light,
            Self::Moonlit => Theme::Moonlit,
            Self::Pine => Theme::Pine,
            Self::Cinnabar => Theme::Cinnabar,
            Self::Mist => Theme::Mist,
        }
    }
}

impl Theme {
    pub fn parse(value: &str) -> Option<Self> {
        BuiltinTheme::ALL
            .into_iter()
            .find_map(|(preset, _)| (preset.id() == value).then(|| preset.theme()))
    }

    pub fn from_builtin(preset: BuiltinTheme) -> Self {
        preset.theme()
    }

    pub fn builtin(&self) -> Option<BuiltinTheme> {
        match self {
            Self::Dark => Some(BuiltinTheme::Dark),
            Self::Light => Some(BuiltinTheme::Light),
            Self::Moonlit => Some(BuiltinTheme::Moonlit),
            Self::Pine => Some(BuiltinTheme::Pine),
            Self::Cinnabar => Some(BuiltinTheme::Cinnabar),
            Self::Mist => Some(BuiltinTheme::Mist),
            Self::Custom(_) => None,
        }
    }

    pub fn custom(theme: CustomTheme) -> Result<Self, String> {
        theme.validate()?;
        Ok(Self::Custom(Box::new(theme)))
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Dark => "dark",
            Self::Light => "light",
            Self::Moonlit => "moonlit",
            Self::Pine => "pine",
            Self::Cinnabar => "cinnabar",
            Self::Mist => "mist",
            Self::Custom(_) => "custom",
        }
    }

    pub fn custom_theme(&self) -> Option<&CustomTheme> {
        match self {
            Self::Custom(theme) => Some(theme.as_ref()),
            Self::Dark | Self::Light | Self::Moonlit | Self::Pine | Self::Cinnabar | Self::Mist => {
                None
            }
        }
    }

    pub fn exportable(&self) -> CustomTheme {
        match self {
            Self::Dark => CustomTheme {
                name: "深色基础".to_owned(),
                palette: self.palette(),
            },
            Self::Light => CustomTheme {
                name: "浅色基础".to_owned(),
                palette: self.palette(),
            },
            Self::Moonlit => CustomTheme {
                name: "月海".to_owned(),
                palette: self.palette(),
            },
            Self::Pine => CustomTheme {
                name: "松烟".to_owned(),
                palette: self.palette(),
            },
            Self::Cinnabar => CustomTheme {
                name: "朱砂".to_owned(),
                palette: self.palette(),
            },
            Self::Mist => CustomTheme {
                name: "雾蓝".to_owned(),
                palette: self.palette(),
            },
            Self::Custom(theme) => theme.as_ref().clone(),
        }
    }

    fn palette(&self) -> Palette {
        match self {
            Self::Dark => Palette {
                paper: "#19211C".to_owned(),
                gutter: "#1F2420".to_owned(),
                card: "#1E2822".to_owned(),
                current_card: "#28362D".to_owned(),
                ink: "#F1EADF".to_owned(),
                soft: "#9FAAA1".to_owned(),
                muted: "#78857C".to_owned(),
                accent: "#DF6B54".to_owned(),
                accent_soft: "#E3A294".to_owned(),
                line: "#536057".to_owned(),
                footer: "#909A91".to_owned(),
            },
            Self::Light => Palette {
                paper: "#F4EEE3".to_owned(),
                gutter: "#E8DFD0".to_owned(),
                card: "#FBF7EF".to_owned(),
                current_card: "#FFF9EF".to_owned(),
                ink: "#282B27".to_owned(),
                soft: "#68716A".to_owned(),
                muted: "#858C84".to_owned(),
                accent: "#B0523E".to_owned(),
                accent_soft: "#8E3D30".to_owned(),
                line: "#CFC6B8".to_owned(),
                footer: "#707870".to_owned(),
            },
            Self::Moonlit => Palette {
                paper: "#172127".to_owned(),
                gutter: "#202A2D".to_owned(),
                card: "#1C2B30".to_owned(),
                current_card: "#29434A".to_owned(),
                ink: "#EAF1ED".to_owned(),
                soft: "#A8BCBA".to_owned(),
                muted: "#74898A".to_owned(),
                accent: "#E0A05A".to_owned(),
                accent_soft: "#C8A279".to_owned(),
                line: "#536B6E".to_owned(),
                footer: "#92A4A3".to_owned(),
            },
            Self::Pine => Palette {
                paper: "#18211C".to_owned(),
                gutter: "#243029".to_owned(),
                card: "#223027".to_owned(),
                current_card: "#2D4133".to_owned(),
                ink: "#F0EBDD".to_owned(),
                soft: "#AAB6A4".to_owned(),
                muted: "#7E8D7C".to_owned(),
                accent: "#D98A61".to_owned(),
                accent_soft: "#B9C493".to_owned(),
                line: "#566858".to_owned(),
                footer: "#97A398".to_owned(),
            },
            Self::Cinnabar => Palette {
                paper: "#3A2723".to_owned(),
                gutter: "#4A2F29".to_owned(),
                card: "#4A302A".to_owned(),
                current_card: "#5E3930".to_owned(),
                ink: "#F8EADD".to_owned(),
                soft: "#D7B7A4".to_owned(),
                muted: "#AE887A".to_owned(),
                accent: "#E58B68".to_owned(),
                accent_soft: "#EAB19C".to_owned(),
                line: "#7C5148".to_owned(),
                footer: "#C5A08F".to_owned(),
            },
            Self::Mist => Palette {
                paper: "#E7ECEF".to_owned(),
                gutter: "#D9E1E5".to_owned(),
                card: "#F5F7F6".to_owned(),
                current_card: "#FFF6E7".to_owned(),
                ink: "#29363B".to_owned(),
                soft: "#66777D".to_owned(),
                muted: "#87979D".to_owned(),
                accent: "#C76B4F".to_owned(),
                accent_soft: "#6D8791".to_owned(),
                line: "#B7C4C9".to_owned(),
                footer: "#738187".to_owned(),
            },
            Self::Custom(theme) => theme.palette.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CustomTheme {
    #[serde(default = "default_custom_name")]
    pub name: String,
    #[serde(flatten)]
    pub palette: Palette,
}

impl CustomTheme {
    pub fn validate(&self) -> Result<(), String> {
        if self.name.trim().is_empty() || self.name.chars().count() > 48 {
            return Err("主题名称应为 1-48 个字符".to_owned());
        }
        self.palette.validate()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Palette {
    pub paper: String,
    pub gutter: String,
    pub card: String,
    pub current_card: String,
    pub ink: String,
    pub soft: String,
    pub muted: String,
    pub accent: String,
    pub accent_soft: String,
    pub line: String,
    pub footer: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LayoutSettings {
    #[serde(default = "default_left_margin")]
    pub left_margin: u8,
    #[serde(default = "default_right_margin")]
    pub right_margin: u8,
    #[serde(default = "default_top_margin")]
    pub top_margin: u8,
    #[serde(default = "default_bottom_margin")]
    pub bottom_margin: u8,
}

impl Default for LayoutSettings {
    fn default() -> Self {
        Self {
            left_margin: default_left_margin(),
            right_margin: default_right_margin(),
            top_margin: default_top_margin(),
            bottom_margin: default_bottom_margin(),
        }
    }
}

impl LayoutSettings {
    pub const MIN_LEFT_MARGIN: u8 = 8;
    pub const MAX_LEFT_MARGIN: u8 = 32;
    pub const MIN_RIGHT_MARGIN: u8 = 1;
    pub const MAX_RIGHT_MARGIN: u8 = 12;
    pub const MIN_TOP_MARGIN: u8 = 3;
    pub const MAX_TOP_MARGIN: u8 = 18;
    pub const MIN_BOTTOM_MARGIN: u8 = 3;
    pub const MAX_BOTTOM_MARGIN: u8 = 18;

    pub fn normalized(self) -> Self {
        Self {
            left_margin: self
                .left_margin
                .clamp(Self::MIN_LEFT_MARGIN, Self::MAX_LEFT_MARGIN),
            right_margin: self
                .right_margin
                .clamp(Self::MIN_RIGHT_MARGIN, Self::MAX_RIGHT_MARGIN),
            top_margin: self
                .top_margin
                .clamp(Self::MIN_TOP_MARGIN, Self::MAX_TOP_MARGIN),
            bottom_margin: self
                .bottom_margin
                .clamp(Self::MIN_BOTTOM_MARGIN, Self::MAX_BOTTOM_MARGIN),
        }
    }
}

fn default_left_margin() -> u8 {
    16
}

fn default_right_margin() -> u8 {
    2
}

fn default_top_margin() -> u8 {
    6
}

fn default_bottom_margin() -> u8 {
    10
}

impl Palette {
    fn validate(&self) -> Result<(), String> {
        for (name, color) in [
            ("paper", &self.paper),
            ("gutter", &self.gutter),
            ("card", &self.card),
            ("currentCard", &self.current_card),
            ("ink", &self.ink),
            ("soft", &self.soft),
            ("muted", &self.muted),
            ("accent", &self.accent),
            ("accentSoft", &self.accent_soft),
            ("line", &self.line),
            ("footer", &self.footer),
        ] {
            validate_hex_color(name, color)?;
        }
        Ok(())
    }
}

fn default_custom_name() -> String {
    "未命名主题".to_owned()
}

fn validate_hex_color(name: &str, color: &str) -> Result<(), String> {
    if color.len() != 7
        || !color.starts_with('#')
        || !color.as_bytes()[1..].iter().all(u8::is_ascii_hexdigit)
    {
        return Err(format!("{name} 必须是 #RRGGBB 格式"));
    }
    Ok(())
}

pub fn wallpaper_svg_with_layout(
    width: u32,
    height: u32,
    year: i32,
    theme: &Theme,
    today: NaiveDate,
    layout: LayoutSettings,
) -> String {
    let palette = theme.palette();
    let layout = layout.normalized();
    let width = f64::from(width);
    let height = f64::from(height);
    let unit = (width / 1920.0).min(height / 1080.0).max(0.65);
    let gutter_width = width * f64::from(layout.left_margin) / 100.0;
    let right_padding = width * f64::from(layout.right_margin) / 100.0;
    let top_padding = height * f64::from(layout.top_margin) / 100.0;
    let bottom_padding = height * f64::from(layout.bottom_margin) / 100.0;
    let grid_left = gutter_width;
    let grid_top = top_padding;
    let grid_bottom = height - bottom_padding;
    let grid_width = width - grid_left - right_padding;
    let gap = 15.0 * unit;
    let card_width = (grid_width - gap * 3.0) / 4.0;
    let card_height = (grid_bottom - grid_top - gap * 2.0) / 3.0;
    let focus = if today.year() == year {
        today
    } else {
        NaiveDate::from_ymd_opt(year, 6, 15).expect("validated calendar year")
    };
    let focus_lunar = solar_lunar(focus);
    let ganzhi = focus_lunar.get_year_in_gan_zhi();

    let mut svg = String::with_capacity(180_000);
    let _ = write!(
        svg,
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{width:.0}" height="{height:.0}" viewBox="0 0 {width:.0} {height:.0}"><rect width="100%" height="100%" fill="{}"/><rect x="0" y="0" width="{gutter_width:.2}" height="{height:.2}" fill="{}"/><line x1="{gutter_width:.2}" y1="0" x2="{gutter_width:.2}" y2="{height:.2}" stroke="{}" stroke-opacity="0.42" stroke-width="{:.2}"/>"#,
        palette.paper, palette.gutter, palette.line, unit,
    );

    for month_index in 0..12 {
        let month = month_index as u32 + 1;
        let col = (month_index % 4) as f64;
        let row = (month_index / 4) as f64;
        let x = grid_left + col * (card_width + gap);
        let y = grid_top + row * (card_height + gap);
        render_month(
            &mut svg,
            year,
            month,
            x,
            y,
            card_width,
            card_height,
            unit,
            today,
            &palette,
        );
    }

    let footer_y = height - 21.0 * unit;
    let footer_left = grid_left + 2.0 * unit;
    let footer_right = width - right_padding;
    text(
        &mut svg,
        footer_left,
        footer_y,
        &format!("{year} · {ganzhi}年 · 月序 / LUNAR ALMANAC"),
        10.0 * unit,
        &palette.footer,
        "start",
        400,
        SANS_FONT,
    );
    let marker_x = footer_right - 144.0 * unit;
    let _ = write!(
        svg,
        r#"<circle cx="{marker_x:.2}" cy="{:.2}" r="{:.2}" fill="{}"/>"#,
        footer_y - 3.2 * unit,
        3.4 * unit,
        palette.accent,
    );
    text(
        &mut svg,
        footer_right,
        footer_y,
        "今日高亮 · LOCAL / OFFLINE",
        9.0 * unit,
        &palette.footer,
        "end",
        400,
        SANS_FONT,
    );
    svg.push_str("</svg>");
    svg
}

#[allow(clippy::too_many_arguments)]
fn render_month(
    svg: &mut String,
    year: i32,
    month: u32,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    unit: f64,
    today: NaiveDate,
    palette: &Palette,
) {
    let is_current_month = today.year() == year && today.month() == month;
    let stroke_width = if is_current_month { 1.8 * unit } else { unit };
    let fill = if is_current_month {
        &palette.current_card
    } else {
        &palette.card
    };
    let stroke = if is_current_month {
        &palette.accent
    } else {
        &palette.line
    };
    let _ = write!(
        svg,
        r#"<rect x="{x:.2}" y="{y:.2}" width="{width:.2}" height="{height:.2}" fill="{fill}" stroke="{stroke}" stroke-width="{stroke_width:.2}"/>"#,
    );

    let inner_left = x + 14.0 * unit;
    let inner_right = x + width - 14.0 * unit;
    let header_baseline = y + 22.0 * unit;
    text(
        svg,
        inner_left,
        header_baseline,
        &format!("{month:02}"),
        9.0 * unit,
        &palette.accent,
        "start",
        500,
        SANS_FONT,
    );
    text(
        svg,
        inner_left + 29.0 * unit,
        header_baseline,
        MONTH_NAMES[(month - 1) as usize],
        15.0 * unit,
        &palette.ink,
        "start",
        600,
        SANS_FONT,
    );
    text(
        svg,
        inner_right,
        header_baseline,
        MONTH_EN[(month - 1) as usize],
        7.5 * unit,
        &palette.muted,
        "end",
        500,
        SANS_FONT,
    );

    let header_line_y = y + 35.0 * unit;
    let _ = write!(
        svg,
        r#"<line x1="{inner_left:.2}" y1="{header_line_y:.2}" x2="{inner_right:.2}" y2="{header_line_y:.2}" stroke="{}" stroke-width="{:.2}"/>"#,
        palette.line,
        0.8 * unit,
    );

    let day_grid_left = x + 10.0 * unit;
    let day_grid_width = width - 20.0 * unit;
    let cell_width = day_grid_width / 7.0;
    let weekday_y = y + 52.0 * unit;
    for (index, weekday) in WEEK_NAMES.iter().enumerate() {
        let center_x = day_grid_left + (index as f64 + 0.5) * cell_width;
        let fill = if index >= 5 {
            &palette.accent_soft
        } else {
            &palette.soft
        };
        text(
            svg,
            center_x,
            weekday_y,
            weekday,
            8.5 * unit,
            fill,
            "middle",
            400,
            SANS_FONT,
        );
    }

    let day_grid_top = y + 62.0 * unit;
    let day_grid_bottom = y + height - 11.0 * unit;
    let cell_height = (day_grid_bottom - day_grid_top) / 6.0;
    let first_date = NaiveDate::from_ymd_opt(year, month, 1).expect("validated calendar month");
    let first_offset = first_date.weekday().num_days_from_monday();
    let days = days_in_month(year, month);

    for day in 1..=days {
        let date = NaiveDate::from_ymd_opt(year, month, day).expect("valid day in calendar month");
        let position = first_offset + day - 1;
        let col = position % 7;
        let row = position / 7;
        let center_x = day_grid_left + (f64::from(col) + 0.5) * cell_width;
        let center_y = day_grid_top + (f64::from(row) + 0.5) * cell_height;
        let is_today = date == today;
        let is_weekend = matches!(date.weekday(), Weekday::Sat | Weekday::Sun);
        let number_fill = if is_today {
            "#FFF8EE"
        } else if is_weekend {
            &palette.accent_soft
        } else {
            &palette.ink
        };
        let lunar_fill = if is_today { "#FFF8EE" } else { &palette.soft };
        if is_today {
            let radius = (cell_width.min(cell_height) * 0.43).max(10.0 * unit);
            let _ = write!(
                svg,
                r#"<circle cx="{center_x:.2}" cy="{center_y:.2}" r="{radius:.2}" fill="{}"/>"#,
                palette.accent,
            );
        }
        text(
            svg,
            center_x,
            center_y - 1.2 * unit,
            &day.to_string(),
            13.0 * unit,
            number_fill,
            "middle",
            600,
            SANS_FONT,
        );
        text(
            svg,
            center_x,
            center_y + 9.0 * unit,
            &lunar_short_label(date),
            7.5 * unit,
            lunar_fill,
            "middle",
            400,
            SERIF_FONT,
        );
    }
}

#[allow(clippy::too_many_arguments)]
fn text(
    svg: &mut String,
    x: f64,
    y: f64,
    value: &str,
    font_size: f64,
    fill: &str,
    anchor: &str,
    weight: u16,
    family: &str,
) {
    let _ = write!(
        svg,
        r#"<text x="{x:.2}" y="{y:.2}" text-anchor="{anchor}" fill="{fill}" font-family="{family}" font-size="{font_size:.2}" font-weight="{weight}">{}</text>"#,
        escape_xml(value),
    );
}

fn solar_lunar(date: NaiveDate) -> lunar_rust::lunar::LunarRef {
    solar::from_ymd(
        i64::from(date.year()),
        i64::from(date.month()),
        i64::from(date.day()),
    )
    .get_lunar()
}

fn lunar_short_label(date: NaiveDate) -> String {
    let lunar = solar_lunar(date);
    if lunar.get_day() == 1 {
        format!("{}月", lunar.get_month_in_chinese())
    } else {
        lunar.get_day_in_chinese()
    }
}

fn days_in_month(year: i32, month: u32) -> u32 {
    let next = if month == 12 {
        NaiveDate::from_ymd_opt(year + 1, 1, 1).expect("valid next year")
    } else {
        NaiveDate::from_ymd_opt(year, month + 1, 1).expect("valid next month")
    };
    (next - chrono::Duration::days(1)).day()
}

fn escape_xml(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_known_lunar_new_year() {
        assert_eq!(
            lunar_short_label(NaiveDate::from_ymd_opt(2025, 1, 29).unwrap()),
            "正月"
        );
        assert_eq!(
            lunar_short_label(NaiveDate::from_ymd_opt(2025, 1, 30).unwrap()),
            "初二"
        );
    }

    #[test]
    fn labels_a_leap_lunar_month() {
        assert_eq!(
            lunar_short_label(NaiveDate::from_ymd_opt(2023, 3, 22).unwrap()),
            "闰二月"
        );
    }

    #[test]
    fn builds_all_twelve_months_without_a_hero_column() {
        let svg = wallpaper_svg_with_layout(
            1920,
            1080,
            2026,
            &Theme::Dark,
            NaiveDate::from_ymd_opt(2026, 8, 4).unwrap(),
            LayoutSettings::default(),
        );
        assert!(svg.contains("12"));
        assert!(svg.contains("十二月"));
        assert!(svg.contains("月序 / LUNAR ALMANAC"));
        assert!(!svg.contains("hero-column"));
    }

    #[test]
    fn accepts_only_supported_themes() {
        assert_eq!(Theme::parse("dark"), Some(Theme::Dark));
        assert_eq!(Theme::parse("light"), Some(Theme::Light));
        assert_eq!(Theme::parse("moonlit"), Some(Theme::Moonlit));
        assert_eq!(Theme::parse("pine"), Some(Theme::Pine));
        assert_eq!(Theme::parse("cinnabar"), Some(Theme::Cinnabar));
        assert_eq!(Theme::parse("mist"), Some(Theme::Mist));
        assert_eq!(Theme::parse("forest"), None);
    }

    #[test]
    fn round_trips_all_builtin_theme_ids() {
        for (preset, _) in BuiltinTheme::ALL {
            assert_eq!(
                Theme::parse(preset.id()).and_then(|theme| theme.builtin()),
                Some(preset)
            );
        }
    }

    #[test]
    fn accepts_valid_custom_palette_and_uses_it_in_svg() {
        let custom = CustomTheme {
            name: "雨夜青".to_owned(),
            palette: Palette {
                paper: "#112128".to_owned(),
                gutter: "#0B151A".to_owned(),
                card: "#162B33".to_owned(),
                current_card: "#203D47".to_owned(),
                ink: "#E5F1F0".to_owned(),
                soft: "#A5B8B6".to_owned(),
                muted: "#718B8B".to_owned(),
                accent: "#E2835C".to_owned(),
                accent_soft: "#F2B38A".to_owned(),
                line: "#48646A".to_owned(),
                footer: "#91AAA9".to_owned(),
            },
        };
        let theme = Theme::custom(custom).unwrap();
        let svg = wallpaper_svg_with_layout(
            1920,
            1080,
            2026,
            &theme,
            NaiveDate::from_ymd_opt(2026, 8, 4).unwrap(),
            LayoutSettings::default(),
        );
        assert!(svg.contains("#112128"));
        assert!(svg.contains("#E2835C"));
    }

    #[test]
    fn uses_the_configured_margins() {
        let narrow = wallpaper_svg_with_layout(
            1920,
            1080,
            2026,
            &Theme::Dark,
            NaiveDate::from_ymd_opt(2026, 8, 4).unwrap(),
            LayoutSettings {
                left_margin: 8,
                right_margin: 1,
                top_margin: 4,
                bottom_margin: 6,
            },
        );
        let wide = wallpaper_svg_with_layout(
            1920,
            1080,
            2026,
            &Theme::Dark,
            NaiveDate::from_ymd_opt(2026, 8, 4).unwrap(),
            LayoutSettings {
                left_margin: 32,
                right_margin: 12,
                top_margin: 16,
                bottom_margin: 18,
            },
        );

        assert!(narrow.contains(r#"<rect x="153.60" y="43.20""#));
        assert!(wide.contains(r#"<rect x="614.40" y="172.80""#));
        assert_ne!(narrow, wide);
    }

    #[test]
    fn clamps_layout_margins_to_supported_ranges() {
        assert_eq!(
            LayoutSettings {
                left_margin: 0,
                right_margin: 255,
                top_margin: 0,
                bottom_margin: 255,
            }
            .normalized(),
            LayoutSettings {
                left_margin: LayoutSettings::MIN_LEFT_MARGIN,
                right_margin: LayoutSettings::MAX_RIGHT_MARGIN,
                top_margin: LayoutSettings::MIN_TOP_MARGIN,
                bottom_margin: LayoutSettings::MAX_BOTTOM_MARGIN,
            }
        );
    }

    #[test]
    fn rejects_unsafe_custom_color_values() {
        let mut palette = Theme::Dark.palette();
        palette.paper = "url(javascript:alert(1))".to_owned();
        assert!(
            Theme::custom(CustomTheme {
                name: "无效主题".to_owned(),
                palette,
            })
            .is_err()
        );
    }
}
