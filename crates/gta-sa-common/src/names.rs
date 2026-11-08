//! Names shared by the GTA San Andreas auto splitters. Mission names also act
//! as setting keys, so don't change them.

// Story missions, in the order the ASL listed them.
#[rustfmt::skip]
pub static LS_MISSIONS: &[&str] = &[
    "Big Smoke", "Ryder", "Tagging up Turf", "Cleaning the Hood", "Drive-Thru",
    "Nines and AKs", "OG Loc", "Life's a Beach", "Running Dog", "Drive-By",
    "Sweet's Girl", "Cesar Vialpando", "High Stakes Lowrider", "Madd Dogg's Rhymes",
    "Management Issues", "House Party (Cutscene)", "Burning Desire",
    "Wrong Side of the Tracks", "Just Business", "Doberman", "Gray Imports",
    "Home Invasion", "House Party", "Catalyst", "Robbing Uncle Sam", "Los Sepulcros",
    "Reuniting the Families", "The Green Sabre",
];
#[rustfmt::skip]
pub static BL_MISSIONS: &[&str] = &[
    "Badlands", "Tanker Commander", "Body Harvest", "King in Exile",
    "Small Town Bank", "Local Liquor Store", "Against All Odds", "Wu Zi Mu",
    "Farewell, My Love", "Are You Going To San Fierro?",
];
#[rustfmt::skip]
pub static SF_MISSIONS: &[&str] = &[
    "Wear Flowers in your Hair", "555 WE TIP", "Deconstruction", "Photo Opportunity",
    "Jizzy (Cutscene)", "Jizzy", "T-Bone Mendez", "Mike Toreno", "Outrider",
    "Snail Trail", "Mountain Cloud Boys", "Ran Fa Li", "Lure", "Ice Cold Killa",
    "Amphibious Assault", "Pier 69", "Toreno's Last Flight", "The Da Nang Thang",
    "Yay Ka-Boom-Boom",
];
#[rustfmt::skip]
pub static DESERT_MISSIONS: &[&str] = &[
    "Monster", "Highjack", "Interdiction", "Verdant Meadows", "Learning to Fly",
];
#[rustfmt::skip]
pub static LV_MISSIONS: &[&str] = &[
    "N.O.E.", "Freefall", "Fender Ketchup", "Explosive Situation", "You've Had Your Chips",
    "Don Peyote", "Intensive Care", "The Meat Business", "Fish in a Barrel", "Madd Dogg",
    "Misappropriation", "Stowaway", "Black Project", "High Noon", "Green Goo",
    "Saint Mark's Bistro",
];
#[rustfmt::skip]
pub static RTLS_MISSIONS: &[&str] = &[
    "A Home in the Hills", "Vertical Bird", "Home Coming", "Beat Down on B Dup",
    "Grove 4 Life", "Cut Throat Business", "Riot", "Los Desperados",
];

#[rustfmt::skip]
pub static PROPERTIES: &[(&str, &[&str])] = &[
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

#[rustfmt::skip]
pub static EXPORT_LISTS: [[&str; 10]; 3] = [
    ["Buffalo", "Sentinel", "Infernus", "Camper", "Admiral",
     "Patriot", "Sanchez", "Stretch", "Feltzer", "Remington"],
    ["Cheetah", "Rancher", "Stallion", "Tanker", "Comet",
     "Slamvan", "Blista Compact", "Stafford", "Sabre", "FCR-900"],
    ["Banshee", "Super GT", "Journey", "Huntley", "BF Injection",
     "Blade", "Freeway", "Mesa", "ZR-350", "Euros"],
];

/// Collectible types, which act as setting IDs.
pub const COLLECTIBLE_KINDS: [&str; 5] =
    ["Photos", "Tags", "Oysters", "Horseshoes", "Stunts (Completed)"];
