use std::{collections::{HashMap, HashSet}, hash::Hash};

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
where T: Clone + std::fmt::Debug + Eq + Hash, U: Clone + std::fmt::Debug + Eq + Hash {
    pub fn new () -> Self {
        let map_first: HashMap<T, HashSet<U>> = HashMap::new ();
        let map_second: HashMap<U, HashSet<T>> = HashMap::new ();

        Self { map_first, map_second }
    }

    pub fn insert (&mut self, values: (T, U)) -> bool {
        let is_inserted_first: bool = if self.map_first.contains_key (&values.0) {
            let collection_first: &mut HashSet<U> = match self.map_first.get_mut (&values.0) {
                Some (c) => c,
                None => return false,
            };

            collection_first.insert (values.1.clone ())
        } else {
            let mut collection_first: HashSet<U> = HashSet::new ();

            collection_first.insert (values.1.clone ());
            self.map_first.insert (values.0.clone (), collection_first).is_none ()
        };
        let is_inserted_second: bool = if self.map_second.contains_key (&values.1) {
            let collection_second: &mut HashSet<T> = match self.map_second.get_mut (&values.1) {
                Some (c) => c,
                None => return false,
            };

            collection_second.insert (values.0)
        } else {
            let mut collection_second: HashSet<T> = HashSet::new ();

            collection_second.insert (values.0);
            self.map_second.insert (values.1, collection_second).is_none ()
        };

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

#[cfg (test)]
mod tests {
    use super::*;

    #[test]
    fn duplicate_cross_map_insert () {
        let mut map: CrossJoinMap<u8, (u8, u8)> = CrossJoinMap::new ();

        // Test empty insert
        assert_eq! (map.insert ((0, (0, 0))), true);
        // Test non-colliding insert
        assert_eq! (map.insert ((1, (1, 1))), true);
        // Test colliding insert
        assert_eq! (map.insert ((1, (0, 0))), true);
        assert_eq! (map.insert ((1, (2, 2))), true);
    }

    #[test]
    fn duplicate_cross_map_get () {
        let mut map: CrossJoinMap<u8, (u8, u8)> = CrossJoinMap::new ();

        // Test empty get
        assert_eq! (map.get_first (&0), None);
        assert_eq! (map.get_second (&(0, 0)), None);
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
    fn duplicate_cross_map_remove () {
        let mut map: CrossJoinMap<u8, (u8, u8)> = CrossJoinMap::new ();

        // Test empty remove
        assert_eq! (map.remove (&0, &(0, 0)), false);
        assert_eq! (map.remove_first (&0), false);
        assert_eq! (map.remove_second (&(0, 0)), false);
        // Test non-empty remove
        map.insert ((0, (0, 0)));
        assert_eq! (map.remove (&0, &(0, 0)), true);
        assert_eq! (map.get_first (&0).unwrap ().len (), 0);
        assert_eq! (map.get_second (&(0, 0)).unwrap ().len (), 0);
        map.insert ((0, (0, 0)));
        assert_eq! (map.remove_first (&0), true);
        assert_eq! (map.get_first (&0), None);
        assert_eq! (map.get_second (&(0, 0)).unwrap ().len (), 0);
        map.insert ((0, (0, 0)));
        assert_eq! (map.remove_second (&(0, 0)), true);
        assert_eq! (map.get_first (&0).unwrap ().len (), 0);
        assert_eq! (map.get_second (&(0, 0)), None);
        // Test colliding remove
        map.insert ((0, (0, 0)));
        map.insert ((1, (0, 0)));
        assert_eq! (map.remove (&0, &(0, 0)), true);
        assert_eq! (map.get_first (&0).unwrap ().len (), 0);
        assert_eq! (map.get_first (&1).unwrap ().len (), 1);
        assert_eq! (map.get_second (&(0, 0)).unwrap ().len (), 1);
        map.insert ((0, (0, 0)));
        map.insert ((1, (0, 0)));
        assert_eq! (map.remove_first (&0), true);
        assert_eq! (map.get_first (&0), None);
        assert_eq! (map.get_first (&1).unwrap ().len (), 1);
        assert_eq! (map.get_second (&(0, 0)).unwrap ().len (), 1);
        map.insert ((0, (0, 0)));
        map.insert ((0, (1, 1)));
        assert_eq! (map.remove_second (&(0, 0)), true);
        assert_eq! (map.get_first (&0).unwrap ().len (), 1);
        assert_eq! (map.get_second (&(0, 0)), None);
        assert_eq! (map.get_second (&(1, 1)).unwrap ().len (), 1);
    }

    #[test]
    fn duplicate_cross_map_contains_key () {
        let mut map: CrossJoinMap<u8, (u8, u8)> = CrossJoinMap::new ();

        // Test empty contains
        assert_eq! (map.contains_key_first (&0), false);
        assert_eq! (map.contains_key_second (&(0, 0)), false);
        // Test non-empty contains
        map.insert ((0, (0, 0)));
        assert_eq! (map.contains_key_first (&0), true);
        assert_eq! (map.contains_key_second (&(0, 0)), true);
        assert_eq! (map.contains_key_first (&1), false);
        assert_eq! (map.contains_key_second (&(1, 1)), false);
    }
}
