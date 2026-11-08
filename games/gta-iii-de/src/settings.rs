//! Settings ported from the `startup` block of the ASL. The layout is in
//! `layout`; the syncing and the part talking to the runtime are shared, in
//! `gta_sa_common::settings`.

mod layout;

pub use gta_sa_common::settings::Settings;

pub fn register() -> Settings {
    gta_sa_common::settings::register(layout::build(), "[GTAIII:DE Autosplitter]")
}
