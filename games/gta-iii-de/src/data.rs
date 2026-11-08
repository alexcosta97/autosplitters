//! Static tables ported from the `startup` block of the ASL.
//!
//! Mission names also act as setting keys, so don't change them.
//!
//! The ASL addresses the script's global variables by the address they have
//! in the 1.0 executable, plus an offset found at runtime. Here they're
//! indexes into the script space (4 bytes each), so `g` turns the ASL's
//! addresses into them and the tables keep the ASL's numbers.

use crate::version::Shift;

/// Where the ASL's addresses start counting (the script space of 1.0).
const SCRIPT_SPACE: u32 = 0x4E19540;

/// The index of the global the ASL finds at `address`.
pub const fn g(address: u32) -> u32 {
    (address - SCRIPT_SPACE) / 4
}

/// Mission givers and their missions, in the ASL's order: the global counting
/// the mission as passed (1 once it is) and its name. These globals sit
/// `Shift::Mission` further on newer versions.
#[rustfmt::skip]
pub static MISSIONS: &[(&str, &[(u32, &str)])] = &[
    ("Luigi Goterelli", &[
        (g(0x4E1993C), "Give Me Liberty & Luigi's Girls"),
        (g(0x4E19940), "Don't Spank Ma Bitch Up"),
        (g(0x4E19944), "Drive Misty For Me"),
        (g(0x4E19948), "Pump-Action Pimp"),
        (g(0x4E1994C), "The Fuzz Ball"),
    ]),
    ("Joey Leone", &[
        (g(0x4E19954), "Mike Lips Last Lunch"),
        (g(0x4E19958), "Farewell 'Chunky' Lee Chong"),
        (g(0x4E1995C), "Van Heist"),
        (g(0x4E19960), "Cipriani's Chauffeur"),
        (g(0x4E19964), "Dead Skunk in The Trunk"),
        (g(0x4E19968), "The Getaway"),
    ]),
    ("Marty Chonks", &[
        (g(0x4E199E0), "The Crook"),
        (g(0x4E199E4), "The Thieves"),
        (g(0x4E199E8), "The Wife"),
        (g(0x4E199EC), "Her Lover"),
    ]),
    ("Toni Cipriani", &[
        (g(0x4E19970), "Taking Out The Laundry"),
        (g(0x4E19974), "The Pick-up"),
        (g(0x4E19978), "Salvatore's Called A Meeting"),
        (g(0x4E1997C), "Triads and Tribulations"),
        (g(0x4E19980), "Blow Fish"),
    ]),
    ("Salvatore Leone", &[
        (g(0x4E19988), "Chaperone"),
        (g(0x4E1998C), "Cutting The Grass"),
        (g(0x4E19990), "Bomb Da Base: Act I"),
        (g(0x4E19994), "Bomb Da Base: Act II"),
        (g(0x4E19998), "Last Requests"),
    ]),
    ("El Burro", &[
        (g(0x4E19A0C), "Turismo"),
        (g(0x4E199B8), "I Scream, You Scream"),
        (g(0x4E199BC), "Trial By Fire"),
        (g(0x4E199C0), "Big 'n' Veiny"),
    ]),
    ("Asuka Kasen", &[
        (g(0x4E19A4C), "Sayonara Salvatore"),
        (g(0x4E19A50), "Under Surveillance"),
        (g(0x4E19A54), "Paparazzi Purge"),
        (g(0x4E19A58), "Payday for Ray"),
        (g(0x4E19A5C), "Two-Faced Tanner"),
        (g(0x4E19AE4), "Bait"),
        (g(0x4E19AE8), "Espresso-2-Go!"),
        (g(0x4E19AEC), "S.A.M"),
    ]),
    ("Kenji Kasen", &[
        (g(0x4E19A80), "Kanbu Bust-out"),
        (g(0x4E19A84), "Grand Theft Auto"),
        (g(0x4E19A88), "Deal Steal"),
        (g(0x4E19A8C), "Shima"),
        (g(0x4E19A90), "Smack Down"),
    ]),
    ("Ray Machowski", &[
        (g(0x4E19A64), "Silence the Sneak"),
        (g(0x4E19A68), "Arms Shortage"),
        (g(0x4E19A6C), "Evidence Dash"),
        (g(0x4E19A70), "Gone Fishing"),
        (g(0x4E19A74), "Plaster Blaster"),
        (g(0x4E19A78), "Marked Man"),
    ]),
    ("Donald Love", &[
        (g(0x4E19A98), "Liberator"),
        (g(0x4E19A9C), "Waka-Gashira Wipeout!"),
        (g(0x4E19AA0), "A Drop in the Ocean"),
        (g(0x4E19AD0), "Grand Theft Aero"),
        (g(0x4E19AD4), "Escort Service"),
        (g(0x4E19AD8), "Decoy"),
        (g(0x4E19ADC), "Love's Disappearance"),
    ]),
    ("King Courtney", &[
        (g(0x4E19AA8), "Bling-Bling Scramble"),
        (g(0x4E19AAC), "Uzi Rider"),
        (g(0x4E19AB0), "Gangcar Round-up"),
        (g(0x4E19AB4), "Kingdom Come"),
    ]),
    ("Catalina", &[
        (g(0x4E19B1C), "The Exchange"),
    ]),
    ("D-Ice", &[
        (g(0x4E19AF8), "Uzi Money"),
        (g(0x4E19AFC), "Toyminator"),
        (g(0x4E19B00), "Rigged to Blow"),
        (g(0x4E19B04), "Bullion Run"),
        (g(0x4E19B08), "Rumble"),
    ]),
    ("Off-Road Challenges", &[
        (g(0x4E19B44), "Patriot Playground"),
        (g(0x4E19B50), "Multistorey Mayhem"),
        (g(0x4E19B48), "A Ride in the Park"),
        (g(0x4E19B4C), "Gripped!"),
    ]),
    ("RC Toyz Missions", &[
        (g(0x4E19BC0), "Diablo Destruction"),
        (g(0x4E19BC4), "Mafia Massacre"),
        (g(0x4E19BC8), "Rumpo Rampage"),
        (g(0x4E19BCC), "Casino Calamity"),
    ]),
];

/// Thread (script) name of a mission to the mission, for splitting when it
/// starts. "Give Me Liberty & Luigi's Girls" has none.
#[rustfmt::skip]
pub static SCRIPTS: &[(&str, &str)] = &[
    ("luigi2", "Don't Spank Ma Bitch Up"),
    ("luigi3", "Drive Misty For Me"),
    ("luigi4", "Pump-Action Pimp"),
    ("luigi5", "The Fuzz Ball"),
    ("joey1", "Mike Lips Last Lunch"),
    ("joey2", "Farewell 'Chunky' Lee Chong"),
    ("joey3", "Van Heist"),
    ("joey4", "Cipriani's Chauffeur"),
    ("joey5", "Dead Skunk in The Trunk"),
    ("joey6", "The Getaway"),
    ("meat1", "The Crook"),
    ("meat2", "The Thieves"),
    ("meat3", "The Wife"),
    ("meat4", "Her Lover"),
    ("toni1", "Taking Out The Laundry"),
    ("toni2", "The Pick-up"),
    ("toni3", "Salvatore's Called A Meeting"),
    ("toni4", "Triads and Tribulations"),
    ("toni5", "Blow Fish"),
    ("frank1", "Chaperone"),
    ("frank2", "Cutting The Grass"),
    ("frank21", "Bomb Da Base: Act I"),
    ("frank3", "Bomb Da Base: Act II"),
    ("frank4", "Last Requests"),
    ("diablo1", "Turismo"),
    ("diablo2", "I Scream, You Scream"),
    ("diablo3", "Trial By Fire"),
    ("diablo4", "Big 'n' Veiny"),
    ("asuka1", "Sayonara Salvatore"),
    ("asuka2", "Under Surveillance"),
    ("asuka3", "Paparazzi Purge"),
    ("asuka4", "Payday for Ray"),
    ("asuka5", "Two-Faced Tanner"),
    ("asusb1", "Bait"),
    ("asusb2", "Espresso-2-Go!"),
    ("asusb3", "S.A.M"),
    ("kenji1", "Kanbu Bust-out"),
    ("kenji2", "Grand Theft Auto"),
    ("kenji3", "Deal Steal"),
    ("kenji4", "Shima"),
    ("kenji5", "Smack Down"),
    ("ray1", "Silence the Sneak"),
    ("ray2", "Arms Shortage"),
    ("ray3", "Evidence Dash"),
    ("ray4", "Gone Fishing"),
    ("ray5", "Plaster Blaster"),
    ("ray6", "Marked Man"),
    ("love1", "Liberator"),
    ("love2", "Waka-Gashira Wipeout!"),
    ("love3", "A Drop in the Ocean"),
    ("love4", "Grand Theft Aero"),
    ("love5", "Escort Service"),
    ("love6", "Decoy"),
    ("love7", "Love's Disappearance"),
    ("yard1", "Bling-Bling Scramble"),
    ("yard2", "Uzi Rider"),
    ("yard3", "Gangcar Round-up"),
    ("yard4", "Kingdom Come"),
    ("cat1", "The Exchange"),
    ("hood1", "Uzi Money"),
    ("hood2", "Toyminator"),
    ("hood3", "Rigged to Blow"),
    ("hood4", "Bullion Run"),
    ("hood5", "Rumble"),
    ("t4x4_1", "Patriot Playground"),
    ("mayhem", "Multistorey Mayhem"),
    ("t4x4_2", "A Ride in the Park"),
    ("t4x4_3", "Gripped!"),
    ("rc1", "Diablo Destruction"),
    ("rc2", "Mafia Massacre"),
    ("rc3", "Rumpo Rampage"),
    ("rc4", "Casino Calamity"),
];

/// The mission a thread (script) is for.
pub fn script_mission(thread: &str) -> Option<&'static str> {
    SCRIPTS.iter().find(|(script, _)| *script == thread).map(|(_, mission)| *mission)
}

/// A count of collectibles found.
pub struct Collectible {
    /// Also the name of its settings.
    pub kind: &'static str,
    pub index: u32,
    /// How many there are. Finding the last one is what "All" splits on.
    pub max: i32,
    pub shift: Shift,
}

/// In the ASL's order.
pub static COLLECTIBLES: &[Collectible] = &[
    Collectible { kind: "Hidden Packages", index: g(0x4E1A5CC), max: 100, shift: Shift::Package },
    Collectible { kind: "Unique Stunt Jumps", index: g(0x4E1A1A8), max: 20, shift: Shift::Usj },
    Collectible { kind: "Rampages", index: g(0x4E1A2A4), max: 20, shift: Shift::Rampage },
];

/// Changes when control is lost at the end of "The Exchange", which is what
/// the Any% final split waits for, with `TE_HELIPAD` being 1.
pub const TE_TIMER: u32 = g(0x4E19C00);
pub const TE_HELIPAD: u32 = g(0x4E1D93C);

/// The thread (script) of "The Exchange".
pub const EXCHANGE_THREAD: &str = "cat1";

/// The thread (script) of the intro cutscene, which a New Game starts.
pub const INTRO_THREAD: &str = "intro";

#[cfg(test)]
mod tests {
    use super::*;

    fn missions() -> impl Iterator<Item = (u32, &'static str)> {
        MISSIONS.iter().flat_map(|(_, ms)| ms.iter().copied())
    }

    #[test]
    fn addresses_become_indexes() {
        assert_eq!(g(0x4E19540), 0);
        assert_eq!(g(0x4E1993C), 255);
        assert_eq!(TE_TIMER, 432);
    }

    #[test]
    fn scripts_map_to_missions() {
        for (script, mission) in SCRIPTS {
            assert!(missions().any(|(_, m)| m == *mission), "{script}: {mission}");
        }
        assert_eq!(script_mission("frank21"), Some("Bomb Da Base: Act I"));
        assert_eq!(script_mission("intro"), None);
        // Every mission but the first has one.
        let without: Vec<_> =
            missions().filter(|(_, m)| !SCRIPTS.iter().any(|(_, s)| s == m)).collect();
        assert_eq!(without, [(255, "Give Me Liberty & Luigi's Girls")]);
        assert_eq!(missions().count(), SCRIPTS.len() + 1);
    }

    #[test]
    fn indexes_are_unique() {
        let mut indexes: Vec<u32> = missions()
            .map(|(i, _)| i)
            .chain(COLLECTIBLES.iter().map(|c| c.index))
            .chain([TE_TIMER, TE_HELIPAD])
            .collect();
        let count = indexes.len();
        indexes.sort();
        indexes.dedup();
        assert_eq!(indexes.len(), count);
    }

    #[test]
    fn names_are_unique() {
        // They're setting keys, together with the givers'.
        let mut names: Vec<&str> = missions().map(|(_, m)| m).collect();
        names.extend(MISSIONS.iter().map(|(giver, _)| *giver));
        let count = names.len();
        names.sort();
        names.dedup();
        assert_eq!(names.len(), count);
    }
}
