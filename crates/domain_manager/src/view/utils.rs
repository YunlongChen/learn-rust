#![allow(unused)]
use iced::Color;

pub fn transparent() -> Color {
    Color::from_rgba8(0, 0, 0, 0.0)
}

pub fn black() -> Color {
    Color::from_rgba8(0, 0, 0, 1.0)
}

pub fn white() -> Color {
    Color::from_rgba8(255, 255, 255, 1.0)
}

pub fn cyan() -> Color {
    Color::from_rgba8(224, 255, 255, 1.0)
}

pub fn blue() -> Color {
    Color::from_rgb8(3, 138, 255)
}