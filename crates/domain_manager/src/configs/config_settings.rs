//! Module defining the `ConfigSettings` struct, which allows to save and reload
//! the application default configuration.

use crate::gui::styles::types::gradient_type::GradientType;
use crate::translations::types::language::Language;
use serde::{Deserialize, Serialize};

use crate::{StyleType, DOMAIN_MANAGER_LOWERCASE};

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct ConfigSettings {
    pub color_gradient: GradientType,
    pub language: Language,
    pub scale_factor: f64,
    pub mmdb_country: String,
    pub mmdb_asn: String,
    pub style_path: String,
    // pub notifications: Notifications,
    // StyleType should be last in order to deserialize as a table properly
    pub style: StyleType,
}

impl ConfigSettings {
    const FILE_NAME: &'static str = "settings";

    #[cfg(not(test))]
    pub fn load() -> Self {
        if let Ok(settings) =
            confy::load::<ConfigSettings>(DOMAIN_MANAGER_LOWERCASE, Self::FILE_NAME)
        {
            settings
        } else {
            dbg!("测试空间：「{}」", DOMAIN_MANAGER_LOWERCASE);
            // let _ = confy::store(
            //     DOMAIN_MANAGER_LOWERCASE,
            //     Self::FILE_NAME,
            //     ConfigSettings::default(),
            // )
            // .log_err(location!());
            ConfigSettings::default()
        }
    }

    #[cfg(not(test))]
    pub fn store(self) {
        // let _ = confy::store(DOMAIN_MANAGER_LOWERCASE, Self::FILE_NAME, self).log_err(location!());
    }
}

impl Default for ConfigSettings {
    fn default() -> Self {
        ConfigSettings {
            color_gradient: GradientType::default(),
            language: Language::default(),
            scale_factor: 1.0,
            mmdb_country: String::new(),
            mmdb_asn: String::new(),
            style_path: String::new(),
            // notifications: Notifications::default(),
            style: StyleType::default(),
        }
    }
}
