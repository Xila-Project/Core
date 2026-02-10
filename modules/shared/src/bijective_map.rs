use core::fmt::Debug;

use alloc::collections::btree_map::BTreeMap;

pub struct BijectiveBTreeMap<L, R> {
    to_left: BTreeMap<R, L>,
    to_right: BTreeMap<L, R>,
}

impl<L: Ord + Copy, R: Ord + Copy> BijectiveBTreeMap<L, R> {
    pub const fn new() -> Self {
        Self {
            to_left: BTreeMap::new(),
            to_right: BTreeMap::new(),
        }
    }

    pub fn insert(&mut self, key: L, value: R) {
        self.to_left.insert(value, key);
        self.to_right.insert(key, value);
    }

    pub fn get_by_left(&self, key: &L) -> Option<&R> {
        self.to_right.get(key)
    }

    pub fn get_by_right(&self, value: &R) -> Option<&L> {
        self.to_left.get(value)
    }

    pub fn remove_by_key(&mut self, key: &L) -> Option<R> {
        if let Some(value) = self.to_right.remove(key) {
            self.to_left.remove(&value);
            Some(value)
        } else {
            None
        }
    }

    pub fn remove_by_value(&mut self, value: &R) -> Option<L> {
        if let Some(key) = self.to_left.remove(value) {
            self.to_right.remove(&key);
            Some(key)
        } else {
            None
        }
    }

    pub fn get_left_keys(&self) -> impl Iterator<Item = &L> {
        self.to_right.keys()
    }

    pub fn get_right_keys(&self) -> impl Iterator<Item = &R> {
        self.to_left.keys()
    }
}

impl<L: Ord + Copy, R: Ord + Copy> Default for BijectiveBTreeMap<L, R> {
    fn default() -> Self {
        Self::new()
    }
}

impl<L: Ord + Copy + Debug, R: Ord + Copy + Debug> Debug for BijectiveBTreeMap<L, R> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("BijectiveBTreeMap")
            .field("to_key", &self.to_left)
            .field("to_value", &self.to_right)
            .finish()
    }
}

impl<L: Ord + Copy, R: Ord + Copy> Clone for BijectiveBTreeMap<L, R> {
    fn clone(&self) -> Self {
        Self {
            to_left: self.to_left.clone(),
            to_right: self.to_right.clone(),
        }
    }
}
