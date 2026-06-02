use arboard::Clipboard;
use std::sync::Mutex;

static CLIPBOARD: Mutex<Option<Clipboard>> = Mutex::new(None);

/// Copies text to the system clipboard
pub fn copy_to_clipboard(text: &str) -> Result<(), String> {
    let mut clipboard = CLIPBOARD.lock().map_err(|e| e.to_string())?;

    if let Some(ref mut cb) = *clipboard {
        cb.set_text(text).map_err(|e| e.to_string())?;
    } else {
        let mut cb = Clipboard::new().map_err(|e| e.to_string())?;
        cb.set_text(text).map_err(|e| e.to_string())?;
        *clipboard = Some(cb);
    }

    Ok(())
}
