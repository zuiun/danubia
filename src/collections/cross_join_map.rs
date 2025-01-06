use std::collections::{HashMap, HashSet};
use std::hash::Hash;

/*
 * Map that behaves like a cross join:
 * Each T maps to many U
 * Each U maps to many T
 * Mappings are duplicated
 */
#[derive (Debug)]
pub struct CrossJoinMap<T, U> {
    map_first: HashMap<T, HashSet<U>>,
    map_second: HashMap<U, HashSet<T>>,
}

impl<T, U> CrossJoinMap<T, U>
where T: Clone + Eq + Hash, U: Clone + Eq + Hash {
    pub fn new () -> Self {
        let map_first: HashMap<T, HashSet<U>> = HashMap::new ();
        let map_second: HashMap<U, HashSet<T>> = HashMap::new ();

        Self { map_first, map_second }
    }

    pub fn insert (&mut self, values: (T, U)) -> bool {
        let is_inserted_first: bool = self.map_first.entry (values.0.clone ())
                .or_default ()
                .insert (values.1.clone ());
        let is_inserted_second: bool = self.map_second.entry (values.1)
                .or_default ()
                .insert (values.0);

        is_inserted_first || is_inserted_second
    }

    pub fn get_first (&self, key: &T) -> Option<&HashSet<U>> {
        self.map_first.get (key)
    }

    pub fn get_second (&self, key: &U) -> Option<&HashSet<T>> {
        self.map_second.get (key)
    }

    pub fn remove (&mut self, key_first: &T, key_second: &U) -> bool {
        let collection_first: &mut HashSet<U> = match self.map_first.get_mut (key_first) {
            Some (c) => c,
            None => return false,
        };
        let is_removed_first: bool = collection_first.remove (key_second);
        let collection_second: &mut HashSet<T> = match self.map_second.get_mut (key_second) {
            Some (c) => c,
            None => return false,
        };
        let is_removed_second: bool = collection_second.remove (key_first);

        assert_eq! (is_removed_first, is_removed_second);

        is_removed_first && is_removed_second
    }

    pub fn remove_first (&mut self, key_first: &T) -> bool {
        let is_removed_first: bool = self.map_first.remove (key_first).is_some ();
        let mut is_removed_second: bool = false;

        for (_, collection_second) in self.map_second.iter_mut () {
            is_removed_second |= collection_second.remove (key_first);
        }

        assert_eq! (is_removed_first, is_removed_second);

        is_removed_first && is_removed_second
    }

    pub fn remove_second (&mut self, key_second: &U) -> bool {
        let is_removed_second: bool = self.map_second.remove (key_second).is_some ();
        let mut is_removed_first: bool = false;

        for (_, collection_first) in self.map_first.iter_mut () {
            is_removed_first |= collection_first.remove (key_second);
        }

        assert_eq! (is_removed_first, is_removed_second);

        is_removed_first && is_removed_second
    }

    pub fn contains_key_first (&self, key: &T) -> bool {
        self.map_first.contains_key (key)
    }

    pub fn contains_key_second (&self, key: &U) -> bool {
        self.map_second.contains_key (key)
    }
}

impl<T, U> Default for CrossJoinMap<T, U>
where T: Clone + Eq + Hash, U: Clone + Eq + Hash {
    fn default () -> Self {
        Self::new ()
    }
}

#[cfg (test)]
mod tests {
    use super::*;

    #[test]
    fn cross_join_map_insert () {
        let mut map: CrossJoinMap<u8, (u8, u8)> = CrossJoinMap::new ();

        // Test empty insert
        assert! (map.insert ((0, (0, 0))));
        // Test non-colliding insert
        assert! (map.insert ((1, (1, 1))));
        // Test colliding insert
        assert! (map.insert ((1, (0, 0))));
        assert! (map.insert ((1, (2, 2))));
    }

    #[test]
    fn cross_join_map_get () {
        let mut map: CrossJoinMap<u8, (u8, u8)> = CrossJoinMap::new ();

        // Test empty get
        assert! (map.get_first (&0).is_none ());
        assert! (map.get_second (&(0, 0)).is_none ());
        // Test non-empty get
        map.insert ((0, (0, 0)));
        assert_eq! (map.get_first (&0).unwrap ().len (), 1);
        assert_eq! (map.get_second (&(0, 0)).unwrap ().len (), 1);
        // Test non-colliding get
        map.insert ((1, (1, 1)));
        assert_eq! (map.get_first (&1).unwrap ().len (), 1);
        assert_eq! (map.get_second (&(1, 1)).unwrap ().len (), 1);
        assert_eq! (map.get_first (&0).unwrap ().len (), 1);
        assert_eq! (map.get_second (&(0, 0)).unwrap ().len (), 1);
        // Test colliding get
        map.insert ((1, (2, 2)));
        assert_eq! (map.get_first (&1).unwrap ().len (), 2);
        assert_eq! (map.get_second (&(1, 1)).unwrap ().len (), 1);
        assert_eq! (map.get_second (&(2, 2)).unwrap ().len (), 1);
        map.insert ((2, (2, 2)));
        assert_eq! (map.get_first (&2).unwrap ().len (), 1);
        assert_eq! (map.get_second (&(2, 2)).unwrap ().len (), 2);
    }

    #[test]
    fn cross_join_map_remove () {
        let mut map: CrossJoinMap<u8, (u8, u8)> = CrossJoinMap::new ();

        // Test empty remove
        assert! (!map.remove (&0, &(0, 0)));
        assert! (!map.remove_first (&0));
        assert! (!map.remove_second (&(0, 0)));
        // Test non-empty remove
        map.insert ((0, (0, 0)));
        assert! (map.remove (&0, &(0, 0)));
        assert! (map.get_first (&0).unwrap ().is_empty ());
        assert! (map.get_second (&(0, 0)).unwrap ().is_empty ());
        map.insert ((0, (0, 0)));
        assert! (map.remove_first (&0));
        assert! (map.get_first (&0).is_none ());
        assert! (map.get_second (&(0, 0)).unwrap ().is_empty ());
        map.insert ((0, (0, 0)));
        assert! (map.remove_second (&(0, 0)));
        assert! (map.get_first (&0).unwrap ().is_empty ());
        assert! (map.get_second (&(0, 0)).is_none ());
        // Test colliding remove
        map.insert ((0, (0, 0)));
        map.insert ((1, (0, 0)));
        assert! (map.remove (&0, &(0, 0)));
        assert! (map.get_first (&0).unwrap ().is_empty ());
        assert_eq! (map.get_first (&1).unwrap ().len (), 1);
        assert_eq! (map.get_second (&(0, 0)).unwrap ().len (), 1);
        map.insert ((0, (0, 0)));
        map.insert ((1, (0, 0)));
        assert! (map.remove_first (&0));
        assert! (map.get_first (&0).is_none ());
        assert_eq! (map.get_first (&1).unwrap ().len (), 1);
        assert_eq! (map.get_second (&(0, 0)).unwrap ().len (), 1);
        map.insert ((0, (0, 0)));
        map.insert ((0, (1, 1)));
        assert! (map.remove_second (&(0, 0)));
        assert_eq! (map.get_first (&0).unwrap ().len (), 1);
        assert! (map.get_second (&(0, 0)).is_none ());
        assert_eq! (map.get_second (&(1, 1)).unwrap ().len (), 1);
    }

    #[test]
    fn cross_join_map_contains_key () {
        let mut map: CrossJoinMap<u8, (u8, u8)> = CrossJoinMap::new ();

        // Test empty contains
        assert! (!map.contains_key_first (&0));
        assert! (!map.contains_key_second (&(0, 0)));
        // Test non-empty contains
        map.insert ((0, (0, 0)));
        assert! (map.contains_key_first (&0));
        assert! (map.contains_key_second (&(0, 0)));
        assert! (!map.contains_key_first (&1));
        assert! (!map.contains_key_second (&(1, 1)));
    }
}
