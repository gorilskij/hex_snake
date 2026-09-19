//! Somewhere to keep a blob of text between runs.
//!
//! This is the whole of the platform split. On the web it is one `localStorage`
//! entry, reached through `quad-storage-sys` — miniquad's own JS interop rather
//! than wasm-bindgen, which would clash with macroquad's loader (the JS side
//! needs `sapp_jsutils.js` and `quad-storage.js`, see `web/index.html`).
//! Natively it is one file in the OS's config directory.
//!
//! Everything above this is platform-agnostic: callers hand over a string and
//! get one back. Losing it is not worth reporting — preferences that fail to
//! load are simply the default ones.

#[cfg(target_arch = "wasm32")]
mod imp {
    /// The `localStorage` key the blob lives under.
    const KEY: &str = "hex-snake-prefs";

    pub fn load() -> Option<String> {
        quad_storage_sys::get(KEY)
    }

    pub fn save(text: &str) {
        quad_storage_sys::set(KEY, text);
    }
}

#[cfg(not(target_arch = "wasm32"))]
mod imp {
    use std::fs;
    use std::path::PathBuf;

    use directories::ProjectDirs;

    /// `~/Library/Application Support/com.gorilskij.hex-snake/prefs` on macOS,
    /// `%APPDATA%\gorilskij\hex-snake\config\prefs` on Windows,
    /// `~/.config/hex-snake/prefs` on Linux.
    fn path() -> Option<PathBuf> {
        let dirs = ProjectDirs::from("com", "gorilskij", "hex-snake")?;
        Some(dirs.config_dir().join("prefs"))
    }

    pub fn load() -> Option<String> {
        fs::read_to_string(path()?).ok()
    }

    pub fn save(text: &str) {
        let Some(path) = path() else { return };
        if let Some(dir) = path.parent() {
            // the config directory does not exist until something writes there
            if fs::create_dir_all(dir).is_err() {
                return;
            }
        }
        let _ = fs::write(path, text);
    }
}

pub use imp::{load, save};
