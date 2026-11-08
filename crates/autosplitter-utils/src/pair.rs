//! A value and the one it had on the previous tick.

#[derive(Copy, Clone, Default, Debug, PartialEq)]
pub struct Pair<T> {
    pub old: T,
    pub current: T,
}

impl<T: Copy> Pair<T> {
    /// Both the old and the current value are `value`.
    pub fn new(value: T) -> Self {
        Self { old: value, current: value }
    }

    /// Keeps the previous value if the read failed, so a failed read can't
    /// look like a value change.
    pub fn update(&mut self, value: Option<T>) {
        self.old = self.current;
        if let Some(value) = value {
            self.current = value;
        }
    }

    pub fn pair(self) -> (T, T) {
        (self.old, self.current)
    }

    pub fn changed(&self) -> bool
    where
        T: PartialEq,
    {
        self.old != self.current
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn failed_read_keeps_the_value() {
        let mut p = Pair::new(5);
        p.update(None);
        assert_eq!(p.pair(), (5, 5));
        p.update(Some(7));
        assert_eq!(p.pair(), (5, 7));
        assert!(p.changed());
        p.update(None);
        assert_eq!(p.pair(), (7, 7));
        assert!(!p.changed());
    }
}
