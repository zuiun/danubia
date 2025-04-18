use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::fmt::Debug;

/*
* Map that behaves like a (left) outer join:
* Each T maps to many U
* Each U maps to one T
 * Mappings are duplicated
 */
#[derive (Debug)]
pub struct OuterJoinMap<T, U> {
    map_first: HashMap<T, HashSet<U>>,
    map_second: HashMap<U, T>
}

impl<T, U> OuterJoinMap<T, U>
where T: Debug + Clone + Eq + Hash, U: Debug + Clone + Eq + Hash {
    #[allow (clippy::new_without_default)]
    pub fn new () -> Self {
        let map_first: HashMap<T, HashSet<U>> = HashMap::new ();
        let map_second: HashMap<U, T> = HashMap::new ();

        Self { map_first, map_second }
    }

    pub fn insert (&mut self, values: (T, U)) -> bool {
        if let Entry::Vacant (e) = self.map_second.entry (values.1.clone ()) {
            self.map_first.entry (values.0.clone ())
                    .or_default ()
                    .insert (values.1);
            e.insert (values.0);

            true
        } else {
            false
        }
    }

    pub fn get_first (&self, key: &T) -> Option<&HashSet<U>> {
        self.map_first.get (key)
    }

    pub fn get_second (&self, key: &U) -> Option<&T> {
        self.map_second.get (key)
    }

    pub fn get_collection_second (&self, key_second: &U) -> Option<&HashSet<U>> {
        let key_first: &T = self.map_second.get (key_second)?;

        self.map_first.get (key_first)
    }

    pub fn remove (&mut self, key_second: &U) -> bool {
        let key_first: &T = match self.map_second.get (key_second) {
            Some (k) => k,
            None => return false,
        };
        let collection_first: &mut HashSet<U> = self.map_first.get_mut (key_first)
                .unwrap_or_else (|| panic! ("Collection not found for key {:?}", key_first));
        let is_removed_first: bool = collection_first.remove (key_second);
        let is_removed_second: bool = self.map_second.remove (key_second).is_some ();

        assert_eq! (is_removed_first, is_removed_second);

        is_removed_first && is_removed_second
    }

    pub fn replace (&mut self, value: U, destination: T) -> Option<T> {
        let key_second: T = self.map_second.get (&value)?.clone ();
        let collection_first: &mut HashSet<U> = self.map_first.get_mut (&key_second)?;

        if collection_first.remove (&value) {
            if self.map_first.contains_key (&destination) {
                let collection_first: &mut HashSet<U> = self.map_first.get_mut (&destination)
                        .unwrap_or_else (|| panic! ("Collection not found for key {:?}", value));

                collection_first.insert (value.clone ());
            } else {
                let mut collection_first: HashSet<U> = HashSet::new ();

                collection_first.insert (value.clone ());
                self.map_first.insert (destination.clone (), collection_first);
            }

            self.map_second.insert (value, destination);

            Some (key_second)
        } else {
            None
        }        
    }

    pub fn contains_key_first (&self, key: &T) -> bool {
        self.map_first.contains_key (key)
    }

    pub fn contains_key_second (&self, key: &U) -> bool {
        self.map_second.contains_key (key)
    }
}

// impl<T, U> Default for OuterJoinMap<T, U>
// where T: Debug + Clone + Eq + Hash, U: Debug + Clone + Eq + Hash {
//     fn default () -> Self {
//         Self::new ()
//     }
// }

#[cfg (test)]
mod tests {
    use super::*;

    #[test]
    fn outer_join_map_insert () {
        let mut map: OuterJoinMap<u8, (u8, u8)> = OuterJoinMap::new ();

        // Test empty insert
        assert! (map.insert ((0, (0, 0))));
        // Test non-colliding insert
        assert! (map.insert ((1, (1, 1))));
        // Test colliding insert
        assert! (!map.insert ((1, (0, 0))));
        assert! (map.insert ((1, (2, 2))));
    }

    #[test]
    fn outer_join_map_get () {
        let mut map: OuterJoinMap<u8, (u8, u8)> = OuterJoinMap::new ();

        // Test empty get
        assert! (map.get_first (&0).is_none ());
        assert! (map.get_second (&(0, 0)).is_none ());
        // Test non-empty get
        map.insert ((0, (0, 0)));
        assert_eq! (map.get_first (&0).unwrap ().len (), 1);
        assert_eq! (map.get_second (&(0, 0)).unwrap (), &0);
        assert_eq! (map.get_collection_second (&(0, 0)).unwrap ().len (), 1);
        // Test non-colliding get
        map.insert ((1, (1, 1)));
        assert_eq! (map.get_first (&1).unwrap ().len (), 1);
        assert_eq! (map.get_second (&(1, 1)).unwrap (), &1);
        assert_eq! (map.get_collection_second (&(1, 1)).unwrap ().len (), 1);
        assert_eq! (map.get_first (&0).unwrap ().len (), 1);
        assert_eq! (map.get_second (&(0, 0)).unwrap (), &0);
        assert_eq! (map.get_collection_second (&(0, 0)).unwrap ().len (), 1);
        // Test colliding get
        map.insert ((1, (2, 2)));
        assert_eq! (map.get_first (&1).unwrap ().len (), 2);
        assert_eq! (map.get_second (&(1, 1)).unwrap (), &1);
        assert_eq! (map.get_collection_second (&(1, 1)).unwrap ().len (), 2);
        assert_eq! (map.get_second (&(2, 2)).unwrap (), &1);
        assert_eq! (map.get_collection_second (&(2, 2)).unwrap ().len (), 2);
    }

    #[test]
    fn outer_join_map_remove () {
        let mut map: OuterJoinMap<u8, (u8, u8)> = OuterJoinMap::new ();

        // Test empty remove
        assert! (!map.remove (&(0, 0)));
        // Test non-empty remove
        map.insert ((0, (0, 0)));
        assert! (map.remove (&(0, 0)));
        assert! (map.get_first (&0).unwrap ().is_empty ());
        assert! (map.get_second (&(0, 0)).is_none ());
        assert! (map.get_collection_second (&(0, 0)).is_none ());
    }

    #[test]
    fn outer_join_map_replace () {
        let mut map: OuterJoinMap<u8, (u8, u8)> = OuterJoinMap::new ();

        // Test empty replace
        assert! (map.replace ((0, 0), 0).is_none ());
        // Test partial replace
        map.remove (&(0, 0));
        map.insert ((0, (1, 1)));
        assert_eq! (map.replace ((1, 1), 1).unwrap (), 0);
        assert_eq! (map.get_first (&1).unwrap ().len (), 1);
        assert_eq! (map.get_second (&(1, 1)).unwrap (), &1);
        assert_eq! (map.get_collection_second (&(1, 1)).unwrap ().len (), 1);
        assert! (map.get_first (&0).unwrap ().is_empty ());
        assert! (map.get_second (&(0, 0)).is_none ());
        // Test complete replace
        map.insert ((1, (0, 0)));
        assert_eq! (map.replace ((0, 0), 0).unwrap (), 1);
        assert_eq! (map.get_first (&0).unwrap ().len (), 1);
        assert_eq! (map.get_second (&(0, 0)).unwrap (), &0);
        assert_eq! (map.get_collection_second (&(0, 0)).unwrap ().len (), 1);
        assert_eq! (map.get_first (&1).unwrap ().len (), 1);
        assert_eq! (map.get_second (&(1, 1)).unwrap (), &1);
    }

    #[test]
    fn outer_join_map_contains_key () {
        let mut map: OuterJoinMap<u8, (u8, u8)> = OuterJoinMap::new ();

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
