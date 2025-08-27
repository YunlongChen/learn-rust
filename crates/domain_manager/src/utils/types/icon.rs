use iced::widget::Text;

use crate::gui::styles::style_constants::ICONS;
use crate::StyleType;

pub enum Icon {
    ArrowBack,
    ArrowLeft,
    ArrowRight,
    ArrowsDown,
    AudioHigh,
    AudioMute,
    Bin,
    Book,
    BytesThreshold,
    Clock,
    Copy,
    Generals,
    Error,
    Feedback,
    File,
    Forbidden,
    Funnel,
    GitHub,
    Language,
    HalfSun,
    Hourglass1,
    Hourglass2,
    Hourglass3,
    Image,
    Inspect,
    Moon,
    News,
    Notification,
    OpenLink,
    Overview,
    PacketsThreshold,
    Restore,
    Rocket,
    Settings,
    DomainManager,
    SortAscending,
    SortDescending,
    SortNeutral,
    Star,
    Sun,
    ThumbnailOpen,
    ThumbnailClose,
    Update,
    Warning,
    Waves,
    Sync,
    Add,
    /// 最小化窗口图标
    Minimize,
    /// 最大化窗口图标
    Maximize,
    /// 铃铛图标
    Bell,
    /// 导出图标
    Export,
    /// 导入图标
    Import,
}

impl Icon {
    pub fn codepoint(&self) -> char {
        match self {
            Icon::ArrowBack => 'C',
            Icon::ArrowLeft => 'i',
            Icon::ArrowRight => 'j',
            Icon::ArrowsDown => ':',
            Icon::AudioHigh => 'Z',
            Icon::AudioMute => 'Y',
            Icon::Bin => 'h',
            Icon::BytesThreshold => 'f',
            Icon::Clock => '9',
            Icon::Generals => 'Q',
            Icon::Error => 'U',
            Icon::File => '8',
            Icon::Forbidden => 'x',
            Icon::Funnel => 'V',
            Icon::GitHub => 'H',
            Icon::Language => 'c',
            Icon::HalfSun => 'K',
            Icon::Hourglass1 => '1',
            Icon::Hourglass2 => '2',
            Icon::Hourglass3 => '3',
            Icon::Image => 'I',
            Icon::Inspect => '5',
            Icon::Moon => 'G',
            Icon::Notification => '7',
            Icon::Overview => 'O',
            Icon::PacketsThreshold => 'e',
            Icon::Restore => 'k',
            Icon::Rocket => 'S',
            Icon::Settings => 'a',
            Icon::DomainManager => 'A',
            Icon::Star => 'g',
            Icon::Warning => 'T',
            Icon::Waves => 'y',
            Icon::Copy => 'u',
            Icon::SortAscending => 'm',
            Icon::SortDescending => 'l',
            Icon::SortNeutral => 'n',
            Icon::Sun => 'F',
            Icon::OpenLink => 'o',
            Icon::ThumbnailOpen => 's',
            Icon::ThumbnailClose => 'r',
            Icon::Book => 'B',
            Icon::Feedback => '=',
            Icon::News => '>',
            Icon::Update => '<',
            Icon::Sync => 'D',
            Icon::Add => 'P',
            Icon::Minimize => 'M',
            Icon::Maximize => 'N',
            Icon::Bell => 'b',
            Icon::Export => 'E',
            Icon::Import => 'J',
        }
    }

    pub fn to_text<'a>(&self) -> Text<'a, StyleType> {
        Text::new(self.codepoint().to_string()).font(ICONS)
    }

    // pub fn get_hourglass<'a>(num: usize) -> Text<'a, StyleType> {
    //     match num {
    //         1 => Icon::Hourglass1.to_text(),
    //         2 => Icon::Hourglass2.to_text(),
    //         3 => Icon::Hourglass3.to_text(),
    //         _ => Text::new(""),
    //     }
    // }
}
