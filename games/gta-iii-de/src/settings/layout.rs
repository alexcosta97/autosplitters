//! Which settings there are, in what order, and their defaults. Ported from the
//! `startup` block of the ASL.
//!
//! Headings at level 0 group related sections. Each mission giver is a heading
//! followed by a dropdown for the checkboxes below it (see `tree`). The ones
//! under "Missions (end)" also offer "All done".

use crate::data::{COLLECTIBLES, MISSIONS};
use crate::splits::{end_key, start_key};
use gta_sa_common::settings::tree::{Builder, Tree};

/// The mission whose start isn't a setting, "don't need this" in the ASL.
const NO_START: &str = "Give Me Liberty & Luigi's Girls";

pub fn build() -> Tree {
    let mut b = Builder::new();

    // Timer control. LiveSplit has its own Start/Reset checkboxes for ASL
    // scripts, LiveSplit One leaves that up to the auto splitter.
    b.heading("General", "General", 0);
    b.check("start", Some("Start timer on New Game"), true);
    b.check("reset", Some("Reset timer on New Game"), true);

    // Splitting on a mission being passed, one section per mission giver.
    b.heading("missions_end", "Missions (end)", 0);
    for (giver, missions) in MISSIONS {
        b.section_all_done(&end_key(giver), giver, 1, |b| {
            for (_, mission) in *missions {
                b.check(&end_key(mission), Some(mission), true);
            }
        });
    }

    // Splitting on a mission starting.
    b.heading("missions_start", "Missions (start)", 0);
    for (giver, missions) in MISSIONS {
        b.section(&start_key(giver), giver, 1, |b| {
            for (_, mission) in missions.iter().filter(|(_, m)| *m != NO_START) {
                b.check(&start_key(mission), Some(mission), false);
            }
        });
    }

    b.heading("collectibles", "Collectibles", 0);
    for c in COLLECTIBLES {
        b.collectible(c.kind);
    }

    b.heading("Other", "Other", 0);
    b.check("any_end", Some("Any% Final Split"), true);
    b.tooltip("Splits once you lose control on \"The Exchange\".");

    b.finish()
}

#[cfg(test)]
mod tests {
    use alloc::{collections::BTreeMap, format, string::String, vec::Vec};

    use super::*;
    use crate::data::SCRIPTS;
    use crate::splits::all_done_key;
    use gta_sa_common::settings::tree::Kind;

    #[test]
    fn widget_keys_are_unique() {
        let tree = build();
        let mut keys: Vec<&str> = tree.widgets.iter().map(|w| w.key.as_str()).collect();
        let count = keys.len();
        keys.sort();
        keys.dedup();
        assert_eq!(keys.len(), count);
    }

    /// Every key the splitter reads is in the tree, with the ASL's default.
    #[test]
    fn every_split_key_exists() {
        let tree = build();
        let state = tree.read(&BTreeMap::new());
        let get = |key: &str| tree.get(&state, key);

        assert_eq!(get("start"), Some(true));
        assert_eq!(get("reset"), Some(true));
        assert_eq!(get("any_end"), Some(true));
        for (giver, missions) in MISSIONS {
            assert_eq!(get(&all_done_key(giver)), Some(false), "{giver}");
            for (_, mission) in *missions {
                assert_eq!(get(&end_key(mission)), Some(true), "{mission}");
                let start = if *mission == NO_START { None } else { Some(false) };
                assert_eq!(get(&start_key(mission)), start, "{mission}");
            }
        }
        for (_, mission) in SCRIPTS {
            assert_eq!(get(&start_key(mission)), Some(false), "{mission}");
        }
        for c in COLLECTIBLES {
            assert_eq!(get(&format!("{}All", c.kind)), Some(false), "{}", c.kind);
            assert_eq!(get(&format!("{}Each", c.kind)), Some(false), "{}", c.kind);
        }
        // Nothing to correct or store at startup.
        assert_eq!(tree.sync(None, &state), state);
    }

    #[test]
    fn every_mission_is_listed_once() {
        let tree = build();
        let checkboxes: Vec<&str> = tree
            .widgets
            .iter()
            .filter(|w| matches!(w.kind, Kind::Bool { .. }))
            .map(|w| w.key.as_str())
            .collect();
        let missions: usize = MISSIONS.iter().map(|(_, ms)| ms.len()).sum();
        // The two timer settings and the Any% split are the others.
        assert_eq!(checkboxes.len(), missions + (missions - 1) + 3);
    }

    #[test]
    fn all_done_is_offered_for_every_giver_under_missions_end() {
        let tree = build();
        let mut offered = tree.all_done_sections();
        offered.sort();
        let mut expected: Vec<String> = MISSIONS.iter().map(|(g, _)| end_key(g)).collect();
        expected.sort();
        assert_eq!(offered, expected);

        for (giver, missions) in MISSIONS {
            let (boxes, _) = tree.section(&end_key(giver));
            let mut expected: Vec<String> = missions.iter().map(|(_, m)| end_key(m)).collect();
            expected.sort();
            let mut boxes: Vec<&str> = boxes;
            boxes.sort();
            assert_eq!(boxes, expected, "{giver}");
        }
    }

    #[test]
    fn collectibles_default_to_off() {
        let tree = build();
        for c in COLLECTIBLES {
            let key = format!("{}#splits", c.kind);
            let widget = tree.widgets.iter().find(|w| w.key == key).unwrap();
            assert!(matches!(widget.kind, Kind::Choice { default: "off", .. }), "{key}");
        }
    }

    /// Every widget of the layout, one line each: `key<TAB>label<TAB>kind<TAB>tooltip`.
    #[test]
    fn widgets_match_snapshot() {
        let lines: Vec<String> = build()
            .widgets
            .iter()
            .map(|w| {
                let kind = match &w.kind {
                    Kind::Title { level } => format!("title:{level}"),
                    Kind::Bool { default } => format!("bool:{default}"),
                    Kind::Choice { options, default } => {
                        let keys: Vec<&str> = options.iter().map(|(k, _)| *k).collect();
                        format!("choice:{default}:{}", keys.join(","))
                    }
                };
                format!("{}\t{}\t{kind}\t{}", w.key, w.label, w.tooltip.unwrap_or(""))
            })
            .collect();
        assert_eq!(lines.join("\n"), include_str!("widgets.snap").trim_end_matches('\n'));
    }
}
