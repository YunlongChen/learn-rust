use crate::notifications::types::sounds::Sound;
use serde::{Deserialize, Serialize};

/// Used to contain the notifications configuration set by the user
#[derive(Clone, Serialize, Deserialize, Copy, PartialEq, Debug)]
pub struct Notifications {
    pub volume: u8,
    // pub packets_notification: PacketsNotification,
    // pub bytes_notification: BytesNotification,
    pub favorite_notification: FavoriteNotification,
}

impl Default for Notifications {
    fn default() -> Self {
        Notifications {
            volume: 60,
            // packets_notification: PacketsNotification::default(),
            // bytes_notification: BytesNotification::default(),
            favorite_notification: FavoriteNotification::default(),
        }
    }
}

#[derive(Clone, Eq, PartialEq, Serialize, Deserialize, Debug, Copy)]
pub struct FavoriteNotification {
    /// Flag to determine if this notification is enabled
    pub notify_on_favorite: bool,
    /// The sound to emit
    pub sound: Sound,
}

impl Default for FavoriteNotification {
    fn default() -> Self {
        FavoriteNotification {
            notify_on_favorite: false,
            sound: Sound::Swoosh,
        }
    }
}

impl FavoriteNotification {
    /// Constructor when the notification is in use
    pub fn on(sound: Sound) -> Self {
        FavoriteNotification {
            notify_on_favorite: true,
            sound,
        }
    }

    /// Constructor when the notification is not in use. Note that sound is used here for caching, although it won't actively be used.
    pub fn off(sound: Sound) -> Self {
        FavoriteNotification {
            notify_on_favorite: false,
            sound,
        }
    }
}
