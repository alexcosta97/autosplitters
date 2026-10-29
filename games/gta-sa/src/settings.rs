//! Which settings there are, in what order, and their defaults. Ported from the
//! `startup` block of `GTASA.asl`.
//!
//! Laid out like gta-sa-de's: headings at level 0 group related sections, and
//! each section is a heading followed by a dropdown for the checkboxes below
//! it. The ASL's defaults are kept: a setting only counts if its parents are
//! ticked too, so everything under a parent that is off by default (Races,
//! Import/Export, Properties, Collectibles, Other) is off.

use alloc::format;

use crate::data;
use gta_sa_common::names::{
    self, BL_MISSIONS, DESERT_MISSIONS, LS_MISSIONS, LV_MISSIONS, PROPERTIES, RTLS_MISSIONS,
    SF_MISSIONS,
};
use gta_sa_common::settings::{
    self,
    tree::{Builder, Tree},
    Settings,
};

// Mission givers of the big sections: heading and the address in
// `data::MISSIONS` counting their missions.
#[rustfmt::skip]
static LS_GIVERS: &[(&str, u32)] = &[
    ("Intro", 0x64A060), ("Sweet", 0x64A070), ("Big Smoke", 0x64A078), ("OG Loc", 0x64A07C),
    ("Cesar", 0x64A084), ("C.R.A.S.H.", 0x64A080), ("Ryder", 0x64A074), ("Finale", 0x64A088),
];
#[rustfmt::skip]
static SF_GIVERS: &[(&str, u32)] = &[
    ("Garage", 0x64A1D4), ("C.R.A.S.H.", 0x64A1E8), ("Syndicate", 0x64A1E4),
    ("Wu Zi Mu", 0x64A1DC),
];
#[rustfmt::skip]
static LV_GIVERS: &[(&str, u32)] = &[
    ("Toreno", 0x64A2A4), ("Casino", 0x64A2B4), ("Madd Dogg", 0x64A2BC),
    ("C.R.A.S.H.", 0x64A2B8),
];

// The ASL's mission counters of the asset missions.
const HEIST: u32 = 0x64A2C0;
const ZERO: u32 = 0x64A1D8;
const WANG_CARS: u32 = 0x64A1E0;
const TRUCKING: u32 = 0x6518DC;

/// The address in `data::MISSIONS` counting this mission.
fn giver(mission: &str) -> Option<u32> {
    data::MISSIONS
        .iter()
        .find(|(_, missions)| missions.iter().any(|(_, m)| *m == mission))
        .map(|(address, _)| *address)
}

fn add_mission_list(b: &mut Builder, missions: &[&str]) {
    for m in missions {
        b.check(m, None, true);
    }
}

/// Add the missions under a heading per mission giver.
fn add_by_giver(b: &mut Builder, section: &str, missions: &[&str], givers: &[(&str, u32)]) {
    for (label, address) in givers {
        b.heading(&format!("{section} {label}"), label, 2);
        for m in missions.iter().filter(|m| giver(m) == Some(*address)) {
            b.check(m, None, true);
        }
    }
}

/// Add missions from `data::MISSIONS`.
fn add_missions(b: &mut Builder, address: u32) {
    let (_, missions) = data::MISSIONS.iter().find(|(a, _)| *a == address).unwrap();
    for (_, m) in missions.iter() {
        b.check(m, None, true);
    }
}

/// Add missions from `data::MISSIONS2`.
fn add_missions2(b: &mut Builder, group: &str, default: bool) {
    for (_, m) in data::missions2(group) {
        b.check(m, None, default);
    }
}

pub fn build() -> Tree {
    let mut b = Builder::new();

    b.heading("General", "General", 0);
    b.check("start", Some("Start timer on New Game"), true);
    b.check("reset", Some("Reset timer on New Game"), true);
    b.check("startOnSaveLoad", Some("Start timer when loading save (experimental)"), false);
    b.tooltip(
        "This may start the timer too early on New Game, however if you have Reset enabled, \n it should reset again before the desired start.",
    );
    b.check("doubleSplitPrevention", Some("Double-Split Prevention"), true);
    b.tooltip(
        "Impose cooldown of 2.5s between auto-splits.\nThis may not work for all types of splits.",
    );

    // Main Missions
    //==============
    b.heading("Story missions", "Story missions", 0);
    b.tooltip("Main Missions and other splits that commonly occur in the any% route");
    b.section("LS", "Los Santos", 1, |b| add_by_giver(b, "LS", LS_MISSIONS, LS_GIVERS));
    b.section("BL", "Badlands", 1, |b| {
        b.check("Badlands", None, true);
        b.check("Bustedwarp BL #1", Some("[Bustedwarp Badlands #1]"), false);
        b.check("Tanker Commander", None, true);
        b.check("Deathwarp BL #1", Some("[Deathwarp Badlands #1]"), false);
        b.check("Body Harvest", None, true);
        b.check("Deathwarp BL #2", Some("[Deathwarp Badlands #2]"), false);
        b.check("King in Exile", None, true);
        b.check("Bustedwarp BL #2", Some("[Bustedwarp Badlands #2]"), false);
        let rest: alloc::vec::Vec<&str> = BL_MISSIONS
            .iter()
            .copied()
            .filter(|m| {
                !["Badlands", "Tanker Commander", "Body Harvest", "King in Exile"].contains(m)
            })
            .collect();
        add_mission_list(b, &rest);
    });
    b.section("SF", "San Fierro", 1, |b| {
        add_by_giver(b, "SF", SF_MISSIONS, SF_GIVERS);
        b.heading("SF Warps", "Warps", 2);
        b.check("Deathwarp SF", Some("[Deathwarp SF]"), true);
        b.tooltip("Splits when wasted between 'The Da Nang Thang' and 'Yay Ka-Boom-Boom'");
    });
    b.section("Desert", "Desert", 1, |b| add_mission_list(b, DESERT_MISSIONS));
    b.section("LV", "Las Venturas", 1, |b| add_by_giver(b, "LV", LV_MISSIONS, LV_GIVERS));
    b.section("RTLS", "Return to Los Santos", 1, |b| {
        add_mission_list(b, RTLS_MISSIONS);
        b.heading("RTLS Ending", "Ending", 2);
        b.check(
            "End of the Line Part 1",
            Some("End of the Line Part 1 (after killing Big Smoke)"),
            false,
        );
        b.check("End of the Line Part 2", Some("End of the Line Part 2 (start of Chase)"), false);
        b.check("End of the Line Part 3", Some("End of the Line Part 3 (after Credits)"), false);
        b.check("GT #1", Some("Gang Territories #1 (at starting of Grove 4 Life)"), false);
        b.check("GT #2", Some("Gang Territories #2 (at starting of Cut Throat Business)"), false);
        b.check("any%", Some("End of any% (start of Firetruck Bridge Cutscene)"), true);
    });

    // Side Missions
    //==============
    b.heading("Asset missions", "Asset missions", 0);
    b.section("HeistMissions", "Heist", 1, |b| add_missions(b, HEIST));
    b.section("ZeroMissions", "Zero", 1, |b| add_missions(b, ZERO));
    b.section("Wang CarsMissions", "Wang Cars", 1, |b| add_missions(b, WANG_CARS));
    b.section("TruckingMissions", "Trucking", 1, |b| add_missions(b, TRUCKING));
    b.section("Assets", "Other Asset Missions", 1, |b| add_missions2(b, "Assets", true));

    b.heading("Schools and vehicles", "Schools and vehicles", 0);
    b.section("Schools", "Schools", 1, |b| {
        add_missions2(b, "Schools", true);
        b.check("Driving School Started", None, false);
        b.tooltip("Splits when starting Driving School for the first time");
    });
    b.section("Vehicle Submissions", "Vehicle Submissions", 1, |b| {
        add_missions2(b, "Vehicle Submissions", true);
        add_mission_list(b, &["Freight Level 1", "Freight Level 2"]);
    });

    b.heading("Races and stadium", "Races and stadium", 0);
    b.section("Races", "Races", 1, |b| {
        b.check("All Races Won", None, false);
        b.section("LS Races", "Los Santos", 2, |b| add_missions2(b, "LS Races", false));
        b.section("SF Races", "San Fierro", 2, |b| add_missions2(b, "SF Races", false));
        b.section("LV Races", "Las Venturas", 2, |b| add_missions2(b, "LV Races", false));
        b.section("Air Races", "Air Races", 2, |b| add_missions2(b, "Air Races", false));
    });
    b.section("Stadium Events", "Stadium Events", 1, |b| add_missions2(b, "Stadium Events", true));

    b.heading("Challenges and gyms", "Challenges and gyms", 0);
    b.section("Challenges", "Challenges", 1, |b| {
        b.check("Chiliad Challenge #1", None, true);
        b.check("Chiliad Challenge #2", None, true);
        b.check("Chiliad Challenge #3", None, true);
        add_missions2(b, "Challenges", true);
    });
    b.section("Gym Moves", "Gym Moves", 1, |b| add_missions2(b, "Gym Moves", true));

    b.heading("Import/Export", "Import/Export", 0);
    b.section("Export Lists", "Import/Export", 1, |b| {
        for (i, list) in names::EXPORT_LISTS.iter().enumerate() {
            let n = i + 1;
            b.section(&format!("Export List {n}"), &format!("List {n}"), 2, |b| {
                for vehicle in list {
                    b.check(&format!("Export {vehicle}"), Some(vehicle), false);
                }
                let complete = format!("Export List {n} Complete");
                b.check(&complete, None, false);
            });
        }
    });

    b.heading("Property purchases", "Properties", 0);
    b.section("Properties", "Properties", 1, |b| {
        for (label, properties) in PROPERTIES {
            b.heading(&format!("Properties {label}"), label, 2);
            for p in *properties {
                b.check(p, None, false);
            }
        }
    });

    // Collectibles
    //=============
    b.heading("Collectibles group", "Collectibles", 0);
    b.section("Collectibles", "Collectibles", 1, |b| {
        for kind in names::COLLECTIBLE_KINDS {
            b.check(&format!("{kind}All"), Some(&format!("{kind} (All Done)")), false);
            b.check(&format!("{kind}Each"), Some(&format!("{kind} (Each)")), false);
        }
    });

    // Other
    //======
    b.heading("Other group", "Other", 0);
    b.section("Other", "Other", 1, |b| {
        b.check("Plane Flight", None, false);
        b.tooltip("Splits when entering the ticket machine marker for the first time");
    });

    b.finish()
}

pub fn register() -> Settings {
    settings::register(build(), "[GTASA Autosplitter]")
}

#[cfg(test)]
mod tests {
    use alloc::{collections::BTreeMap, format, vec::Vec};

    use super::*;
    use gta_sa_common::settings::tree::Kind;

    fn checkboxes(tree: &Tree) -> Vec<&str> {
        tree.widgets
            .iter()
            .filter(|w| matches!(w.kind, Kind::Bool { .. }))
            .map(|w| w.key.as_str())
            .collect()
    }

    fn dropdown(tree: &Tree, key: &str) -> &'static str {
        match tree.widgets.iter().find(|w| w.key == key).unwrap().kind {
            Kind::Choice { default, .. } => default,
            _ => unreachable!(),
        }
    }

    /// Every widget of the layout, one line each: `key<TAB>label<TAB>kind<TAB>tooltip`.
    #[test]
    fn widgets_match_snapshot() {
        let lines: Vec<alloc::string::String> = build()
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

    #[test]
    fn defaults_are_the_asls() {
        let tree = build();
        let state = tree.read(&BTreeMap::new());

        let mut off: Vec<alloc::string::String> = [
            "Bustedwarp BL #1",
            "Deathwarp BL #1",
            "Deathwarp BL #2",
            "Bustedwarp BL #2",
            "Driving School Started",
            "End of the Line Part 1",
            "End of the Line Part 2",
            "End of the Line Part 3",
            "GT #1",
            "GT #2",
            "startOnSaveLoad",
        ]
        .iter()
        .map(|k| (*k).into())
        .collect();
        for section in ["Races", "Export Lists", "Properties", "Collectibles", "Other"] {
            off.extend(tree.section(section).0.into_iter().map(Into::into));
        }
        assert!(off.len() > 100);

        for key in checkboxes(&tree) {
            let expected = !off.iter().any(|k| k == key);
            assert_eq!(tree.get(&state, key), Some(expected), "{key}");
        }

        for (key, expected) in [
            ("Deathwarp SF", true),
            ("All Races Won", false),
            ("Export List 2 Complete", false),
            ("HorseshoesEach", false),
            ("Quarry Complete", true),
            ("Freight Level 1", true),
            ("doubleSplitPrevention", true),
            ("start", true),
            ("reset", true),
        ] {
            assert_eq!(tree.get(&state, key), Some(expected), "{key}");
        }
    }

    #[test]
    fn dropdowns_match_their_checkboxes() {
        let tree = build();
        let state = tree.read(&BTreeMap::new());
        assert_eq!(tree.sync(None, &state), state);
        assert_eq!(dropdown(&tree, "Races#section"), "none");
        assert_eq!(dropdown(&tree, "Export Lists#section"), "none");
        assert_eq!(dropdown(&tree, "Collectibles#section"), "none");
        assert_eq!(dropdown(&tree, "LS#section"), "all");
        assert_eq!(dropdown(&tree, "BL#section"), "selection");
    }

    #[test]
    fn every_split_has_a_checkbox() {
        let tree = build();
        let boxes = checkboxes(&tree);
        let once = |name: &str| {
            assert_eq!(boxes.iter().filter(|k| **k == name).count(), 1, "{name}");
        };

        for (_, missions) in data::MISSIONS {
            missions.iter().for_each(|(_, m)| once(m));
        }
        for (_, missions) in data::MISSIONS2 {
            missions.iter().for_each(|(_, m)| once(m));
        }
        data::START_MISSIONS.iter().for_each(|(_, m)| once(m));
        for name in [
            "Deathwarp SF",
            "Deathwarp BL #1",
            "Deathwarp BL #2",
            "Bustedwarp BL #1",
            "Bustedwarp BL #2",
            "any%",
            "Chiliad Challenge #1",
            "Chiliad Challenge #2",
            "Chiliad Challenge #3",
        ] {
            once(name);
        }
        for (i, list) in names::EXPORT_LISTS.iter().enumerate() {
            for vehicle in list {
                once(&format!("Export {vehicle}"));
            }
            once(&format!("Export List {} Complete", i + 1));
        }
        for kind in names::COLLECTIBLE_KINDS {
            once(&format!("{kind}All"));
            once(&format!("{kind}Each"));
        }
    }

    #[test]
    fn story_missions_are_asl_missions() {
        for list in
            [LS_MISSIONS, BL_MISSIONS, SF_MISSIONS, DESERT_MISSIONS, LV_MISSIONS, RTLS_MISSIONS]
        {
            for m in list {
                let known = data::MISSIONS.iter().any(|(_, ms)| ms.iter().any(|(_, n)| n == m));
                assert!(known, "{m}");
            }
        }
    }

    #[test]
    fn widget_keys_are_unique() {
        let tree = build();
        let mut keys: Vec<&str> = tree.widgets.iter().map(|w| w.key.as_str()).collect();
        let count = keys.len();
        keys.sort();
        keys.dedup();
        assert_eq!(keys.len(), count);
    }

    #[test]
    fn no_all_done_offered() {
        assert!(build().all_done_sections().is_empty());
    }
}
