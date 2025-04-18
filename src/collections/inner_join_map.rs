use std::collections::HashMap;
use std::hash::Hash;

/*
 * Map that behaves like an inner join:
 * Each T maps to one U
 * Each U maps to one T
 * Mappings are duplicated
 */
#[derive (Debug)]
pub struct InnerJoinMap<T, U> {
    map_first: HashMap<T, U>,
    map_second: HashMap<U, T>,
}

impl<T, U> InnerJoinMap<T, U>
where T: Clone + Eq + Hash, U: Clone + Eq + Hash {
    #[allow (clippy::new_without_default)]
    pub fn new () -> Self {
        let map_first: HashMap<T, U> = HashMap::new ();
        let map_second: HashMap<U, T> = HashMap::new ();

        Self { map_first, map_second }
    }

    pub fn insert (&mut self, values: (T, U)) -> Option<(Option<U>, Option<T>)> {
        if self.map_first.contains_key (&values.0) || self.map_second.contains_key (&values.1) {
            None
        } else {
            let original_first: Option<U> = self.map_first.insert (values.0.clone (), values.1.clone ());
            let original_second: Option<T> = self.map_second.insert (values.1, values.0);
    
            Some ((original_first, original_second))
        }
    }

    pub fn get_first (&self, key: &T) -> Option<&U> {
        self.map_first.get (key)
    }

    pub fn get_second (&self, key: &U) -> Option<&T> {
        self.map_second.get (key)
    }

    pub fn remove_first (&mut self, key_first: &T) -> Option<U> {
        let key_second: &U = self.map_first.get (key_first)?;
        let original_second: Option<T> = self.map_second.remove (key_second);
        let original_first: Option<U> = self.map_first.remove (key_first);

        assert_eq! (original_first.is_some (), original_second.is_some ());

        original_first
    }

    pub fn remove_second (&mut self, key_second: &U) -> Option<T> {
        let key_first: &T = self.map_second.get (key_second)?;
        let original_first: Option<U> = self.map_first.remove (key_first);
        let original_second: Option<T> = self.map_second.remove (key_second);

        assert_eq! (original_first.is_some (), original_second.is_some ());

        original_second
    }

    pub fn replace_first (&mut self, value: T, destination: U) -> Option<(U, Option<T>)> {
        let original_first: U = self.map_first.remove (&value)?;
        let original_second: Option<T> = match self.map_second.get (&destination) {
            Some (k) => {
                self.map_first.remove (k);

                self.map_second.remove (&destination)
            }
            None => None,
        };

        self.map_second.remove (&original_first);
        self.insert ((value, destination));

        Some ((original_first, original_second))
    }

    pub fn replace_second (&mut self, value: U, destination: T) -> Option<(T, Option<U>)> {
        let original_second: T = self.map_second.remove (&value)?;
        let original_first: Option<U> = match self.map_first.get (&destination) {
            Some (k) => {
                self.map_second.remove (k);

                self.map_first.remove (&destination)
            }
            None => None,
        };

        self.map_first.remove (&original_second);
        self.insert ((destination, value));

        Some ((original_second, original_first))
    }

    pub fn contains_key_first (&self, key: &T) -> bool {
        self.map_first.contains_key (key)
    }

    pub fn contains_key_second (&self, key: &U) -> bool {
        self.map_second.contains_key (key)
    }
}

// impl<T, U> Default for InnerJoinMap<T, U>
// where T: Clone + Eq + Hash, U: Clone + Eq + Hash {
//     fn default () -> Self {
//         Self::new ()
//     }
// }

#[cfg (test)]
mod tests {
    use super::*;

    #[test]
    fn inner_join_map_insert () {
        let mut map: InnerJoinMap<u8, (u8, u8)> = InnerJoinMap::new ();

        // Test empty insert
        assert_eq! (map.insert ((0, (0, 0))).unwrap (), (None, None));
        // Test non-colliding insert
        assert_eq! (map.insert ((1, (1, 1))).unwrap (), (None, None));
        // Test colliding insert
        assert! (map.insert ((0, (1, 1))).is_none ());
        assert! (map.insert ((1, (0, 0))).is_none ());
    }

    #[test]
    fn inner_join_map_get () {
        let mut map: InnerJoinMap<u8, (u8, u8)> = InnerJoinMap::new ();

        // Test empty get
        assert! (map.get_first (&0).is_none ());
        assert! (map.get_second (&(0, 0)).is_none ());
        // Test non-empty get
        map.insert ((0, (0, 0)));
        assert_eq! (map.get_first (&0).unwrap (), &(0, 0));
        assert_eq! (map.get_second (&(0, 0)).unwrap (), &0);
        // Test non-colliding get
        map.insert ((1, (1, 1)));
        assert_eq! (map.get_first (&1).unwrap (), &(1, 1));
        assert_eq! (map.get_second (&(1, 1)).unwrap (), &1);
        assert_eq! (map.get_first (&0).unwrap (), &(0, 0));
        assert_eq! (map.get_second (&(0, 0)).unwrap (), &0);
    }

    #[test]
    fn inner_join_map_remove () {
        let mut map: InnerJoinMap<u8, (u8, u8)> = InnerJoinMap::new ();

        // Test empty remove
        assert! (map.remove_first (&0).is_none ());
        assert! (map.remove_second (&(0, 0)).is_none ());
        // Test non-empty remove
        map.insert ((0, (0, 0)));
        assert_eq! (map.remove_first (&0).unwrap (), (0, 0));
        assert! (map.get_first (&0).is_none ());
        assert! (map.get_second (&(0, 0)).is_none ());
        map.insert ((1, (1, 1)));
        assert_eq! (map.remove_second (&(1, 1)).unwrap (), 1);
        assert! (map.get_first (&1).is_none ());
        assert! (map.get_second (&(1, 1)).is_none ());
    }

    #[test]
    fn inner_join_map_replace () {
        let mut map: InnerJoinMap<u8, (u8, u8)> = InnerJoinMap::new ();

        // Test empty replace
        assert! (map.replace_first (0, (0, 0)).is_none ());
        assert! (map.replace_second ((0, 0), 0).is_none ());
        // Test partial replace
        map.insert ((1, (0, 0)));
        assert_eq! (map.replace_first (1, (1, 1)).unwrap (), ((0, 0), None));
        assert_eq! (map.get_first (&1).unwrap (), &(1, 1));
        assert_eq! (map.get_second (&(1, 1)).unwrap (), &1);
        assert! (map.get_second (&(0, 0)).is_none ());
        map.insert ((0, (2, 2)));
        assert_eq! (map.replace_second ((2, 2), 2).unwrap (), (0, None));
        assert_eq! (map.get_first (&2).unwrap (), &(2, 2));
        assert_eq! (map.get_second (&(2, 2)).unwrap (), &2);
        assert! (map.get_first (&0).is_none ());
        // Test complete replace
        assert_eq! (map.replace_first (1, (2, 2)).unwrap (), ((1, 1), Some (2)));
        assert_eq! (map.get_first (&1).unwrap (), &(2, 2));
        assert_eq! (map.get_second (&(2, 2)).unwrap (), &1);
        assert! (map.get_first (&2).is_none ());
        assert! (map.get_second (&(1, 1)).is_none ());
        map.insert ((3, (3, 3)));
        assert_eq! (map.replace_second ((2, 2), 3).unwrap (), (1, Some ((3, 3))));
        assert_eq! (map.get_first (&3).unwrap (), &(2, 2));
        assert_eq! (map.get_second (&(2, 2)).unwrap (), &3);
        assert! (map.get_first (&1).is_none ());
        assert! (map.get_second (&(3, 3)).is_none ());
    }

    #[test]
    fn inner_join_map_contains_key () {
        let mut map: InnerJoinMap<u8, (u8, u8)> = InnerJoinMap::new ();

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
