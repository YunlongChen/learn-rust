use crate::gui::styles::style_constants::FONT_SIZE_FOOTER;
use crate::notifications::types::sounds::Sound::{Gulp, Pop, Swoosh};
use crate::utils::types::icon::Icon;
use crate::StyleType;
use iced::widget::Text;
use iced::{Alignment, Font, Length};
use serde::{Deserialize, Serialize};
use std::fmt;

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
    let mp3_sound = sound.mp3_sound();
    dbg!(mp3_sound.len());

    // let _ = thread::Builder::new()
    //     .name("thread_play_sound".to_string())
    //     .spawn(move || {
    //         // Get an output stream handle to the default physical sound device
    //         let Ok((_stream, stream_handle)) = OutputStream::try_default().log_err(location!())
    //         else {
    //             return;
    //         };
    //         let Ok(sink) = Sink::try_new(&stream_handle).log_err(location!()) else {
    //             return;
    //         };
    //         //load data
    //         let data = std::io::Cursor::new(mp3_sound);
    //         // Decode that sound file into a source
    //         let Ok(source) = Decoder::new(data).log_err(location!()) else {
    //             return;
    //         };
    //         // Play the sound directly on the device
    //         sink.set_volume(f32::from(volume) / 200.0); // set the desired volume
    //         sink.append(source);
    //         // The sound plays in a separate thread. This call will block the current thread until the sink
    //         // has finished playing all its queued sounds.
    //         sink.sleep_until_end();
    //     })
    //     .log_err(location!());
}
