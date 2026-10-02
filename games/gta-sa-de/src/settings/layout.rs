//! Which settings there are, in what order, and their defaults. Ported from the
//! `startup` block of `GTA_SA.asl`, regrouped so the list is easier to scan.
//!
//! Headings at level 0 group related sections. Each section (Los Santos,
//! Races, Properties, ...) is a heading followed by a dropdown for the
//! checkboxes below it (see `tree`). Sections with many settings are split up
//! further by plain level 2 headings. The sections in `completion::SECTIONS`
//! and the Import/Export lists also offer "All done".

use alloc::format;

use super::tree::{Builder, Tree};
use crate::data::{self, TRUCKING};

// Story missions, in the order the ASL listed them.
#[rustfmt::skip]
static LS_MISSIONS: &[&str] = &[
    "Big Smoke", "Ryder", "Tagging up Turf", "Cleaning the Hood", "Drive-Thru",
    "Nines and AKs", "OG Loc", "Life's a Beach", "Running Dog", "Drive-By",
    "Sweet's Girl", "Cesar Vialpando", "High Stakes Lowrider", "Madd Dogg's Rhymes",
    "Management Issues", "House Party (Cutscene)", "Burning Desire",
    "Wrong Side of the Tracks", "Just Business", "Doberman", "Gray Imports",
    "Home Invasion", "House Party", "Catalyst", "Robbing Uncle Sam", "Los Sepulcros",
    "Reuniting the Families", "The Green Sabre",
];
#[rustfmt::skip]
static BL_MISSIONS: &[&str] = &[
    "Badlands", "Tanker Commander", "Body Harvest", "King in Exile",
    "Small Town Bank", "Local Liquor Store", "Against All Odds", "Wu Zi Mu",
    "Farewell, My Love", "Are You Going To San Fierro?",
];
#[rustfmt::skip]
static SF_MISSIONS: &[&str] = &[
    "Wear Flowers in your Hair", "555 WE TIP", "Deconstruction", "Photo Opportunity",
    "Jizzy (Cutscene)", "Jizzy", "T-Bone Mendez", "Mike Toreno", "Outrider",
    "Snail Trail", "Mountain Cloud Boys", "Ran Fa Li", "Lure", "Ice Cold Killa",
    "Amphibious Assault", "Pier 69", "Toreno's Last Flight", "The Da Nang Thang",
    "Yay Ka-Boom-Boom",
];
#[rustfmt::skip]
static DESERT_MISSIONS: &[&str] = &[
    "Monster", "Highjack", "Interdiction", "Verdant Meadows", "Learning to Fly",
];
#[rustfmt::skip]
static LV_MISSIONS: &[&str] = &[
    "N.O.E.", "Freefall", "Fender Ketchup", "Explosive Situation", "You've Had Your Chips",
    "Don Peyote", "Intensive Care", "The Meat Business", "Fish in a Barrel", "Madd Dogg",
    "Misappropriation", "Stowaway", "Black Project", "High Noon", "Green Goo",
    "Saint Mark's Bistro",
];
#[rustfmt::skip]
static RTLS_MISSIONS: &[&str] = &[
    "A Home in the Hills", "Vertical Bird", "Home Coming", "Beat Down on B Dup",
    "Grove 4 Life", "Cut Throat Business", "Riot", "Los Desperados",
];

// Mission givers of the big sections: heading and the `data::MISSIONS` global
// counting their missions. Ordered by the first of their missions in the lists
// above.
#[rustfmt::skip]
static LS_GIVERS: &[(&str, u32)] = &[
    ("Intro", 450), ("Sweet", 454), ("Big Smoke", 456), ("OG Loc", 457),
    ("Cesar", 459), ("C.R.A.S.H.", 458), ("Ryder", 455), ("Finale", 460),
];
#[rustfmt::skip]
static SF_GIVERS: &[(&str, u32)] = &[
    ("Garage", 543), ("C.R.A.S.H.", 548), ("Syndicate", 547), ("Wu Zi Mu", 545),
];
#[rustfmt::skip]
static LV_GIVERS: &[(&str, u32)] = &[
    ("Toreno", 595), ("Casino", 599), ("Madd Dogg", 601), ("C.R.A.S.H.", 600),
];

#[rustfmt::skip]
static PROPERTIES: &[(&str, &[&str])] = &[
    ("Businesses", &[
        "Zero (RC Shop Bought)", "Wang Cars (Showroom Bought)", "Verdant Meadows (Safehouse)",
    ]),
    ("Los Santos safehouses", &[
        "Santa Maria Beach (Safehouse)", "Mulholland (Safehouse)", "Verona Beach (Safehouse)",
        "Willowfield (Safehouse)", "Jefferson (Safehouse)", "Verdant Bluffs (Safehouse)",
    ]),
    ("San Fierro safehouses", &[
        "Calton Heights (Safehouse)", "Paradiso (Safehouse)", "Hashbury (Safehouse)",
        "Chinatown (Safehouse)", "Doherty (Safehouse)",
    ]),
    ("Las Venturas safehouses", &[
        "Rockshore West (Safehouse)", "Prickle Pine (Safehouse)", "Whitewood Estate (Safehouse)",
        "Redsands West (Safehouse)", "Creek (Safehouse)",
    ]),
    ("Countryside safehouses", &[
        "Palomino Creek (Safehouse)", "Blueberry (Safehouse)", "Dillimore (Safehouse)",
        "Angel Pine (Safehouse)", "Whetstone (Safehouse)",
    ]),
    ("Desert safehouses", &[
        "Fort Carson (Safehouse)", "El Quebrados (Safehouse)", "Tierra Robada (Safehouse)",
    ]),
    ("Hotel suites", &[
        "Pirates In Men's Pants (Hotel Suite)", "The Camel's Toe (Hotel Suite)",
        "Queens (Hotel Suite)", "Old Venturas Strip (Hotel Suite)",
        "The Clown's Pocket (Hotel Suite)",
    ]),
];

/// The global of `data::MISSIONS` counting this mission.
fn giver(mission: &str) -> Option<u32> {
    data::MISSIONS
        .iter()
        .find(|(_, missions)| missions.iter().any(|(_, m)| *m == mission))
        .map(|(index, _)| *index)
}

fn add_mission_list(b: &mut Builder, missions: &[&str]) {
    for m in missions {
        b.check(m, None, true);
    }
}

/// Add the missions under a heading per mission giver.
fn add_by_giver(b: &mut Builder, section: &str, missions: &[&str], givers: &[(&str, u32)]) {
    for (label, index) in givers {
        b.heading(&format!("{section} {label}"), label, 2);
        for m in missions.iter().filter(|m| giver(m) == Some(*index)) {
            b.check(m, None, true);
        }
    }
}

/// Add missions from `data::MISSIONS`.
fn add_missions(b: &mut Builder, index: u32) {
    let (_, missions) = data::MISSIONS.iter().find(|(i, _)| *i == index).unwrap();
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

    // Timer control. LiveSplit has its own Start/Reset checkboxes for ASL
    // scripts, LiveSplit One leaves that up to the auto splitter.
    b.heading("General", "General", 0);
    b.check("start", Some("Start timer on New Game"), true);
    b.check("reset", Some("Reset timer on New Game"), true);
    // Off by default: reloading a save during a run would reset it.
    b.check("startOnSaveLoad", Some("Start timer on loading a save"), false);
    b.tooltip("For practising part of a run from a save");
    b.check("resetOnSaveLoad", Some("Reset timer on loading a save"), false);
    b.tooltip("Starts it again if \"Start timer on loading a save\" is on");
    b.check("doubleSplitPrevention", Some("Double-Split Prevention"), true);
    b.tooltip(
        "Impose cooldown of 2.5s between auto-splits.\nThis may not work for all types of splits.",
    );

    // Main Missions
    //==============
    b.heading("Story missions", "Story missions", 0);
    b.tooltip("Main Missions and other splits that commonly occur in the any% route");
    b.section("LS", "Los Santos", 1, |b| add_by_giver(b, "LS", LS_MISSIONS, LS_GIVERS));
    b.section("BL", "Badlands", 1, |b| add_mission_list(b, BL_MISSIONS));
    b.section("SF", "San Fierro", 1, |b| add_by_giver(b, "SF", SF_MISSIONS, SF_GIVERS));
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
    b.section_all_done("HeistMissions", "Heist", 1, |b| add_missions(b, 602));
    b.section_all_done("ZeroMissions", "Zero", 1, |b| add_missions(b, 544));
    b.section_all_done("Wang CarsMissions", "Wang Cars", 1, |b| add_missions(b, 546));
    b.section_all_done("TruckingMissions", "Trucking", 1, |b| add_missions(b, TRUCKING));
    b.section_all_done("QuarryMissions", "Quarry", 1, |b| add_missions(b, 9593));
    b.section_all_done("Assets", "Couriers and Valet", 1, |b| add_missions2(b, "Assets", true));

    b.heading("Schools and vehicles", "Schools and vehicle missions", 0);
    b.section_all_done("Schools", "Schools", 1, |b| {
        add_missions2(b, "Schools", true);
        b.check("Driving School Started", None, false);
        b.tooltip("Splits when starting Driving School for the first time");
    });
    b.section_all_done("Vehicle Submissions", "Vehicle Submissions", 1, |b| {
        add_missions2(b, "Vehicle Submissions", true);
        add_mission_list(b, &["Freight Level 1", "Freight Level 2"]);
    });

    b.heading("Races and stadium", "Races and stadium", 0);
    b.section("Races", "Races", 1, |b| {
        b.check("All Races Won", None, true);
        b.section_all_done("LS Races", "Los Santos Races", 2, |b| {
            add_missions2(b, "LS Races", false)
        });
        b.section_all_done("SF Races", "San Fierro Races", 2, |b| {
            add_missions2(b, "SF Races", false)
        });
        b.section_all_done("LV Races", "Las Venturas Races", 2, |b| {
            add_missions2(b, "LV Races", false)
        });
        b.section_all_done("Air Races", "Air Races", 2, |b| add_missions2(b, "Air Races", false));
    });
    b.section_all_done("Stadium Events", "Stadium Events", 1, |b| {
        add_missions2(b, "Stadium Events", true)
    });

    b.heading("Challenges and gyms", "Challenges and gyms", 0);
    b.section_all_done("Challenges", "Challenges", 1, |b| {
        b.check("Chiliad Challenge #1", None, true);
        b.check("Chiliad Challenge #2", None, true);
        b.check("Chiliad Challenge #3", None, true);
        add_missions2(b, "Challenges", true);
    });
    b.section_all_done("Gym Moves", "Gym Moves", 1, |b| add_missions2(b, "Gym Moves", true));

    b.heading("Import/Export", "Import/Export", 0);
    b.section("Export Lists", "Export Lists", 1, |b| {
        for (i, list) in data::EXPORT_LISTS.iter().enumerate() {
            let n = i + 1;
            // "All done" replaces the ASL's "Export List n Complete", split
            // by the export code in `lib.rs` rather than `completion`.
            b.section_all_done(
                &format!("Export List {n}"),
                &format!("Import/Export List {n}"),
                2,
                |b| {
                    for vehicle in list {
                        b.check(&format!("Export {vehicle}"), Some(vehicle), false);
                    }
                },
            );
        }
    });

    b.heading("Property purchases", "Properties", 0);
    b.section_all_done("Properties", "Properties", 1, |b| {
        for (label, properties) in PROPERTIES {
            b.heading(&format!("Properties {label}"), label, 2);
            for p in *properties {
                b.check(p, None, false);
            }
        }
    });

    // Collectibles
    //=============
    b.heading("Collectibles", "Collectibles", 0);
    for (kind, _) in data::COLLECTIBLES {
        b.collectible(kind);
    }

    // Other
    //======
    b.heading("Other", "Other", 0);
    b.check("Plane Flight", None, false);
    b.tooltip("Splits when entering the ticket machine marker for the first time");

    for (key, default) in DEFAULTS_100 {
        b.set_default(key, *default);
    }
    b.finish()
}

/// Defaults for a 100% run, taken from the Windows LiveSplit settings in
/// "Grand Theft Auto San Andreas Remastered - 100%.lss", plus the user's own
/// changes since (GT #1 and GT #2). Only the settings whose value differs from
/// the ASL's default are listed. `{kind}Each` and `{kind}All` set the
/// collectible dropdowns, `Export List n#allDone` stands for the ASL's
/// "Export List n Complete". Every section switch was on there, so these are
/// the splits that happened by default before the switches became dropdowns.
#[rustfmt::skip]
static DEFAULTS_100: &[(&str, bool)] = &[
    ("All Races Won", false),
    ("Angel Pine (Safehouse)", true),
    ("Blueberry (Safehouse)", true),
    ("Calton Heights (Safehouse)", true),
    ("Chinatown (Safehouse)", true),
    ("Creek (Safehouse)", true),
    ("Dillimore (Safehouse)", true),
    ("Doherty (Safehouse)", true),
    ("El Quebrados (Safehouse)", true),
    ("End of the Line Part 2", true),
    ("End of the Line Part 3", true),
    ("Export Admiral", true),
    ("Export Buffalo", true),
    ("Export Camper", true),
    ("Export Feltzer", true),
    ("Export Infernus", true),
    ("Export List 2#allDone", true),
    ("Export List 3#allDone", true),
    ("Export Patriot", true),
    ("Export Remington", true),
    ("Export Sanchez", true),
    ("Export Sentinel", true),
    ("Export Stretch", true),
    ("Fort Carson (Safehouse)", true),
    ("Freight Level 1", false),
    ("GT #1", true),
    ("GT #2", true),
    ("Hashbury (Safehouse)", true),
    ("HorseshoesEach", true),
    ("Jefferson (Safehouse)", true),
    ("Mulholland (Safehouse)", true),
    ("Old Venturas Strip (Hotel Suite)", true),
    ("OystersEach", true),
    ("Palomino Creek (Safehouse)", true),
    ("Paradiso (Safehouse)", true),
    ("PhotosEach", true),
    ("Pirates In Men's Pants (Hotel Suite)", true),
    ("Prickle Pine (Safehouse)", true),
    ("Quarry 1", false),
    ("Quarry 2", false),
    ("Quarry 3", false),
    ("Quarry 4", false),
    ("Quarry 6", false),
    ("Redsands West (Safehouse)", true),
    ("Rockshore West (Safehouse)", true),
    ("Santa Maria Beach (Safehouse)", true),
    ("TagsEach", true),
    ("The Camel's Toe (Hotel Suite)", true),
    ("The Clown's Pocket (Hotel Suite)", true),
    ("Tierra Robada (Safehouse)", true),
    ("Verdant Bluffs (Safehouse)", true),
    ("Verdant Meadows (Safehouse)", true),
    ("Verona Beach (Safehouse)", true),
    ("Wang Cars (Showroom Bought)", true),
    ("Whetstone (Safehouse)", true),
    ("Whitewood Estate (Safehouse)", true),
    ("Willowfield (Safehouse)", true),
    ("Zero (RC Shop Bought)", true),
    ("any%", false),
];

#[cfg(test)]
mod tests {
    use alloc::{collections::BTreeMap, string::String, vec::Vec};

    use super::*;
    use crate::{completion, settings::tree::Kind};

    /// What every setting of the old tree (with the "Enable splits in this
    /// section" switches) did with no saved settings, `key<TAB>true|false`.
    #[rustfmt::skip]
    static OLD_DEFAULTS: &str = include_str!("old_defaults.tsv");

    /// Old keys whose default was changed on purpose since.
    #[rustfmt::skip]
    static CHANGED: &[&str] = &["GT #1", "GT #2"];

    /// The old section switches, which are dropdowns now.
    fn is_section(tree: &Tree, key: &str) -> bool {
        let dropdown = format!("{key}#section");
        tree.widgets.iter().any(|w| w.key == dropdown)
    }

    #[test]
    fn every_old_key_resolves_to_its_old_default() {
        let tree = build();
        let state = tree.read(&BTreeMap::new());
        let synced = tree.sync(None, &state);
        assert_eq!(state, synced, "section defaults disagree with their checkboxes");

        let (mut sections, mut settings) = (0, 0);
        for line in OLD_DEFAULTS.lines() {
            let (key, value) = line.split_once('\t').unwrap();
            if is_section(&tree, key) {
                sections += 1;
                continue;
            }
            settings += 1;
            let expected = if CHANGED.contains(&key) { true } else { value == "true" };
            // The list's "All done" option replaced its "Complete" checkbox.
            let key = match key.strip_suffix(" Complete") {
                Some(list) if list.starts_with("Export List ") => format!("{list}#allDone"),
                _ => key.into(),
            };
            assert_eq!(tree.get(&state, &key), Some(expected), "{key}");
        }
        assert_eq!((sections, settings), (27, 250));
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
    fn every_story_mission_is_listed_once() {
        let tree = build();
        let checkboxes: Vec<&str> = tree
            .widgets
            .iter()
            .filter(|w| matches!(w.kind, Kind::Bool { .. }))
            .map(|w| w.key.as_str())
            .collect();
        for list in
            [LS_MISSIONS, BL_MISSIONS, SF_MISSIONS, DESERT_MISSIONS, LV_MISSIONS, RTLS_MISSIONS]
        {
            for m in list {
                assert_eq!(checkboxes.iter().filter(|k| *k == m).count(), 1, "{m}");
            }
        }
        // Grouping by giver neither drops nor repeats a mission.
        for (missions, givers) in
            [(LS_MISSIONS, LS_GIVERS), (SF_MISSIONS, SF_GIVERS), (LV_MISSIONS, LV_GIVERS)]
        {
            for m in missions {
                let groups = givers.iter().filter(|(_, index)| giver(m) == Some(*index)).count();
                assert_eq!(groups, 1, "{m}");
            }
        }
    }

    #[test]
    fn every_property_is_listed_once() {
        let mut expected: Vec<&str> =
            data::missions2("Properties").iter().map(|(_, p)| *p).collect();
        expected.push("Verdant Meadows (Safehouse)");
        expected.push("Wang Cars (Showroom Bought)");
        expected.sort();
        let mut listed: Vec<&str> =
            PROPERTIES.iter().flat_map(|(_, p)| p.iter().copied()).collect();
        listed.sort();
        assert_eq!(listed, expected);
    }

    #[test]
    fn collectibles_default_to_the_old_checkboxes() {
        let tree = build();
        let state = tree.read(&BTreeMap::new());
        let defaults: BTreeMap<String, &str> = tree
            .widgets
            .iter()
            .filter_map(|w| match w.kind {
                Kind::Choice { default, .. } if w.key.ends_with("#splits") => {
                    Some((w.key.clone(), default))
                }
                _ => None,
            })
            .collect();
        assert_eq!(defaults["Photos#splits"], "each");
        assert_eq!(defaults["Tags#splits"], "each");
        assert_eq!(defaults["Oysters#splits"], "each");
        assert_eq!(defaults["Horseshoes#splits"], "each");
        assert_eq!(defaults["Stunts (Completed)#splits"], "off");
        assert_eq!(tree.get(&state, "Stunts (Completed)Each"), Some(false));
    }

    #[test]
    fn all_done_sections_count_their_checkboxes() {
        let tree = build();
        let mut offered = tree.all_done_sections();
        offered.sort();
        let mut expected: Vec<&str> = completion::SECTIONS.iter().map(|(key, _)| *key).collect();
        expected.extend(["Export List 1", "Export List 2", "Export List 3"]);
        expected.sort();
        assert_eq!(offered, expected);

        // Checkboxes that aren't counted, see `completion::SECTIONS`.
        let not_counted: &[(&str, &[&str])] = &[
            ("Schools", &["Driving School Started"]),
            ("Properties", &["Wang Cars (Showroom Bought)", "Verdant Meadows (Safehouse)"]),
            (
                "Challenges",
                &["Chiliad Challenge #1", "Chiliad Challenge #2", "Chiliad Challenge #3"],
            ),
        ];
        for (key, count) in completion::SECTIONS {
            let (boxes, _) = tree.section(key);
            let skip = not_counted.iter().find(|(k, _)| k == key).map_or(&[][..], |(_, s)| s);
            let mut expected: Vec<&str> = boxes.into_iter().filter(|b| !skip.contains(b)).collect();
            expected.sort();
            let mut counted = count.names();
            counted.sort();
            assert_eq!(counted, expected, "{key}");
        }
    }

    #[test]
    fn export_lists_default_to_selection() {
        let tree = build();
        let state = tree.read(&BTreeMap::new());
        assert_eq!(tree.get(&state, "Export List 1#allDone"), Some(false));
        assert_eq!(tree.get(&state, "Export List 2#allDone"), Some(true));
        assert_eq!(tree.get(&state, "Export List 3#allDone"), Some(true));
        let dropdown = |key: &str| match tree.widgets.iter().find(|w| w.key == key).unwrap().kind {
            Kind::Choice { default, .. } => default,
            _ => unreachable!(),
        };
        assert_eq!(dropdown("Export List 1#section"), "all");
        assert_eq!(dropdown("Export List 2#section"), "alldone");
        assert_eq!(dropdown("Export Lists#section"), "selection");
        // Nothing to correct or store at startup.
        assert_eq!(tree.sync(None, &state), state);
    }
}
