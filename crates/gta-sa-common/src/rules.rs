//! The conditions deciding when a split happens, lifted from the ASL's
//! `split` block. Values are `(old, current)` pairs.

/// The mission a counter just reached, if it went up to a value in `missions`.
pub fn counter_split<'a>(missions: &'a [(i32, &'a str)], value: (i32, i32)) -> Option<&'a str> {
    let (old, current) = value;
    if current > old {
        missions.iter().find(|(v, _)| *v == current).map(|(_, id)| *id)
    } else {
        None
    }
}

/// A flag that went from 0 to above 0 (some go 0 -> 2).
pub fn flag_set(value: (i32, i32)) -> bool {
    value.1 > 0 && value.0 == 0
}

/// The value of a collectible's count when all of them are collected.
pub fn collectible_max(kind: &str) -> i32 {
    match kind {
        "Tags" => 100,
        "Stunts (Completed)" => 70,
        _ => 50,
    }
}

/// The Chiliad Challenge race done by this change, if any.
///
/// "chiliadRace" contains the next race to be started (1-3), but also repeats
/// when you do the races again (changes to 1 on finishing the last race).
/// "chiliadDone" changes from 0 to 1 when all races have been done.
pub fn chiliad_race_done(race: (i32, i32), done: (i32, i32)) -> Option<i32> {
    if (race.1 > race.0 && race.1 > 1 && done.1 == 0) || done.1 > done.0 {
        Some(if done.1 == 1 { 3 } else { race.1 - 1 })
    } else {
        None
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Exported {
    /// Index of the export list, 0-2.
    pub list: usize,
    /// Index of the vehicle exported, 0-9.
    pub vehicle: usize,
    /// Whether all vehicles of the list are exported.
    pub all_done: bool,
}

/// The vehicle just exported, if any.
///
/// The three lists all contain 10 vehicles, which have their exported state
/// stored in an array, so basically 10 values that change from 0 to 1 when
/// that car is exported. This is per list, so which vehicles the values refer
/// to changes based on which list is active.
pub fn exported(list: i32, vehicles: [(i32, i32); 10]) -> Option<Exported> {
    if !(0..=2).contains(&list) {
        return None;
    }
    let mut all_done = true;
    let mut exported = None;
    for (i, (old, current)) in vehicles.into_iter().enumerate() {
        if current == 1 && old == 0 {
            exported = Some(i);
        }
        if current == 0 {
            all_done = false;
        }
    }
    exported.map(|vehicle| Exported { list: list as usize, vehicle, all_done })
}

/// For a mission, the counter value it needs and the counter's current value,
/// or `None` if the mission isn't in `table` (the name has to be exact). The
/// mission is passed if current >= required.
pub fn passed<K: Copy>(
    table: &[(K, &[(i32, &str)])],
    mission: &str,
    current: impl Fn(K) -> i32,
) -> Option<(i32, i32)> {
    for (key, missions) in table {
        if let Some((value, _)) = missions.iter().find(|(_, m)| *m == mission) {
            return Some((*value, current(*key)));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counter_split_names_the_mission_reached() {
        let missions = [(1, "A"), (2, "B")];
        assert_eq!(counter_split(&missions, (1, 2)), Some("B"));
        assert_eq!(counter_split(&missions, (2, 2)), None);
        assert_eq!(counter_split(&missions, (2, 1)), None);
        assert_eq!(counter_split(&missions, (2, 5)), None);
    }

    #[test]
    fn flag_set_needs_zero_to_above_zero() {
        assert!(flag_set((0, 2)));
        assert!(!flag_set((1, 2)));
        assert!(!flag_set((0, 0)));
    }

    #[test]
    fn collectible_maxes() {
        assert_eq!(collectible_max("Tags"), 100);
        assert_eq!(collectible_max("Stunts (Completed)"), 70);
        assert_eq!(collectible_max("Photos"), 50);
    }

    #[test]
    fn chiliad_races() {
        assert_eq!(chiliad_race_done((1, 2), (0, 0)), Some(1));
        assert_eq!(chiliad_race_done((2, 3), (0, 0)), Some(2));
        assert_eq!(chiliad_race_done((3, 1), (0, 1)), Some(3));
        assert_eq!(chiliad_race_done((1, 1), (0, 1)), Some(3));
        assert_eq!(chiliad_race_done((2, 1), (0, 0)), None);
        assert_eq!(chiliad_race_done((0, 1), (0, 0)), None);
    }

    fn vehicles(set: &[(usize, (i32, i32))]) -> [(i32, i32); 10] {
        let mut v = [(1, 1); 10];
        for (i, pair) in set {
            v[*i] = *pair;
        }
        v
    }

    #[test]
    fn exported_vehicle() {
        assert_eq!(exported(3, vehicles(&[(4, (0, 1))])), None);
        assert_eq!(exported(-1, vehicles(&[(4, (0, 1))])), None);
        assert_eq!(
            exported(1, vehicles(&[(4, (0, 1))])),
            Some(Exported { list: 1, vehicle: 4, all_done: true })
        );
        assert_eq!(
            exported(1, vehicles(&[(4, (0, 1)), (6, (0, 0))])),
            Some(Exported { list: 1, vehicle: 4, all_done: false })
        );
        assert_eq!(exported(1, vehicles(&[])), None);
        assert_eq!(exported(0, vehicles(&[(2, (0, 1)), (7, (0, 1))])).map(|e| e.vehicle), Some(7));
    }

    #[test]
    fn passed_reports_required_and_current() {
        let table: &[(u32, &[(i32, &str)])] = &[(10, &[(1, "A"), (2, "B")]), (20, &[(1, "C")])];
        assert_eq!(passed(table, "B", |k| k as i32 + 1), Some((2, 11)));
        assert_eq!(passed(table, "C", |k| k as i32), Some((1, 20)));
        assert_eq!(passed(table, "Z", |_| 0), None);
    }
}
