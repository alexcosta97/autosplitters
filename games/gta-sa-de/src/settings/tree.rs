//! The settings widgets, and how each section's dropdown stays in sync with the
//! checkboxes in it.
//!
//! This part doesn't touch `asr`: values come in through [`Source`] and the
//! values to store go out as a list, so it can be tested on the host.
//!
//! The checkboxes decide which splits happen. A section's dropdown is only a
//! bulk control (None unticks every checkbox in it, All ticks them all) and
//! shows whether all, none or some of them are ticked. It never gates
//! anything. Some sections also offer "All done", which unticks every
//! checkbox in it and splits once instead, when everything in it is done
//! (read through the key `{section}#allDone`).

use alloc::{collections::BTreeMap, format, string::String, vec, vec::Vec};

const NONE: &str = "none";
const ALL: &str = "all";
const SELECTION: &str = "selection";
const ALL_DONE: &str = "alldone";

static SECTION_OPTIONS: &[(&str, &str)] = &[(NONE, "None"), (ALL, "All"), (SELECTION, "Selection")];
static ALL_DONE_OPTIONS: &[(&str, &str)] =
    &[(NONE, "None"), (ALL, "All"), (SELECTION, "Selection"), (ALL_DONE, "All done")];
const SECTION_LABEL: &str = "Splits in this section";
const SECTION_TOOLTIP: &str =
    "None unticks every split below, All ticks them all. Ticking or unticking splits below updates this.";
const ALL_DONE_TOOLTIP: &str =
    "None unticks every split below, All ticks them all. All done splits once, \
    when everything below is done. Ticking or unticking splits below updates this.";

static COLLECTIBLE_OPTIONS: &[(&str, &str)] =
    &[("off", "Off"), ("each", "Each"), ("all", "All done"), ("both", "Each and all done")];

pub enum Kind {
    Title { level: u32 },
    Bool { default: bool },
    Choice { options: &'static [(&'static str, &'static str)], default: &'static str },
}

/// A widget, in the order it's shown.
pub struct Widget {
    pub key: String,
    pub label: String,
    pub tooltip: Option<&'static str>,
    pub kind: Kind,
}

struct Section {
    /// Index into the choices.
    choice: usize,
    /// Every checkbox below it, including the ones in sub-sections.
    boxes: Vec<usize>,
    /// Every section below it, however deep.
    inner: Vec<usize>,
    /// Whether it offers "All done".
    all_done: bool,
    /// Whether it defaults to "All done" (and all its checkboxes are off).
    default_all_done: bool,
}

/// What a key passed to [`Tree::get`] reads.
enum Lookup {
    Bool(usize),
    /// `{kind}Each` of a collectible choice.
    Each(usize),
    /// `{kind}All` of a collectible choice.
    All(usize),
    /// `{section}#allDone`: the section is set to "All done".
    AllDone(usize),
}

pub struct Tree {
    pub widgets: Vec<Widget>,
    /// Widget index of each checkbox.
    bools: Vec<usize>,
    /// Widget index of each choice.
    choices: Vec<usize>,
    /// Outer sections come before the sections inside them.
    sections: Vec<Section>,
    lookup: BTreeMap<String, Lookup>,
}

/// The value of every checkbox and choice.
#[derive(Clone, Debug, PartialEq)]
pub struct State {
    bools: Vec<bool>,
    choices: Vec<&'static str>,
}

/// Where the values the user has set come from (the settings map).
pub trait Source {
    fn get_bool(&self, key: &str) -> Option<bool>;
    fn get_string(&self, key: &str) -> Option<String>;
}

/// A value to store in the settings map.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Value {
    Bool(bool),
    Choice(&'static str),
}

fn collectible_option(each: bool, all: bool) -> &'static str {
    match (each, all) {
        (false, false) => "off",
        (true, false) => "each",
        (false, true) => "all",
        (true, true) => "both",
    }
}

/// `(each, all)` for a collectible option.
fn collectible_flags(option: &str) -> (bool, bool) {
    (matches!(option, "each" | "both"), matches!(option, "all" | "both"))
}

impl Tree {
    fn bool_default(&self, i: usize) -> bool {
        match self.widgets[self.bools[i]].kind {
            Kind::Bool { default } => default,
            _ => unreachable!(),
        }
    }

    fn choice_kind(&self, i: usize) -> (&'static [(&'static str, &'static str)], &'static str) {
        match self.widgets[self.choices[i]].kind {
            Kind::Choice { options, default } => (options, default),
            _ => unreachable!(),
        }
    }

    /// The value of every widget, falling back to its default when the user
    /// hasn't set it (or it isn't a valid value).
    pub fn read(&self, source: &impl Source) -> State {
        let bools = (0..self.bools.len())
            .map(|i| {
                source
                    .get_bool(&self.widgets[self.bools[i]].key)
                    .unwrap_or_else(|| self.bool_default(i))
            })
            .collect();
        let choices = (0..self.choices.len())
            .map(|i| {
                let (options, default) = self.choice_kind(i);
                let value = source.get_string(&self.widgets[self.choices[i]].key);
                options
                    .iter()
                    .find(|(option, _)| Some(*option) == value.as_deref())
                    .map_or(default, |(option, _)| option)
            })
            .collect();
        State { bools, choices }
    }

    /// Equivalent of `settings[key]` in ASL. `None` for an unknown key.
    pub fn get(&self, state: &State, key: &str) -> Option<bool> {
        Some(match *self.lookup.get(key)? {
            Lookup::Bool(i) => state.bools[i],
            Lookup::Each(i) => collectible_flags(state.choices[i]).0,
            Lookup::All(i) => collectible_flags(state.choices[i]).1,
            Lookup::AllDone(i) => state.choices[self.sections[i].choice] == ALL_DONE,
        })
    }

    /// What section `i`'s checkboxes add up to. A section with no box ticked
    /// but a sub-section on "All done" still splits, so it shows Selection.
    fn derive(&self, i: usize, bools: &[bool], choices: &[&str]) -> &'static str {
        let section = &self.sections[i];
        let ticked = section.boxes.iter().filter(|&&b| bools[b]).count();
        if ticked == section.boxes.len() {
            ALL
        } else if ticked > 0
            || section.inner.iter().any(|&j| choices[self.sections[j].choice] == ALL_DONE)
        {
            SELECTION
        } else {
            NONE
        }
    }

    /// Apply the dropdowns and checkboxes the user changed since `baseline`
    /// (the values after the previous sync) to each other. With no baseline
    /// (at startup) the checkboxes are the truth and every dropdown is derived
    /// from them, except that "All done" with no box ticked stays.
    ///
    /// Sections are derived innermost first, so a parent sees its
    /// sub-sections' new values.
    pub fn sync(&self, baseline: Option<&State>, current: &State) -> State {
        let mut next = current.clone();
        let Some(base) = baseline else {
            for (i, s) in self.sections.iter().enumerate().rev() {
                let derived = self.derive(i, &next.bools, &next.choices);
                if !(next.choices[s.choice] == ALL_DONE && derived == NONE) {
                    next.choices[s.choice] = derived;
                }
            }
            return next;
        };

        // Dropdowns the user changed win over checkboxes changed in the same
        // tick. Outer sections go first, so a sub-section changed in the same
        // tick still gets its own value.
        let mut settled = vec![false; self.sections.len()];
        for (i, s) in self.sections.iter().enumerate() {
            let value = current.choices[s.choice];
            if value == base.choices[s.choice] {
                continue;
            }
            settled[i] = true;
            next.choices[s.choice] = value;
            let tick = match value {
                NONE | ALL_DONE => false,
                ALL => true,
                _ => continue,
            };
            for &b in &s.boxes {
                next.bools[b] = tick;
            }
            // Only sections without sub-sections offer "All done".
            for &j in &s.inner {
                next.choices[self.sections[j].choice] = value;
                settled[j] = true;
            }
        }

        // Every other section shows what its checkboxes add up to, but only
        // once one of them changes (or a sub-section goes to or from "All
        // done"), so "Selection" with every box ticked stays.
        for (i, s) in self.sections.iter().enumerate().rev() {
            let boxes_changed = s.boxes.iter().any(|&b| next.bools[b] != base.bools[b]);
            let all_done_changed = s.inner.iter().any(|&j| {
                let c = self.sections[j].choice;
                (next.choices[c] == ALL_DONE) != (base.choices[c] == ALL_DONE)
            });
            if !settled[i] && (boxes_changed || all_done_changed) {
                next.choices[s.choice] = self.derive(i, &next.bools, &next.choices);
            }
        }
        next
    }

    /// The checkbox keys in a section, and whether it offers "All done".
    #[cfg(test)]
    pub fn section(&self, key: &str) -> (Vec<&str>, bool) {
        let dropdown = format!("{key}#section");
        let s = self
            .sections
            .iter()
            .find(|s| self.widgets[self.choices[s.choice]].key == dropdown)
            .unwrap();
        let boxes = s.boxes.iter().map(|&b| self.widgets[self.bools[b]].key.as_str()).collect();
        (boxes, s.all_done)
    }

    /// The keys of the sections offering "All done".
    #[cfg(test)]
    pub fn all_done_sections(&self) -> Vec<&str> {
        self.sections
            .iter()
            .filter(|s| s.all_done)
            .map(|s| self.widgets[self.choices[s.choice]].key.strip_suffix("#section").unwrap())
            .collect()
    }

    /// The values to store to get from `from` to `to`.
    pub fn changes(&self, from: &State, to: &State) -> Vec<(&str, Value)> {
        let bools = (0..self.bools.len())
            .filter(|&i| from.bools[i] != to.bools[i])
            .map(|i| (self.widgets[self.bools[i]].key.as_str(), Value::Bool(to.bools[i])));
        let choices = (0..self.choices.len())
            .filter(|&i| from.choices[i] != to.choices[i])
            .map(|i| (self.widgets[self.choices[i]].key.as_str(), Value::Choice(to.choices[i])));
        bools.chain(choices).collect()
    }
}

/// Builds a [`Tree`] in the order the widgets are shown.
pub struct Builder {
    tree: Tree,
    /// The sections the next widgets go into, outermost first.
    open: Vec<usize>,
}

impl Builder {
    pub fn new() -> Self {
        Self {
            tree: Tree {
                widgets: Vec::new(),
                bools: Vec::new(),
                choices: Vec::new(),
                sections: Vec::new(),
                lookup: BTreeMap::new(),
            },
            open: Vec::new(),
        }
    }

    fn push(&mut self, key: String, label: &str, kind: Kind) {
        self.tree.widgets.push(Widget { key, label: label.into(), tooltip: None, kind });
    }

    fn lookup(&mut self, key: String, lookup: Lookup) {
        let duplicate = self.tree.lookup.insert(key, lookup);
        assert!(duplicate.is_none(), "duplicate setting");
    }

    /// A heading that only groups the settings below it. `key` only has to be
    /// unique.
    pub fn heading(&mut self, key: &str, label: &str, level: u32) {
        self.push(format!("{key}#title"), label, Kind::Title { level });
    }

    /// A heading followed by a dropdown for all the checkboxes added in
    /// `body`.
    pub fn section(&mut self, key: &str, label: &str, level: u32, body: impl FnOnce(&mut Self)) {
        self.add_section(key, label, level, false, body);
    }

    /// A section that also offers "All done". It can't contain sections.
    pub fn section_all_done(
        &mut self,
        key: &str,
        label: &str,
        level: u32,
        body: impl FnOnce(&mut Self),
    ) {
        self.add_section(key, label, level, true, body);
    }

    fn add_section(
        &mut self,
        key: &str,
        label: &str,
        level: u32,
        all_done: bool,
        body: impl FnOnce(&mut Self),
    ) {
        self.heading(key, label, level);
        let section = self.tree.sections.len();
        for &s in &self.open {
            assert!(
                !self.tree.sections[s].all_done,
                "section with \"All done\" can't contain {key}"
            );
            self.tree.sections[s].inner.push(section);
        }
        self.tree.sections.push(Section {
            choice: self.tree.choices.len(),
            boxes: Vec::new(),
            inner: Vec::new(),
            all_done,
            default_all_done: false,
        });
        self.tree.choices.push(self.tree.widgets.len());
        if all_done {
            self.lookup(format!("{key}#allDone"), Lookup::AllDone(section));
        }
        // The default is derived from the checkboxes' defaults in `finish`.
        let (options, tooltip) = if all_done {
            (ALL_DONE_OPTIONS, ALL_DONE_TOOLTIP)
        } else {
            (SECTION_OPTIONS, SECTION_TOOLTIP)
        };
        self.push(format!("{key}#section"), SECTION_LABEL, Kind::Choice { options, default: NONE });
        self.tooltip(tooltip);

        self.open.push(section);
        body(self);
        self.open.pop();
    }

    /// A checkbox. The label defaults to the key.
    pub fn check(&mut self, key: &str, label: Option<&str>, default: bool) {
        let i = self.tree.bools.len();
        for &s in &self.open {
            self.tree.sections[s].boxes.push(i);
        }
        self.tree.bools.push(self.tree.widgets.len());
        self.lookup(key.into(), Lookup::Bool(i));
        self.push(key.into(), label.unwrap_or(key), Kind::Bool { default });
    }

    /// A choice between splitting on each collectible of this kind, on all of
    /// them being done, both or neither. Read through the keys `{kind}Each`
    /// and `{kind}All`, which were checkboxes in the ASL.
    pub fn collectible(&mut self, kind: &str) {
        let i = self.tree.choices.len();
        self.tree.choices.push(self.tree.widgets.len());
        self.lookup(format!("{kind}Each"), Lookup::Each(i));
        self.lookup(format!("{kind}All"), Lookup::All(i));
        self.push(
            format!("{kind}#splits"),
            kind,
            Kind::Choice { options: COLLECTIBLE_OPTIONS, default: "off" },
        );
    }

    /// Set the tooltip of the widget added last.
    pub fn tooltip(&mut self, tooltip: &'static str) {
        self.tree.widgets.last_mut().unwrap().tooltip = Some(tooltip);
    }

    /// Change the default of a key [`Tree::get`] reads.
    pub fn set_default(&mut self, key: &str, value: bool) {
        let tree = &mut self.tree;
        let widget = match tree.lookup[key] {
            Lookup::Bool(i) => tree.bools[i],
            Lookup::Each(i) | Lookup::All(i) => tree.choices[i],
            Lookup::AllDone(i) => {
                tree.sections[i].default_all_done = value;
                return;
            }
        };
        match (&tree.lookup[key], &mut tree.widgets[widget].kind) {
            (Lookup::Bool(_), Kind::Bool { default }) => *default = value,
            (Lookup::Each(_), Kind::Choice { default, .. }) => {
                *default = collectible_option(value, collectible_flags(default).1);
            }
            (Lookup::All(_), Kind::Choice { default, .. }) => {
                *default = collectible_option(collectible_flags(default).0, value);
            }
            _ => unreachable!(),
        }
    }

    /// Derive each section's default from its checkboxes' defaults.
    pub fn finish(mut self) -> Tree {
        let tree = &mut self.tree;
        let bools: Vec<bool> = (0..tree.bools.len()).map(|i| tree.bool_default(i)).collect();
        let mut choices: Vec<&'static str> =
            (0..tree.choices.len()).map(|i| tree.choice_kind(i).1).collect();
        for i in (0..tree.sections.len()).rev() {
            let derived = tree.derive(i, &bools, &choices);
            let s = &tree.sections[i];
            choices[s.choice] = if s.default_all_done {
                // Otherwise startup would change it.
                assert_eq!(derived, NONE, "\"All done\" by default with checkboxes ticked");
                ALL_DONE
            } else {
                derived
            };
            if let Kind::Choice { default, .. } = &mut tree.widgets[tree.choices[s.choice]].kind {
                *default = choices[s.choice];
            }
        }
        self.tree
    }
}

#[cfg(test)]
impl Source for BTreeMap<String, Value> {
    fn get_bool(&self, key: &str) -> Option<bool> {
        match self.get(key)? {
            Value::Bool(b) => Some(*b),
            Value::Choice(_) => None,
        }
    }

    fn get_string(&self, key: &str) -> Option<String> {
        match self.get(key)? {
            Value::Choice(c) => Some((*c).into()),
            Value::Bool(_) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A section "A" holding "a1", "a2" and a sub-section "B" ("b1", "b2"),
    /// a section "C" ("c1"), a collectible, a section "R" holding two
    /// sub-sections offering "All done" ("R1": "r1a", "r1b"; "R2": "r2a"), and
    /// a section "E" ("e1") on "All done" by default.
    fn tree() -> Tree {
        let mut b = Builder::new();
        b.heading("Top", "Top", 0);
        b.section("A", "A", 1, |b| {
            b.check("a1", None, true);
            b.check("a2", None, false);
            b.section("B", "B", 2, |b| {
                b.check("b1", None, true);
                b.check("b2", None, true);
            });
        });
        b.section("C", "C", 1, |b| b.check("c1", None, true));
        b.collectible("Photos");
        b.set_default("PhotosEach", true);
        b.section("R", "R", 1, |b| {
            b.section_all_done("R1", "R1", 2, |b| {
                b.check("r1a", None, false);
                b.check("r1b", None, false);
            });
            b.section_all_done("R2", "R2", 2, |b| b.check("r2a", None, false));
        });
        b.section_all_done("E", "E", 1, |b| b.check("e1", None, false));
        b.set_default("E#allDone", true);
        b.finish()
    }

    /// Simulates the splitter: the settings map, and the values it saw on the
    /// previous tick.
    struct Sim {
        tree: Tree,
        map: BTreeMap<String, Value>,
        baseline: Option<State>,
    }

    impl Sim {
        fn new() -> Self {
            Self { tree: tree(), map: BTreeMap::new(), baseline: None }
        }

        fn set(&mut self, key: &str, value: Value) -> &mut Self {
            self.map.insert(key.into(), value);
            self
        }

        /// One tick. Returns the keys stored.
        fn tick(&mut self) -> Vec<String> {
            let current = self.tree.read(&self.map);
            let next = self.tree.sync(self.baseline.as_ref(), &current);
            let changes: Vec<(String, Value)> = self
                .tree
                .changes(&current, &next)
                .into_iter()
                .map(|(k, v)| (String::from(k), v))
                .collect();
            let mut keys = Vec::new();
            for (key, value) in changes {
                keys.push(key.clone());
                self.map.insert(key, value);
            }
            self.baseline = Some(next);
            keys.sort();
            keys
        }

        fn get(&self, key: &str) -> bool {
            let state = self.tree.read(&self.map);
            self.tree.get(&state, key).unwrap()
        }

        fn choice(&self, key: &str) -> &'static str {
            let state = self.tree.read(&self.map);
            let i =
                self.tree.choices.iter().position(|&w| self.tree.widgets[w].key == key).unwrap();
            state.choices[i]
        }
    }

    fn bools(sim: &Sim) -> [bool; 5] {
        ["a1", "a2", "b1", "b2", "c1"].map(|k| sim.get(k))
    }

    #[test]
    fn section_defaults_come_from_their_checkboxes() {
        let sim = Sim::new();
        assert_eq!(sim.choice("A#section"), SELECTION);
        assert_eq!(sim.choice("B#section"), ALL);
        assert_eq!(sim.choice("C#section"), ALL);
        assert_eq!(sim.choice("Photos#splits"), "each");
    }

    #[test]
    fn startup_with_defaults_stores_nothing() {
        let mut sim = Sim::new();
        assert!(sim.tick().is_empty());
        assert!(sim.tick().is_empty());
    }

    #[test]
    fn startup_corrects_dropdowns_from_checkboxes() {
        let mut sim = Sim::new();
        sim.set("B#section", Value::Choice(NONE))
            .set("C#section", Value::Choice(SELECTION))
            .set("a2", Value::Bool(true));
        assert_eq!(sim.tick(), ["A#section", "B#section", "C#section"]);
        assert_eq!(bools(&sim), [true, true, true, true, true]);
        assert_eq!(sim.choice("A#section"), ALL);
        assert_eq!(sim.choice("B#section"), ALL);
        assert_eq!(sim.choice("C#section"), ALL);
    }

    #[test]
    fn none_unticks_everything_below() {
        let mut sim = Sim::new();
        sim.tick();
        sim.set("A#section", Value::Choice(NONE));
        assert_eq!(sim.tick(), ["B#section", "a1", "b1", "b2"]);
        assert_eq!(bools(&sim), [false, false, false, false, true]);
        assert_eq!(sim.choice("A#section"), NONE);
        assert_eq!(sim.choice("B#section"), NONE);
        // Its own writes aren't changes on the next tick.
        assert!(sim.tick().is_empty());
    }

    #[test]
    fn all_ticks_everything_below() {
        let mut sim = Sim::new();
        sim.set("b2", Value::Bool(false)).set("b1", Value::Bool(false));
        sim.tick();
        assert_eq!(sim.choice("B#section"), NONE);
        sim.set("A#section", Value::Choice(ALL));
        assert_eq!(sim.tick(), ["B#section", "a2", "b1", "b2"]);
        assert_eq!(bools(&sim), [true; 5]);
        assert_eq!(sim.choice("B#section"), ALL);
        assert!(sim.tick().is_empty());
    }

    #[test]
    fn all_sets_nested_dropdowns_even_if_their_boxes_are_ticked() {
        let mut sim = Sim::new();
        sim.tick();
        // "Selection" with every box ticked is left alone...
        sim.set("B#section", Value::Choice(SELECTION));
        assert!(sim.tick().is_empty());
        assert_eq!(sim.choice("B#section"), SELECTION);
        // ...until the parent is set to All.
        sim.set("A#section", Value::Choice(ALL));
        assert_eq!(sim.tick(), ["B#section", "a2"]);
        assert_eq!(sim.choice("B#section"), ALL);
    }

    #[test]
    fn selection_changes_nothing() {
        let mut sim = Sim::new();
        sim.tick();
        sim.set("C#section", Value::Choice(SELECTION));
        assert!(sim.tick().is_empty());
        assert_eq!(bools(&sim), [true, false, true, true, true]);
        assert_eq!(sim.choice("C#section"), SELECTION);
        assert!(sim.tick().is_empty());
    }

    #[test]
    fn checkboxes_update_their_sections() {
        let mut sim = Sim::new();
        sim.tick();
        sim.set("c1", Value::Bool(false));
        assert_eq!(sim.tick(), ["C#section"]);
        assert_eq!(sim.choice("C#section"), NONE);
        sim.set("b1", Value::Bool(false));
        assert_eq!(sim.tick(), ["B#section"]);
        assert_eq!(sim.choice("B#section"), SELECTION);
        assert_eq!(sim.choice("A#section"), SELECTION);
        sim.set("b2", Value::Bool(false));
        assert_eq!(sim.tick(), ["B#section"]);
        assert_eq!(sim.choice("B#section"), NONE);
        assert_eq!(sim.choice("A#section"), SELECTION);
    }

    #[test]
    fn ticking_the_last_box_updates_section_and_parent() {
        let mut sim = Sim::new();
        sim.set("a2", Value::Bool(true)).set("b2", Value::Bool(false));
        sim.tick();
        assert_eq!(sim.choice("A#section"), SELECTION);
        assert_eq!(sim.choice("B#section"), SELECTION);
        sim.set("b2", Value::Bool(true));
        assert_eq!(sim.tick(), ["A#section", "B#section"]);
        assert_eq!(sim.choice("A#section"), ALL);
        assert_eq!(sim.choice("B#section"), ALL);
    }

    #[test]
    fn nested_bulk_change_updates_parent() {
        let mut sim = Sim::new();
        sim.set("a2", Value::Bool(true));
        sim.tick();
        assert_eq!(sim.choice("A#section"), ALL);
        sim.set("B#section", Value::Choice(NONE));
        assert_eq!(sim.tick(), ["A#section", "b1", "b2"]);
        assert_eq!(sim.choice("A#section"), SELECTION);
        assert_eq!(bools(&sim), [true, true, false, false, true]);
    }

    #[test]
    fn dropdown_wins_over_checkboxes_in_the_same_tick() {
        let mut sim = Sim::new();
        sim.tick();
        sim.set("A#section", Value::Choice(NONE)).set("a2", Value::Bool(true));
        sim.tick();
        assert_eq!(bools(&sim), [false, false, false, false, true]);
        assert_eq!(sim.choice("A#section"), NONE);

        // "Selection" keeps the checkbox change, and stays.
        sim.set("A#section", Value::Choice(SELECTION)).set("a1", Value::Bool(true));
        assert!(sim.tick().is_empty());
        assert_eq!(sim.choice("A#section"), SELECTION);
        assert!(sim.get("a1"));
    }

    #[test]
    fn nested_dropdown_changed_with_its_parent_keeps_its_value() {
        let mut sim = Sim::new();
        sim.set("b1", Value::Bool(false)).set("b2", Value::Bool(false));
        sim.tick();
        sim.set("A#section", Value::Choice(NONE)).set("B#section", Value::Choice(ALL));
        sim.tick();
        assert_eq!(bools(&sim), [false, false, true, true, true]);
        assert_eq!(sim.choice("A#section"), NONE);
        assert_eq!(sim.choice("B#section"), ALL);
    }

    #[test]
    fn invalid_values_fall_back_to_defaults() {
        let mut sim = Sim::new();
        sim.set("A#section", Value::Bool(true))
            .set("a1", Value::Choice(ALL))
            .set("Photos#splits", Value::Choice("sometimes"));
        assert!(sim.tick().is_empty());
        assert!(sim.get("a1"));
        assert_eq!(sim.choice("A#section"), SELECTION);
        assert_eq!(sim.choice("Photos#splits"), "each");
    }

    #[test]
    fn collectibles_read_through_each_and_all() {
        let mut sim = Sim::new();
        for (option, each, all) in [
            ("off", false, false),
            ("each", true, false),
            ("all", false, true),
            ("both", true, true),
        ] {
            sim.set("Photos#splits", Value::Choice(option));
            assert!(sim.tick().is_empty());
            assert_eq!(sim.get("PhotosEach"), each, "{option}");
            assert_eq!(sim.get("PhotosAll"), all, "{option}");
        }
        assert_eq!(sim.tree.get(&sim.tree.read(&sim.map), "Photos"), None);
    }

    #[test]
    fn collectible_defaults() {
        let mut b = Builder::new();
        b.collectible("Tags");
        b.set_default("TagsAll", true);
        b.set_default("TagsEach", true);
        b.set_default("TagsEach", false);
        let tree = b.finish();
        let state = tree.read(&BTreeMap::new());
        assert_eq!(state.choices, ["all"]);
    }

    #[test]
    fn all_done_default_is_kept() {
        let mut sim = Sim::new();
        assert!(sim.tick().is_empty());
        assert_eq!(sim.choice("E#section"), ALL_DONE);
        assert_eq!(sim.choice("R#section"), NONE);
        assert!(sim.get("E#allDone"));
        assert!(!sim.get("R1#allDone"));
    }

    #[test]
    fn all_done_unticks_the_section() {
        let mut sim = Sim::new();
        sim.set("r1a", Value::Bool(true));
        assert_eq!(sim.tick(), ["R#section", "R1#section"]);
        assert_eq!(sim.choice("R1#section"), SELECTION);
        sim.set("R1#section", Value::Choice(ALL_DONE));
        assert_eq!(sim.tick(), ["r1a"]);
        assert!(!sim.get("r1a"));
        assert!(sim.get("R1#allDone"));
        // The parent still splits, through R1.
        assert_eq!(sim.choice("R#section"), SELECTION);
        assert!(sim.tick().is_empty());
    }

    #[test]
    fn ticking_a_box_leaves_all_done() {
        let mut sim = Sim::new();
        sim.tick();
        sim.set("e1", Value::Bool(true));
        assert_eq!(sim.tick(), ["E#section"]);
        assert_eq!(sim.choice("E#section"), ALL);

        sim.set("R1#section", Value::Choice(ALL_DONE));
        sim.tick();
        sim.set("r1b", Value::Bool(true));
        sim.tick();
        assert_eq!(sim.choice("R1#section"), SELECTION);
        assert!(!sim.get("R1#allDone"));
    }

    #[test]
    fn startup_keeps_all_done_only_without_boxes() {
        let mut sim = Sim::new();
        sim.set("R2#section", Value::Choice(ALL_DONE))
            .set("R1#section", Value::Choice(ALL_DONE))
            .set("r1a", Value::Bool(true));
        assert_eq!(sim.tick(), ["R#section", "R1#section"]);
        assert_eq!(sim.choice("R1#section"), SELECTION);
        assert_eq!(sim.choice("R2#section"), ALL_DONE);
        assert_eq!(sim.choice("R#section"), SELECTION);
    }

    #[test]
    fn parent_shows_selection_while_a_sub_section_is_all_done() {
        let mut sim = Sim::new();
        sim.tick();
        sim.set("R2#section", Value::Choice(ALL_DONE));
        assert_eq!(sim.tick(), ["R#section"]);
        assert_eq!(sim.choice("R#section"), SELECTION);
        sim.set("R2#section", Value::Choice(NONE));
        assert_eq!(sim.tick(), ["R#section"]);
        assert_eq!(sim.choice("R#section"), NONE);
    }

    #[test]
    fn parent_none_or_all_overrides_all_done() {
        let mut sim = Sim::new();
        sim.tick();
        sim.set("R2#section", Value::Choice(ALL_DONE));
        sim.tick();
        sim.set("R#section", Value::Choice(ALL));
        sim.tick();
        assert!(sim.get("r1a") && sim.get("r1b") && sim.get("r2a"));
        assert_eq!(sim.choice("R1#section"), ALL);
        assert_eq!(sim.choice("R2#section"), ALL);

        sim.set("R2#section", Value::Choice(ALL_DONE));
        sim.tick();
        assert!(!sim.get("r2a"));
        assert_eq!(sim.choice("R#section"), SELECTION);
        sim.set("R#section", Value::Choice(NONE));
        sim.tick();
        assert!(!sim.get("r1a") && !sim.get("r1b") && !sim.get("r2a"));
        assert_eq!(sim.choice("R2#section"), NONE);
        assert!(!sim.get("R2#allDone"));
    }

    #[test]
    fn all_done_is_rejected_where_not_offered() {
        let mut sim = Sim::new();
        sim.set("A#section", Value::Choice(ALL_DONE));
        assert!(sim.tick().is_empty());
        assert_eq!(sim.choice("A#section"), SELECTION);
        assert_eq!(sim.tree.get(&sim.tree.read(&sim.map), "A#allDone"), None);
    }
}
