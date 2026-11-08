//! Settings ported from the `startup` block of the ASL. The layout is in
//! `layout`; the syncing and the part talking to the runtime are shared, in
//! `autosplitter_utils::settings`.

mod layout;

pub use autosplitter_utils::settings::Settings;

pub fn register() -> Settings {
    autosplitter_utils::settings::register(layout::build(), "[GTAIII:DE Autosplitter]")
}
