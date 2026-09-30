//! Settings ported from the `startup` block of `GTA_SA.asl`.
//!
//! ASL settings are nested: a setting only counts as enabled if all of its
//! parents are enabled too. The auto splitting runtime only has a flat list of
//! widgets (which can't be hidden or shown conditionally), so here each
//! checkbox decides on its own, and each section has a dropdown that ticks or
//! unticks the checkboxes below it and shows what they add up to. The layout is
//! in `layout`, the syncing in `tree`; this is the part talking to the runtime.

mod layout;
mod tree;

use alloc::{format, string::String};
use asr::settings::{gui, Map};

use tree::{Kind, Source, State, Tree, Value};

impl Source for Map {
    fn get_bool(&self, key: &str) -> Option<bool> {
        self.get(key)?.get_bool()
    }

    fn get_string(&self, key: &str) -> Option<String> {
        self.get(key)?.get_string()
    }
}

pub struct Settings {
    tree: Tree,
    values: State,
    /// The values after the previous sync, to tell what the user changed.
    baseline: Option<State>,
}

impl Settings {
    /// Equivalent of `settings[key]` in ASL.
    pub fn get(&self, key: &str) -> bool {
        self.tree.get(&self.values, key).unwrap_or_else(|| {
            asr::print_message(&format!("[GTASA:DE Autosplitter] Unknown setting: {key}"));
            false
        })
    }

    /// Reload the values the user has set and sync the section dropdowns with
    /// their checkboxes. Call once per tick.
    pub fn update(&mut self) {
        loop {
            let map = Map::load();
            let current = self.tree.read(&map);
            let synced = self.tree.sync(self.baseline.as_ref(), &current);
            let changes = self.tree.changes(&current, &synced);
            if !changes.is_empty() {
                let new = map.clone();
                for (key, value) in changes {
                    match value {
                        Value::Bool(b) => new.insert(key, b),
                        Value::Choice(c) => new.insert(key, c),
                    }
                }
                // The user changed something in the meantime, sync again.
                if !new.store_if_unchanged(&map) {
                    continue;
                }
            }
            // What was stored is the new baseline, so the next tick doesn't
            // take these writes for the user's.
            self.baseline = Some(synced.clone());
            self.values = synced;
            return;
        }
    }
}

pub fn register() -> Settings {
    let tree = layout::build();
    for w in &tree.widgets {
        match w.kind {
            Kind::Title { level } => gui::add_title(&w.key, &w.label, level),
            Kind::Bool { default } => {
                gui::add_bool(&w.key, &w.label, default);
            }
            Kind::Choice { options, default } => {
                gui::add_choice(&w.key, &w.label, default);
                for (option, label) in options {
                    gui::add_choice_option(&w.key, option, label);
                }
            }
        }
        if let Some(tooltip) = w.tooltip {
            gui::set_tooltip(&w.key, tooltip);
        }
    }

    let values = tree.read(&Map::load());
    let mut s = Settings { tree, values, baseline: None };
    s.update();
    s
}
