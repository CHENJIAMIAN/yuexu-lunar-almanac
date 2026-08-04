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
    Custom(Box<CustomTheme>),
}

impl Theme {
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "dark" => Some(Self::Dark),
            "light" => Some(Self::Light),
            _ => None,
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
            Self::Custom(_) => "custom",
        }
    }

    pub fn custom_theme(&self) -> Option<&CustomTheme> {
        match self {
            Self::Custom(theme) => Some(theme.as_ref()),
            Self::Dark | Self::Light => None,
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

pub fn wallpaper_svg(
    width: u32,
    height: u32,
    year: i32,
    theme: &Theme,
    today: NaiveDate,
) -> String {
    let palette = theme.palette();
    let width = f64::from(width);
    let height = f64::from(height);
    let unit = (width / 1920.0).min(height / 1080.0).max(0.65);
    let gutter_width = width * 0.15625;
    let right_padding = (width * 0.01875).max(28.0 * unit);
    let grid_left = gutter_width;
    let grid_top = 68.0 * unit;
    let grid_bottom = height - 107.0 * unit;
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
    fn builds_all_twelve_months_without_a_hero_column() {
        let svg = wallpaper_svg(
            1920,
            1080,
            2026,
            &Theme::Dark,
            NaiveDate::from_ymd_opt(2026, 8, 4).unwrap(),
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
        assert_eq!(Theme::parse("forest"), None);
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
        let svg = wallpaper_svg(
            1920,
            1080,
            2026,
            &theme,
            NaiveDate::from_ymd_opt(2026, 8, 4).unwrap(),
        );
        assert!(svg.contains("#112128"));
        assert!(svg.contains("#E2835C"));
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
