//! Static tables ported from the `startup` block of `GTA_SA.asl`.
//!
//! Mission names also act as setting keys, so don't change them.

/// `$TRUCKING_TOTAL_PASSED_MISSIONS` on the versions the ASL's indexes are for.
pub const TRUCKING: u32 = 9581;

/// Globals that sit `Addresses::globals_shift` (6) slots later on
/// 1.0.113.21181 than the ASL's indexes, which are for other versions. The ASL
/// only did this for Trucking; the gyms were found by watching $9582 change
/// when learning the San Fierro gym moves on 1.0.113.21181.
pub static VERSION_SHIFTED: &[u32] = &[9575, 9576, 9580, TRUCKING];

/// Global variable index and the associated values and missions.
#[rustfmt::skip]
pub static MISSIONS: &[(u32, &[(i32, &str)])] = &[
    (450, &[(1, "Big Smoke"), (2, "Ryder")]), // $INTRO_TOTAL_PASSED_MISSIONS
    (454, &[ // $SWEET_TOTAL_PASSED_MISSIONS
        (1, "Tagging up Turf"), (2, "Cleaning the Hood"), (3, "Drive-Thru"),
        (4, "Nines and AKs"), (5, "Drive-By"), (6, "Sweet's Girl"),
        (7, "Cesar Vialpando"), (8, "Doberman"), (9, "Los Sepulcros"),
    ]),
    (456, &[(1, "OG Loc"), (2, "Running Dog"), (3, "Wrong Side of the Tracks"), (4, "Just Business")]), // $SMOKE_TOTAL_PASSED_MISSIONS
    (455, &[(1, "Home Invasion"), (2, "Catalyst"), (3, "Robbing Uncle Sam")]), // $RYDER_TOTAL_PASSED_MISSIONS
    (460, &[(1, "Reuniting the Families"), (2, "The Green Sabre")]), // $LS_FINAL_TOTAL_PASSED_MISSIONS
    (458, &[(1, "Burning Desire"), (2, "Gray Imports")]), // $CRASH_LS_TOTAL_PASSED_MISSIONS
    (457, &[ // $OG_LOC_TOTAL_PASSED_MISSIONS
        (1, "Life's a Beach"), (2, "Madd Dogg's Rhymes"), (3, "Management Issues"),
        (4, "House Party (Cutscene)"), (5, "House Party"),
    ]),
    (459, &[(1, "High Stakes Lowrider")]), // $MISSION_LOWRIDER_PASSED
    (495, &[(1, "Badlands")]), // $MISSION_BADLANDS_PASSED
    (720, &[(1, "Tanker Commander")]), // $MISSION_TANKER_COMMANDER_PASSED
    (719, &[(1, "Small Town Bank")]), // $MISSION_SMALL_TOWN_BANK_PASSED
    (718, &[(1, "Local Liquor Store")]), // $MISSION_LOCAL_LIQUOR_STORE_PASSED
    (721, &[(1, "Against All Odds")]), // $ALL_CATALINA_MISSIONS_PASSED (not aptly named variable)
    (3584, &[(1, "King in Exile")]), // $2163
    (493, &[(1, "Body Harvest"), (2, "Are You Going To San Fierro?")]), // $TRUTH_TOTAL_PASSED_MISSIONS
    (3751, &[(2, "Wu Zi Mu"), (3, "Farewell, My Love"), (25, "All Races Won")]), // $RACES_WON_NUMBER
    (543, &[(1, "Wear Flowers in your Hair"), (2, "Deconstruction")]), // $GARAGE_TOTAL_PASSED_MISSIONS
    (545, &[ // $WUZIMU_TOTAL_PASSED_MISSIONS
        (1, "Mountain Cloud Boys"), (2, "Ran Fa Li"), (3, "Lure"),
        (4, "Amphibious Assault"), (5, "The Da Nang Thang"),
    ]),
    (547, &[ // $SYNDICATE_TOTAL_PASSED_MISSIONS
        (1, "Photo Opportunity"), (2, "Jizzy (Cutscene)"), (3, "Jizzy"),
        (4, "T-Bone Mendez"), (5, "Mike Toreno"), (6, "Outrider"),
        (7, "Ice Cold Killa"), (8, "Pier 69"), (9, "Toreno's Last Flight"),
        (10, "Yay Ka-Boom-Boom"),
    ]),
    (548, &[(1, "555 WE TIP"), (2, "Snail Trail")]), // $CRASH_SF_TOTAL_PASSED_MISSIONS
    (595, &[ // $TORENO_TOTAL_PASSED_MISSIONS
        (1, "Monster"), (2, "Highjack"), (3, "Interdiction"), (4, "Verdant Meadows"),
        (5, "Learning to Fly"), (6, "N.O.E."), (7, "Stowaway"), (8, "Black Project"),
        (9, "Green Goo"),
    ]),
    (544, &[(1, "Air Raid"), (2, "Supply Lines..."), (3, "New Model Army")]), // $ZERO_TOTAL_PASSED_MISSIONS
    (546, &[(1, "Zeroing In"), (2, "Test Drive"), (3, "Customs Fast Track"), (4, "Puncture Wounds")]), // $STEAL_TOTAL_PASSED_MISSIONS
    (599, &[ // $CASINO_TOTAL_PASSED_MISSIONS
        (1, "Fender Ketchup"), (2, "Explosive Situation"), (3, "You've Had Your Chips"),
        (4, "Don Peyote"), (5, "Intensive Care"), (6, "The Meat Business"),
        (7, "Fish in a Barrel"), (8, "Freefall"), (9, "Saint Mark's Bistro"),
    ]),
    (600, &[(1, "Misappropriation"), (2, "High Noon")]), // $598 (CRASH_LV)
    (601, &[(1, "Madd Dogg")]), // $599 (Madd Dogg)
    (602, &[ // $HEIST_TOTAL_PASSED_MISSIONS
        (1, "Architectural Espionage"), (2, "Key to her Heart"), (3, "Dam and Blast"),
        (4, "Cop Wheels"), (5, "Up, Up and Away!"), (6, "Breaking the Bank at Caligula's"),
    ]),
    (628, &[(1, "A Home in the Hills"), (2, "Vertical Bird"), (3, "Home Coming"), (4, "Cut Throat Business")]), // $MANSION_TOTAL_PASSED_MISSIONS
    (629, &[(1, "Beat Down on B Dup"), (2, "Grove 4 Life")]), // $GROVE_TOTAL_PASSED_MISSIONS
    (631, &[ // $RIOT_TOTAL_PASSED_MISSIONS
        (1, "Riot"), (2, "Los Desperados"), (3, "End of the Line Part 1"),
        (4, "End of the Line Part 2"), (5, "End of the Line Part 3"), // After credits
    ]),
    (TRUCKING, &[ // $TRUCKING_TOTAL_PASSED_MISSIONS
        (1, "Trucking 1"), (2, "Trucking 2"), (3, "Trucking 3"), (4, "Trucking 4"),
        (5, "Trucking 5"), (6, "Trucking 6"), (7, "Trucking 7"), (8, "Trucking 8"),
    ]),
    (9593, &[
        (1, "Quarry 1"), (2, "Quarry 2"), (3, "Quarry 3"), (4, "Quarry 4"),
        (5, "Quarry 5"), (6, "Quarry 6"), (7, "Quarry 7"),
    ]),
];

/// Addresses that are responsible for a single mission each.
#[rustfmt::skip]
pub static MISSIONS2: &[(&str, &[(u32, &str)])] = &[
    // Flight School not here because it is a Story Mission
    ("Schools", &[
        (8832, "Driving School Passed"), // $MISSION_BACK_TO_SCHOOL_PASSED
        (3390, "Boat School Passed"),    // $MISSION_BOAT_SCHOOL_PASSED
        (3622, "Bike School Passed"),    // $MISSION_DRIVING_SCHOOL_PASSED (actually Bike School)
    ]),
    ("Vehicle Submissions", &[
        (2898, "Firefighter Complete"),  // $1489 (directly goes to 2 when complete)
        (2897, "Vigilante Complete"),    // $1488
        (2900, "Taxi Mission Complete"), // $MISSION_TAXI_PASSED ($1491)
        (2896, "Paramedic Complete"),    // $1487
        (3412, "Pimping Complete"),      // $MISSION_PIMPING_PASSED ($1991)
    ]),
    ("Properties", &[
        (3039, "Zero (RC Shop Bought)"),
        (735, "Santa Maria Beach (Safehouse)"),
        (736, "Rockshore West (Safehouse)"),
        (737, "Fort Carson (Safehouse)"),
        (738, "Prickle Pine (Safehouse)"),
        (739, "Whitewood Estate (Safehouse)"),
        (740, "Palomino Creek (Safehouse)"),
        (741, "Redsands West (Safehouse)"),
        (742, "Verdant Bluffs (Safehouse)"),
        (743, "Calton Heights (Safehouse)"),
        (744, "Mulholland (Safehouse)"),
        (745, "Paradiso (Safehouse)"),
        (746, "Hashbury (Safehouse)"),
        (747, "Verona Beach (Safehouse)"),
        (748, "Pirates In Men's Pants (Hotel Suite)"),
        (749, "The Camel's Toe (Hotel Suite)"),
        (750, "Chinatown (Safehouse)"),
        (751, "Whetstone (Safehouse)"),
        (752, "Doherty (Safehouse)"),
        (753, "Queens (Hotel Suite)"),
        (754, "Angel Pine (Safehouse)"),
        (755, "El Quebrados (Safehouse)"),
        (756, "Tierra Robada (Safehouse)"),
        (757, "Dillimore (Safehouse)"),
        (758, "Jefferson (Safehouse)"),
        (759, "Old Venturas Strip (Hotel Suite)"),
        (760, "The Clown's Pocket (Hotel Suite)"),
        (761, "Creek (Safehouse)"),
        (762, "Willowfield (Safehouse)"),
        (763, "Blueberry (Safehouse)"),
    ]),
    ("Freight", &[
        (9670, "Freight Level 1"), // $8240
        (9669, "Freight Level 2"), // $8239 (goes to 2 at the end of the level)
    ]),
    ("Gym Moves", &[
        (9575, "Los Santos Gym Moves"),    // $8153
        (9576, "San Fierro Gym Moves"),    // $8154
        (9580, "Las Venturas Gym Moves"),  // $8158
    ]),
    ("Challenges", &[
        (4214, "NRG-500 Stunt Challenge"), // $4214
        (4213, "BMX Stunt Challenge"),     // $4213
        // player_has_fast_reload: set to 1 the first time the final
        // (assault rifle) round is passed. The ASL used 6690, which is
        // car_gen_hover[5] in DE and gets a car generator handle on New Game.
        (6694, "Shooting Range Complete"), // $5272
    ]),
    ("Assets", &[
        (3413, "Los Santos Courier"),    // $MISSION_COURIER_LS_PASSED ($1992)
        (3414, "Las Venturas Courier"),  // $MISSION_COURIER_LV_PASSED ($1993)
        (3415, "San Fierro Courier"),    // $MISSION_COURIER_SF_PASSED ($1994)
        (3320, "Valet Parking Complete"), // $1900
    ]),
    // Races addresses are based on the global variable $RACES_WON ($3661), which
    // is an array. The number in the comment is the $RACE_INDEX ($353).
    //
    // Missing are races that are already done during story missions:
    // Lowrider Race (0), Badlands A (7), Badlands B (8)
    ("LS Races", &[
        (3722, "Little Loop"),       // 1
        (3723, "Backroad Wanderer"), // 2
        (3724, "City Circuit"),      // 3
        (3725, "Vinewood (Race)"),   // 4
        (3726, "Freeway (Race)"),    // 5
        (3727, "Into the Country"),  // 6
    ]),
    ("SF Races", &[
        (3728, "Dirtbike Danger"),     // 9
        (3729, "Bandito County"),      // 10
        (3730, "Go-Go Karting"),       // 11
        (3731, "San Fierro Fastlane"), // 12
        (3732, "San Fierro Hills"),    // 13
        (3733, "Country Endurance"),   // 14
    ]),
    ("LV Races", &[
        (3734, "SF to LV"),     // 15
        (3735, "Dam Rider"),    // 16
        (3736, "Desert Tricks"), // 17
        (3737, "LV Ringroad"),  // 18
    ]),
    ("Air Races", &[
        (3738, "World War Ace"),        // 19
        (3739, "Barnstorming"),         // 20
        (3740, "Military Service"),     // 21
        (3741, "Chopper Checkpoint"),   // 22
        (3742, "Whirly Bird Waypoint"), // 23
        (3743, "Heli Hell"),            // 24
    ]),
    ("Stadium Events", &[
        (3744, "8-Track"),   // 25
        (3745, "Dirt Track"), // 26
        (91, "Kickstart"),   // $MISSION_KICKSTART_PASSED ($90)
        (3362, "Bloodring"), // $MISSION_BLOODRING_PASSED ($1941)
    ]),
];

/// Collectible type (acts as setting ID) and integer stat ID.
#[rustfmt::skip]
pub static COLLECTIBLES: &[(&str, u32)] = &[
    ("Photos", 231),
    ("Tags", 322), // read from a separate address, the stat ID isn't used
    ("Oysters", 243),
    ("Horseshoes", 241),
    ("Stunts (Completed)", 145),
];

#[rustfmt::skip]
pub static EXPORT_LISTS: [[&str; 10]; 3] = [
    ["Buffalo", "Sentinel", "Infernus", "Camper", "Admiral",
     "Patriot", "Sanchez", "Stretch", "Feltzer", "Remington"],
    ["Cheetah", "Rancher", "Stallion", "Tanker", "Comet",
     "Slamvan", "Blista Compact", "Stafford", "Sabre", "FCR-900"],
    ["Banshee", "Super GT", "Journey", "Huntley", "BF Injection",
     "Blade", "Freeway", "Mesa", "ZR-350", "Euros"],
];

/// Split when a certain thread was started, usually when a mission was started.
#[rustfmt::skip]
pub static START_MISSIONS: &[(&str, &str)] = &[
    ("grove2", "GT #1"),  // Grove 4 Life
    ("manson5", "GT #2"), // Cut Throat Business
    ("steal", "Wang Cars (Showroom Bought)"),
    ("planes", "Plane Flight"),
    ("psch", "Verdant Meadows (Safehouse)"),
    ("dskool", "Driving School Started"),
];

// Global variables that aren't missions
pub const EOTL: u32 = 9436;
pub const CHILIAD_RACE: u32 = 3218;
pub const CHILIAD_DONE: u32 = 3220;
pub const EXPORT_LIST: u32 = 1088;
pub const EXPORT_BASE: u32 = 1099;

/// Highest global variable index read, so the whole range can be read at once.
pub const MAX_GLOBAL: u32 = 9670;

pub fn missions2(group: &str) -> &'static [(u32, &'static str)] {
    MISSIONS2.iter().find(|(g, _)| *g == group).map_or(&[], |(_, m)| m)
}
