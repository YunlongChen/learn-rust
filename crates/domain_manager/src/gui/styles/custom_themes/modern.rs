//! 现代化主题 - 提供更好的视觉体验和用户界面

use iced::Color;
use crate::gui::styles::types::palette::Palette;
use crate::gui::styles::types::palette_extension::PaletteExtension;
use crate::gui::styles::style_constants::{SARASA_MONO, SARASA_MONO_BOLD};

// 现代化深色主题
const PRIMARY_MODERN_DARK: Color = Color {
    r: 0.09,  // #171717 - 现代深色背景
    g: 0.09,
    b: 0.09,
    a: 1.0,
};

const SECONDARY_MODERN_DARK: Color = Color {
    r: 0.16,  // #292929 - 卡片背景色
    g: 0.16,
    b: 0.16,
    a: 1.0,
};

const ACCENT_MODERN_DARK: Color = Color {
    r: 0.40,  // #6366f1 - 现代紫色强调色
    g: 0.40,
    b: 0.95,
    a: 1.0,
};

pub const MODERN_DARK_PALETTE: Palette = Palette {
    primary: PRIMARY_MODERN_DARK,
    secondary: SECONDARY_MODERN_DARK,
    starred: Color {
        r: 1.0,   // #fbbf24 - 金色星标
        g: 0.75,
        b: 0.14,
        a: 0.8,
    },
    outgoing: ACCENT_MODERN_DARK,
    text_headers: Color {
        r: 0.95,  // #f3f4f6 - 浅色标题文字
        g: 0.96,
        b: 0.96,
        a: 1.0,
    },
    text_body: Color {
        r: 0.87,  // #d1d5db - 浅色正文文字
        g: 0.84,
        b: 0.86,
        a: 1.0,
    },
};

const BUTTONS_MODERN_DARK: Color = Color {
    r: 0.20,  // #333333 - 按钮背景
    g: 0.20,
    b: 0.20,
    a: 1.0,
};

const RED_ALERT_MODERN_DARK: Color = Color {
    r: 0.96,  // #f87171 - 现代红色警告
    g: 0.44,
    b: 0.44,
    a: 1.0,
};

pub const MODERN_DARK_PALETTE_EXTENSION: PaletteExtension = PaletteExtension {
    is_nightly: true,
    font: SARASA_MONO,
    font_headers: SARASA_MONO_BOLD,
    alpha_chart_badge: 0.12,
    alpha_round_borders: 0.25,
    alpha_round_containers: 0.15,
    buttons_color: BUTTONS_MODERN_DARK,
    red_alert_color: RED_ALERT_MODERN_DARK,
};

// 现代化浅色主题
const PRIMARY_MODERN_LIGHT: Color = Color {
    r: 0.99,  // #fcfcfc - 现代浅色背景
    g: 0.99,
    b: 0.99,
    a: 1.0,
};

const SECONDARY_MODERN_LIGHT: Color = Color {
    r: 0.96,  // #f5f5f5 - 卡片背景色
    g: 0.96,
    b: 0.96,
    a: 1.0,
};

const ACCENT_MODERN_LIGHT: Color = Color {
    r: 0.25,  // #3b82f6 - 现代蓝色强调色
    g: 0.51,
    b: 0.96,
    a: 1.0,
};

pub const MODERN_LIGHT_PALETTE: Palette = Palette {
    primary: PRIMARY_MODERN_LIGHT,
    secondary: SECONDARY_MODERN_LIGHT,
    starred: Color {
        r: 0.92,  // #eab308 - 金色星标
        g: 0.70,
        b: 0.03,
        a: 0.9,
    },
    outgoing: ACCENT_MODERN_LIGHT,
    text_headers: Color {
        r: 0.11,  // #1f2937 - 深色标题文字
        g: 0.16,
        b: 0.22,
        a: 1.0,
    },
    text_body: Color {
        r: 0.37,  // #6b7280 - 深色正文文字
        g: 0.45,
        b: 0.50,
        a: 1.0,
    },
};

const BUTTONS_MODERN_LIGHT: Color = Color {
    r: 0.93,  // #eeeeee - 按钮背景
    g: 0.93,
    b: 0.93,
    a: 1.0,
};

const RED_ALERT_MODERN_LIGHT: Color = Color {
    r: 0.86,  // #dc2626 - 现代红色警告
    g: 0.15,
    b: 0.15,
    a: 1.0,
};

pub const MODERN_LIGHT_PALETTE_EXTENSION: PaletteExtension = PaletteExtension {
    is_nightly: false,
    font: SARASA_MONO_BOLD,
    font_headers: SARASA_MONO,
    alpha_chart_badge: 0.65,
    alpha_round_borders: 0.35,
    alpha_round_containers: 0.18,
    buttons_color: BUTTONS_MODERN_LIGHT,
    red_alert_color: RED_ALERT_MODERN_LIGHT,
};