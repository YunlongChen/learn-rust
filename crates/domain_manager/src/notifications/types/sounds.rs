use crate::gui::styles::style_constants::FONT_SIZE_FOOTER;
use crate::notifications::types::sounds::Sound::{Gulp, Pop, Swoosh};
use crate::utils::types::icon::Icon;
use crate::StyleType;
use iced::widget::Text;
use iced::{Alignment, Font, Length};
// use rodio::{Decoder, OutputStream, Sink};
use serde::{Deserialize, Serialize};
use std::fmt;
// use tracing::info;

/// Enum representing the possible notification sounds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Sound {
    Gulp,
    Pop,
    Swoosh,
    None,
}

pub const GULP: &[u8] = include_bytes!("../../../resources/sounds/gulp.mp3");
pub const POP: &[u8] = include_bytes!("../../../resources/sounds/pop.mp3");
pub const SWOOSH: &[u8] = include_bytes!("../../../resources/sounds/swoosh.mp3");

impl fmt::Display for Sound {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl Sound {
    pub(crate) const ALL: [Sound; 4] = [Gulp, Pop, Swoosh, Sound::None];

    fn mp3_sound(self) -> &'static [u8] {
        match self {
            Gulp => GULP,
            Pop => POP,
            Swoosh => SWOOSH,
            Sound::None => &[],
        }
    }

    pub fn get_text<'a>(self, font: Font) -> Text<'a, StyleType> {
        match self {
            Gulp => Text::new("Gulp").font(font),
            Pop => Text::new("Pop").font(font),
            Swoosh => Text::new("Swoosh").font(font),
            Sound::None => Icon::Forbidden.to_text(),
        }
        .size(FONT_SIZE_FOOTER)
        .width(Length::Fill)
        .align_x(Alignment::Center)
        .align_y(Alignment::Center)
    }
}

pub fn play(sound: Sound, volume: u8) {
    if sound.eq(&Sound::None) || volume == 0 {
        return;
    }
    let _mp3_sound = sound.mp3_sound();
    // info!("{}", mp3_sound.len());
}
