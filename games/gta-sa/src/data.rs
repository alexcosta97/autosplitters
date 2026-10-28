//! Static tables ported from the `startup` block of `GTASA.asl`.
//!
//! Addresses are module-relative and for version 1.0, exactly as in the ASL.
//! Mission names also act as setting keys, so don't change them.
//!
//! The ASL also watches `intro_state` (0x64ED04) and `started`, but never
//! uses them, so they are left out.

use alloc::vec::Vec;

/// One collectible kind: the 1.0 address (plus the version offset) and the
/// separate Steam address.
pub struct Collectible {
    pub kind: &'static str,
    pub addr: u32,
    pub steam_addr: u32,
}

/// In `names::COLLECTIBLE_KINDS` order.
pub static COLLECTIBLES: [Collectible; 5] = [
    Collectible { kind: "Photos", addr: 0x7791BC, steam_addr: 0x80C3E4 },
    Collectible { kind: "Tags", addr: 0x69AD74, steam_addr: 0x71258C },
    Collectible { kind: "Oysters", addr: 0x7791EC, steam_addr: 0x80C414 },
    Collectible { kind: "Horseshoes", addr: 0x7791E4, steam_addr: 0x80C40C },
    Collectible { kind: "Stunts (Completed)", addr: 0x779064, steam_addr: 0x80C28C },
];

/// Memory addresses and the associated values and missions.
#[rustfmt::skip]
pub static MISSIONS: &[(u32, &[(i32, &str)])] = &[
    (0x64A060, &[ // $INTRO_TOTAL_PASSED_MISSIONS
        (1, "Big Smoke"),
        (2, "Ryder"),
    ]),
    (0x64A070, &[ // $SWEET_TOTAL_PASSED_MISSIONS
        (1, "Tagging up Turf"),
        (2, "Cleaning the Hood"),
        (3, "Drive-Thru"),
        (4, "Nines and AKs"),
        (5, "Drive-By"),
        (6, "Sweet's Girl"),
        (7, "Cesar Vialpando"),
        (8, "Doberman"),
        (9, "Los Sepulcros"),
    ]),
    (0x64A078, &[ // $SMOKE_TOTAL_PASSED_MISSIONS
        (1, "OG Loc"),
        (2, "Running Dog"),
        (3, "Wrong Side of the Tracks"),
        (4, "Just Business"),
    ]),
    (0x64A074, &[ // $RYDER_TOTAL_PASSED_MISSIONS
        (1, "Home Invasion"),
        (2, "Catalyst"),
        (3, "Robbing Uncle Sam"),
    ]),
    (0x64A088, &[ // $LS_FINAL_TOTAL_PASSED_MISSIONS
        (1, "Reuniting the Families"),
        (2, "The Green Sabre"),
    ]),
    (0x64A080, &[ // $CRASH_LS_TOTAL_PASSED_MISSIONS
        (1, "Burning Desire"),
        (2, "Gray Imports"),
    ]),
    (0x64A07C, &[ // $OG_LOC_TOTAL_PASSED_MISSIONS
        (1, "Life's a Beach"),
        (2, "Madd Dogg's Rhymes"),
        (3, "Management Issues"),
        (4, "House Party (Cutscene)"),
        (5, "House Party"),
    ]),
    (0x64A084, &[ // $MISSION_LOWRIDER_PASSED
        (1, "High Stakes Lowrider"),
    ]),
    (0x64A114, &[ // $MISSION_BADLANDS_PASSED
        (1, "Badlands"),
    ]),
    (0x64A490, &[ // $MISSION_TANKER_COMMANDER_PASSED
        (1, "Tanker Commander"),
    ]),
    (0x64A48C, &[ // $MISSION_SMALL_TOWN_BANK_PASSED
        (1, "Small Town Bank"),
    ]),
    (0x64A488, &[ // $MISSION_LOCAL_LIQUOR_STORE_PASSED
        (1, "Local Liquor Store"),
    ]),
    (0x64A494, &[ // $ALL_CATALINA_MISSIONS_PASSED (not aptly named variable)
        (1, "Against All Odds"),
    ]),
    (0x64BB2C, &[ // $2163
        (1, "King in Exile"),
    ]),
    (0x64A10C, &[ // $TRUTH_TOTAL_PASSED_MISSIONS
        (1, "Body Harvest"),
        (2, "Are You Going To San Fierro?"),
    ]),
    (0x64BDC8, &[ // $RACES_WON_NUMBER (first 3 Races are in a fixed order due to missions)
        (2, "Wu Zi Mu"),
        (3, "Farewell, My Love"),
        (25, "All Races Won"),
    ]),
    (0x64A1D4, &[ // $GARAGE_TOTAL_PASSED_MISSIONS
        (1, "Wear Flowers in your Hair"),
        (2, "Deconstruction"),
    ]),
    (0x64A1DC, &[ // $WUZIMU_TOTAL_PASSED_MISSIONS
        (1, "Mountain Cloud Boys"),
        (2, "Ran Fa Li"),
        (3, "Lure"),
        (4, "Amphibious Assault"),
        (5, "The Da Nang Thang"),
    ]),
    (0x64A1E4, &[ // $SYNDICATE_TOTAL_PASSED_MISSIONS
        (1, "Photo Opportunity"),
        (2, "Jizzy (Cutscene)"),
        (3, "Jizzy"),
        (4, "T-Bone Mendez"),
        (5, "Mike Toreno"),
        (6, "Outrider"),
        (7, "Ice Cold Killa"),
        (8, "Pier 69"),
        (9, "Toreno's Last Flight"),
        (10, "Yay Ka-Boom-Boom"),
    ]),
    (0x64A1E8, &[ // $CRASH_SF_TOTAL_PASSED_MISSIONS
        (1, "555 WE TIP"),
        (2, "Snail Trail"),
    ]),
    (0x64A2A4, &[ // $TORENO_TOTAL_PASSED_MISSIONS
        (1, "Monster"),
        (2, "Highjack"),
        (3, "Interdiction"),
        (4, "Verdant Meadows"),
        (5, "Learning to Fly"),
        (6, "N.O.E."),
        (7, "Stowaway"),
        (8, "Black Project"),
        (9, "Green Goo"),
    ]),
    (0x64A1D8, &[ // $ZERO_TOTAL_PASSED_MISSIONS
        (1, "Air Raid"),
        (2, "Supply Lines..."),
        (3, "New Model Army"),
    ]),
    (0x64A1E0, &[ // $STEAL_TOTAL_PASSED_MISSIONS
        (1, "Zeroing In"),
        (2, "Test Drive"),
        (3, "Customs Fast Track"),
        (4, "Puncture Wounds"),
    ]),
    (0x64A2B4, &[ // $CASINO_TOTAL_PASSED_MISSIONS
        (1, "Fender Ketchup"),
        (2, "Explosive Situation"),
        (3, "You've Had Your Chips"),
        (4, "Don Peyote"),
        (5, "Intensive Care"),
        (6, "The Meat Business"),
        (7, "Fish in a Barrel"),
        (8, "Freefall"),
        (9, "Saint Mark's Bistro"),
    ]),
    (0x64A2B8, &[ // $598 (CRASH_LV)
        (1, "Misappropriation"),
        (2, "High Noon"),
    ]),
    (0x64A2BC, &[ // $599 (Madd Dogg)
        (1, "Madd Dogg"),
    ]),
    (0x64A2C0, &[ // $HEIST_TOTAL_PASSED_MISSIONS
        (1, "Architectural Espionage"),
        (2, "Key to her Heart"),
        (3, "Dam and Blast"),
        (4, "Cop Wheels"),
        (5, "Up, Up and Away!"),
        (6, "Breaking the Bank at Caligula's"),
    ]),
    (0x64A328, &[ // $MANSION_TOTAL_PASSED_MISSIONS
        (1, "A Home in the Hills"),
        (2, "Vertical Bird"),
        (3, "Home Coming"),
        (4, "Cut Throat Business"),
    ]),
    (0x64A32C, &[ // $GROVE_TOTAL_PASSED_MISSIONS
        (1, "Beat Down on B Dup"),
        (2, "Grove 4 Life"),
    ]),
    (0x64A334, &[ // $RIOT_TOTAL_PASSED_MISSIONS
        (1, "Riot"),
        (2, "Los Desperados"),
        (3, "End of the Line Part 1"),
        (4, "End of the Line Part 2"),
        (5, "End of the Line Part 3"), // After credits
    ]),
    (0x6518DC, &[ // $TRUCKING_TOTAL_PASSED_MISSIONS
        (1, "Trucking 1"),
        (2, "Trucking 2"),
        (3, "Trucking 3"),
        (4, "Trucking 4"),
        (5, "Trucking 5"),
        (6, "Trucking 6"),
        (7, "Trucking 7"),
        (8, "Trucking 8"),
    ]),
];

/// Other missions: addresses that are responsible for a single mission each.
#[rustfmt::skip]
pub static MISSIONS2: &[(&str, &[(u32, &str)])] = &[
    ("Schools", &[
        (0x649AB8, "Driving School Passed"), // $MISSION_BACK_TO_SCHOOL_PASSED
        (0x64B824, "Boat School Passed"), // $MISSION_BOAT_SCHOOL_PASSED
        (0x64BBC4, "Bike School Passed"), // $MISSION_DRIVING_SCHOOL_PASSED (actually Bike School)
    ]),
    ("Vehicle Submissions", &[
        (0x64B0A4, "Firefighter Complete"), // $1489 (directly goes to 2 when complete)
        (0x64B0A0, "Vigilante Complete"), // $1488
        (0x64B0AC, "Taxi Mission Complete"), // $MISSION_TAXI_PASSED ($1491)
        (0x64B09C, "Paramedic Complete"), // $1487
        (0x64B87C, "Pimping Complete"), // $MISSION_PIMPING_PASSED ($1991)
    ]),
    ("Properties", &[
        (0x64B2B0, "Zero (RC Shop Bought)"),
        (0x64A4CC, "Santa Maria Beach (Safehouse)"),
        (0x64A4D0, "Rockshore West (Safehouse)"),
        (0x64A4D4, "Fort Carson (Safehouse)"),
        (0x64A4D8, "Prickle Pine (Safehouse)"),
        (0x64A4DC, "Whitewood Estate (Safehouse)"),
        (0x64A4E0, "Palomino Creek (Safehouse)"),
        (0x64A4E4, "Redsands West (Safehouse)"),
        (0x64A4E8, "Verdant Bluffs (Safehouse)"),
        (0x64A4EC, "Calton Heights (Safehouse)"),
        (0x64A4F0, "Mulholland (Safehouse)"),
        (0x64A4F4, "Paradiso (Safehouse)"),
        (0x64A4F8, "Hashbury (Safehouse)"),
        (0x64A4FC, "Verona Beach (Safehouse)"),
        (0x64A500, "Pirates In Men's Pants (Hotel Suite)"),
        (0x64A504, "The Camel's Toe (Hotel Suite)"),
        (0x64A508, "Chinatown (Safehouse)"),
        (0x64A50C, "Whetstone (Safehouse)"),
        (0x64A510, "Doherty (Safehouse)"),
        (0x64A514, "Queens (Hotel Suite)"),
        (0x64A518, "Angel Pine (Safehouse)"),
        (0x64A51C, "El Quebrados (Safehouse)"),
        (0x64A520, "Tierra Robada (Safehouse)"),
        (0x64A524, "Dillimore (Safehouse)"),
        (0x64A528, "Jefferson (Safehouse)"),
        (0x64A52C, "Old Venturas Strip (Hotel Suite)"),
        (0x64A530, "The Clown's Pocket (Hotel Suite)"),
        (0x64A534, "Creek (Safehouse)"),
        (0x64A538, "Willowfield (Safehouse)"),
        (0x64A53C, "Blueberry (Safehouse)"),
    ]),
    ("Freight", &[
        (0x651A20, "Freight Level 1"), // $8240
        (0x651A1C, "Freight Level 2"), // $8239 (goes to 2 at the end of the level)
    ]),
    ("Gym Moves", &[
        (0x6518C4, "Los Santos Gym Moves"), // $8153
        (0x6518C8, "San Fierro Gym Moves"), // $8154
        (0x6518D8, "Las Venturas Gym Moves"), // $8158
    ]),
    ("Challenges", &[
        (0x64C510, "NRG-500 Stunt Challenge"), // $2796
        (0x64C50C, "BMX Stunt Challenge"), // $2795
        (0x64EBC0, "Shooting Range Complete"), // $5272
    ]),
    ("Assets", &[
        (0x64B880, "Los Santos Courier"), // $MISSION_COURIER_LS_PASSED ($1992)
        (0x64B884, "Las Venturas Courier"), // $MISSION_COURIER_LV_PASSED ($1993)
        (0x64B888, "San Fierro Courier"), // $MISSION_COURIER_SF_PASSED ($1994)
        (0x64B710, "Valet Parking Complete"), // $1900
        (0x64B0B4, "Quarry Complete"), // $MISSION_QUARRY_PASSED ($1493)
    ]),
        // Races addresses are based on the global variable $RACES_WON ($2300), which
        // is an array. The number in the comment is the $RACE_INDEX ($352).
        //
        // Missing are races that are already done during story missions:
        // Lowrider Race (0), Badlands A (7), Badlands B (8)
        //
    ("LS Races", &[
        (0x64BD54, "Little Loop"), // 1
        (0x64BD58, "Backroad Wanderer"), // 2
        (0x64BD5C, "City Circuit"), // 3
        (0x64BD60, "Vinewood (Race)"), // 4
        (0x64BD64, "Freeway (Race)"), // 5
        (0x64BD68, "Into the Country"), // 6
    ]),
    ("SF Races", &[
        (0x64BD74, "Dirtbike Danger"), // 9
        (0x64BD78, "Bandito County"), // 10
        (0x64BD7C, "Go-Go Karting"), // 11
        (0x64BD80, "San Fierro Fastlane"), // 12
        (0x64BD84, "San Fierro Hills"), // 13
        (0x64BD88, "Country Endurance"), // 14
    ]),
    ("LV Races", &[
        (0x64BD8C, "SF to LV"), // 15
        (0x64BD90, "Dam Rider"), // 16
        (0x64BD94, "Desert Tricks"), // 17
        (0x64BD98, "LV Ringroad"), // 18
    ]),
    ("Air Races", &[
        (0x64BD9C, "World War Ace"), // 19
        (0x64BDA0, "Barnstorming"), // 20
        (0x64BDA4, "Military Service"), // 21
        (0x64BDA8, "Chopper Checkpoint"), // 22
        (0x64BDAC, "Whirly Bird Waypoint"), // 23
        (0x64BDB0, "Heli Hell"), // 24
    ]),
    ("Stadium Events", &[
        (0x64BDB4, "8-Track"), // 25
        (0xA4BDB8, "Dirt Track"), // 26
        (0x649AC8, "Kickstart"), // $MISSION_KICKSTART_PASSED ($90)
        (0x64B7B4, "Bloodring"), // $MISSION_BLOODRING_PASSED ($1941)
    ]),
];

/// The entries of `MISSIONS2` in the group called `group`.
pub fn missions2(group: &str) -> &'static [(u32, &'static str)] {
    MISSIONS2
        .iter()
        .find(|(name, _)| *name == group)
        .map_or(&[], |(_, entries)| entries)
}

/// Split when a certain thread was started, usually when a mission was started.
pub static START_MISSIONS: &[(&str, &str)] = &[
    ("grove2", "GT #1"),  // Grove 4 Life
    ("manson5", "GT #2"), // Cut Throat Business
    ("steal", "Wang Cars (Showroom Bought)"),
    ("planes", "Plane Flight"),
    ("psch", "Verdant Meadows (Safehouse)"),
    ("dskool", "Driving School Started"),
];

/// Threads that are running before the game proper has started.
pub static EARLY_GAME_THREADS: &[&str] = &["a_cont", "main", "intro", "gfagnt", "noname"];

pub const EOTL: u32 = 0x651698;
pub const CHILIAD_RACE: u32 = 0x64B57C;
pub const CHILIAD_DONE: u32 = 0x64B584;
pub const EXPORT_LIST: u32 = 0x64A9C4;
/// Vehicle `i` of the export list is at `EXPORT_BASE + i * 4`.
pub const EXPORT_BASE: u32 = 0x64A9F0;

/// Every address read as a script global, deduplicated.
pub fn global_addresses() -> Vec<u32> {
    let mut all: Vec<u32> = MISSIONS.iter().map(|(addr, _)| *addr).collect();
    for (_, entries) in MISSIONS2 {
        all.extend(entries.iter().map(|(addr, _)| *addr));
    }
    all.extend([EOTL, CHILIAD_RACE, CHILIAD_DONE, EXPORT_LIST]);
    all.extend((0..10).map(|i| EXPORT_BASE + i * 4));
    all.sort_unstable();
    all.dedup();
    all
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tables_match_the_asl() {
        assert_eq!(MISSIONS.len(), 31);
        let groups: Vec<&str> = MISSIONS2.iter().map(|(g, _)| *g).collect();
        assert_eq!(
            groups,
            [
                "Schools", "Vehicle Submissions", "Properties", "Freight", "Gym Moves",
                "Challenges", "Assets", "LS Races", "SF Races", "LV Races", "Air Races",
                "Stadium Events",
            ]
        );
        let dirt = missions2("Stadium Events").iter().find(|(_, n)| *n == "Dirt Track");
        assert_eq!(dirt.map(|(a, _)| *a), Some(0xA4BDB8));
        let mut names: Vec<&str> = MISSIONS.iter().flat_map(|(_, e)| e.iter().map(|(_, n)| *n)).collect();
        names.extend(MISSIONS2.iter().flat_map(|(_, e)| e.iter().map(|(_, n)| *n)));
        names.extend(START_MISSIONS.iter().map(|(_, n)| *n));
        let total = names.len();
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), total);
    }

    #[test]
    fn collectibles_follow_the_names() {
        let kinds: Vec<&str> = COLLECTIBLES.iter().map(|c| c.kind).collect();
        assert_eq!(kinds, gta_sa_common::names::COLLECTIBLE_KINDS);
    }
}

