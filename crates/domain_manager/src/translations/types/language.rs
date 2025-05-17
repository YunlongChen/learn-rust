use crate::countries::flags_pictures::{
    CN, DE, ES, FI, FLAGS_WIDTH_BIG, FR, GB, GR, ID, IT, JP, KR, PL, PT, RO, RU, SE, TR, TW, UA,
    UZ, VN,
};
use crate::StyleType;
use iced::widget::svg::Handle;
use iced::widget::Svg;
use serde::{Deserialize, Serialize};
use std::fmt;

/// This enum defines the available languages.
#[derive(PartialEq, Eq, Clone, Copy, Debug, Serialize, Deserialize, Hash, Default)]
pub enum Language {
    #[default]
    /// 中文
    ZH,
    /// 繁体中文
    #[allow(non_camel_case_types)]
    ZH_TW,
    /// 英语.
    EN,
    /// 意大利语.
    IT,
    /// 法语.
    FR,
    /// 西班牙语.
    ES,
    /// 波兰语.
    PL,
    /// 德语,
    DE,
    /// 乌克兰语
    UK,
    /// 罗马语
    RO,
    /// 韩语
    KO,
    /// 葡萄牙语
    PT,
    /// 土耳其语
    TR,
    /// 俄罗斯语
    RU,
    /// 希腊语
    EL,
    // /// 波斯语
    // FA,
    /// 瑞典语
    SV,
    /// 芬兰语
    FI,
    /// 日语
    JA,
    /// 乌兹别克语
    UZ,
    /// 越南语
    VI,
    /// 印度尼西亚语
    ID,
}

impl Language {
    pub const ALL: [Language; 21] = [
        Language::EN,
        Language::DE,
        Language::EL,
        Language::ES,
        Language::FI,
        Language::FR,
        Language::ID,
        Language::IT,
        Language::JA,
        Language::KO,
        Language::PL,
        Language::PT,
        Language::RO,
        Language::RU,
        Language::SV,
        Language::TR,
        Language::UK,
        Language::UZ,
        Language::VI,
        Language::ZH,
        Language::ZH_TW,
    ];

    pub fn get_flag<'a>(self) -> Svg<'a, StyleType> {
        Svg::new(Handle::from_memory(Vec::from(match self {
            Language::ZH => CN,
            Language::ZH_TW => TW,
            Language::DE => DE,
            Language::ES => ES,
            Language::FR => FR,
            Language::EN => GB,
            Language::IT => IT,
            Language::KO => KR,
            Language::PL => PL,
            Language::PT => PT,
            Language::RO => RO,
            Language::RU => RU,
            Language::TR => TR,
            Language::UK => UA,
            Language::EL => GR,
            // Language::FA => IR,
            Language::SV => SE,
            Language::FI => FI,
            Language::JA => JP,
            Language::UZ => UZ,
            Language::VI => VN,
            Language::ID => ID,
        })))
            .width(FLAGS_WIDTH_BIG)
    }

    pub fn is_up_to_date(self) -> bool {
        matches!(
            self,
            Language::FR
                | Language::EN
                | Language::IT
                | Language::DE
                | Language::PL
                | Language::RU
                | Language::RO
                | Language::JA
                | Language::UZ
                | Language::SV
                | Language::VI
                | Language::ZH
                | Language::ZH_TW
                | Language::KO
                | Language::TR
                | Language::PT
                | Language::UK
                | Language::ID
        )
    }
}

impl fmt::Display for Language {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let lang_str = match self {
            Language::EN => "English",
            Language::IT => "Italiano",
            Language::FR => "Français",
            Language::ES => "Español",
            Language::PL => "Polski",
            Language::DE => "Deutsch",
            Language::UK => "Українська",
            Language::ZH => "简体中文",
            Language::ZH_TW => "繁體中文",
            Language::RO => "Română",
            Language::KO => "한국어",
            Language::TR => "Türkçe",
            Language::RU => "Русский",
            Language::PT => "Português",
            Language::EL => "Ελληνικά",
            // Language::FA => "فارسی",
            Language::SV => "Svenska",
            Language::FI => "Suomi",
            Language::JA => "日本語",
            Language::UZ => "O'zbekcha",
            Language::VI => "Tiếng Việt",
            Language::ID => "Bahasa Indonesia",
        };
        write!(f, "{self:?} - {lang_str}")
    }
}
