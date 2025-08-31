//! Module defining the constants used for aesthetic purposes (colors, borders...)

use crate::gui::styles::types::palette::Palette;
use crate::gui::styles::types::palette_extension::PaletteExtension;
use crate::{FONT_FAMILY_NAME, ICON_FONT_FAMILY_NAME};
use iced::font::{Family, Stretch, Style, Weight};
use iced::{Color, Font};

// night theme - 优化后的深色主题
const PRIMARY_NIGHT: Color = Color {
    r: 0.11, // #1c1c1c - 更深的背景色
    g: 0.11,
    b: 0.11,
    a: 1.0,
};
const SECONDARY_NIGHT: Color = Color {
    r: 0.25, // #404040 - 更柔和的次要色
    g: 0.25,
    b: 0.25,
    a: 1.0,
};
pub const NIGHT_PALETTE: Palette = Palette {
    primary: PRIMARY_NIGHT,
    secondary: SECONDARY_NIGHT,
    starred: Color {
        r: 245.0 / 255.0,
        g: 193.0 / 255.0,
        b: 39.0 / 255.0,
        a: 0.5,
    },
    outgoing: SECONDARY_DAY,
    text_headers: Color::BLACK,
    text_body: Color::WHITE,
};

pub const BUTTONS_NIGHT: Color = Color {
    r: 0.18, // #2e2e2e - 更明显的按钮背景
    g: 0.18,
    b: 0.18,
    a: 1.0,
};

pub const NIGHT_PALETTE_EXTENSION: PaletteExtension = PaletteExtension {
    is_nightly: true,
    font: SARASA_MONO,
    font_headers: SARASA_MONO_BOLD,
    alpha_chart_badge: 0.15,
    alpha_round_borders: 0.4,
    alpha_round_containers: 0.3,
    buttons_color: BUTTONS_NIGHT,
    red_alert_color: RED_ALERT_COLOR_NIGHTLY,
};

// day theme - 优化后的浅色主题
const PRIMARY_DAY: Color = Color {
    r: 0.98, // #fafafa - 更柔和的白色背景
    g: 0.98,
    b: 0.98,
    a: 1.0,
};
const SECONDARY_DAY: Color = Color {
    r: 0.13, // #2196f3 - 现代化的蓝色
    g: 0.59,
    b: 0.95,
    a: 1.0,
};
pub const DAY_PALETTE: Palette = Palette {
    primary: PRIMARY_DAY,
    secondary: SECONDARY_DAY,
    outgoing: SECONDARY_NIGHT,
    starred: Color {
        r: 215.0 / 255.0,
        g: 163.0 / 255.0,
        b: 19.0 / 255.0,
        a: 0.9,
    },
    text_headers: Color::WHITE,
    text_body: Color::BLACK,
};

pub const BUTTONS_DAY: Color = Color {
    r: 0.95, // #f2f2f2 - 更柔和的按钮背景
    g: 0.95,
    b: 0.95,
    a: 1.0,
};

pub const DAY_PALETTE_EXTENSION: PaletteExtension = PaletteExtension {
    is_nightly: false,
    font: SARASA_MONO_BOLD,
    font_headers: SARASA_MONO,
    alpha_chart_badge: 0.75,
    alpha_round_borders: 0.45,
    alpha_round_containers: 0.2,
    buttons_color: BUTTONS_DAY,
    red_alert_color: RED_ALERT_COLOR_DAILY,
};

// deep sea theme
const PRIMARY_DEEP_SEA: Color = Color {
    r: 28.0 / 255.0,
    g: 49.0 / 255.0,
    b: 94.0 / 255.0,
    a: 1.0,
};
const SECONDARY_DEEP_SEA: Color = Color {
    r: 8.0 / 255.0,
    g: 131.0 / 255.0,
    b: 149.0 / 255.0,
    a: 1.0,
};
const OUTGOING_DEEP_SEA: Color = Color {
    r: 254.0 / 255.0,
    g: 254.0 / 255.0,
    b: 134.0 / 255.0,
    a: 1.0,
};
pub const DEEP_SEA_PALETTE: Palette = Palette {
    primary: PRIMARY_DEEP_SEA,
    secondary: SECONDARY_DEEP_SEA,
    starred: Color {
        r: 245.0 / 255.0,
        g: 193.0 / 255.0,
        b: 39.0 / 255.0,
        a: 0.5,
    },
    outgoing: OUTGOING_DEEP_SEA,
    text_headers: Color::BLACK,
    text_body: Color::WHITE,
};

pub const BUTTONS_DEEP_SEA: Color = Color {
    r: 48.0 / 255.0,
    g: 71.0 / 255.0,
    b: 94.0 / 255.0,
    a: 1.0,
};

pub const DEEP_SEA_PALETTE_EXTENSION: PaletteExtension = PaletteExtension {
    is_nightly: true,
    font: SARASA_MONO,
    font_headers: SARASA_MONO_BOLD,
    alpha_chart_badge: 0.15,
    alpha_round_borders: 0.35,
    alpha_round_containers: 0.15,
    buttons_color: BUTTONS_DEEP_SEA,
    red_alert_color: RED_ALERT_COLOR_NIGHTLY,
};

// mon amour theme
const SECONDARY_MON_AMOUR: Color = Color {
    r: 67.0 / 255.0,
    g: 44.0 / 255.0,
    b: 122.0 / 255.0,
    a: 1.0,
};
const PRIMARY_MON_AMOUR: Color = Color {
    r: 245.0 / 255.0,
    g: 245.0 / 255.0,
    b: 220.0 / 255.0,
    a: 1.0,
};
const OUTGOING_MON_AMOUR: Color = Color {
    r: 58.0 / 255.0,
    g: 166.0 / 255.0,
    b: 185.0 / 255.0,
    a: 1.0,
};
pub const MON_AMOUR_PALETTE: Palette = Palette {
    primary: PRIMARY_MON_AMOUR,
    secondary: SECONDARY_MON_AMOUR,
    starred: Color {
        r: 215.0 / 255.0,
        g: 163.0 / 255.0,
        b: 19.0 / 255.0,
        a: 0.9,
    },
    outgoing: OUTGOING_MON_AMOUR,
    text_headers: Color::WHITE,
    text_body: Color::BLACK,
};

pub const BUTTONS_MON_AMOUR: Color = Color {
    r: 242.0 / 255.0,
    g: 190.0 / 255.0,
    b: 209.0 / 255.0,
    a: 1.0,
};

pub const MON_AMOUR_PALETTE_EXTENSION: PaletteExtension = PaletteExtension {
    is_nightly: false,
    font: SARASA_MONO_BOLD,
    font_headers: SARASA_MONO,
    alpha_chart_badge: 0.75,
    alpha_round_borders: 0.5,
    alpha_round_containers: 0.25,
    buttons_color: BUTTONS_MON_AMOUR,
    red_alert_color: RED_ALERT_COLOR_DAILY,
};

pub const SARASA_MONO_BYTES: &[u8] =
    include_bytes!("../../../resources/fonts/full/sarasa-mono-sc-regular.ttf");

pub const SARASA_MONO_BOLD_BYTES: &[u8] =
    include_bytes!("../../../resources/fonts/full/sarasa-mono-sc-bold.ttf");

pub const MAPLE_MONO_NF_CN_REGULAR: &[u8] =
    include_bytes!("../../../resources/fonts/full/MapleMono-NF-CN-Regular.ttf");

//font to display icons
pub const ICONS_BYTES: &[u8] =
    include_bytes!("../../../resources/icons/Icons for Domain Manager.ttf");

pub const ICONS: Font = Font::with_name(ICON_FONT_FAMILY_NAME);

pub const SARASA_MONO_BOLD: Font = Font {
    family: Family::Name(FONT_FAMILY_NAME),
    weight: Weight::Bold,
    stretch: Stretch::Normal,
    style: Style::Normal,
};

pub const SARASA_MONO: Font = Font {
    family: Family::Name(FONT_FAMILY_NAME),
    weight: Weight::Normal,
    stretch: Stretch::Normal,
    style: Style::Normal,
};

// font sizes
pub const FONT_SIZE_FOOTER: f32 = 14.3;
pub const FONT_SIZE_BODY: f32 = 16.8;
pub const FONT_SIZE_SUBTITLE: f32 = 18.3;
pub const FONT_SIZE_TITLE: f32 = 19.9;

// border styles
pub const BORDER_WIDTH: f32 = 2.0;
pub const CHARTS_LINE_BORDER: u32 = 1;
pub const BORDER_ROUNDED_RADIUS: f32 = 15.0;
pub const BORDER_BUTTON_RADIUS: f32 = 180.0;

// red colors for alerts
pub const RED_ALERT_COLOR_NIGHTLY: Color = Color {
    r: 1.0,
    g: 0.4,
    b: 0.4,
    a: 1.0,
};
pub const RED_ALERT_COLOR_DAILY: Color = Color {
    r: 0.701_960_8,
    g: 0.0,
    b: 0.0,
    a: 1.0,
};
