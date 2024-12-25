use core::fmt::Debug;
use std::{collections::{HashMap, HashSet}, fmt, hash::Hash, sync::atomic::{AtomicUsize, Ordering}};

pub type ID = usize; // Due to event values, ID is assumed to be an u8
pub type Adjustment = (Statistic, u16, bool); // statistic, change (value depends on context), is add
pub type Adjustments = [Option<Adjustment>; 4]; // Any more than 4 is probably excessive

pub const TYPE_UNIT: ID = 0;
pub const TYPE_TERRAIN: ID = 1;
pub const TYPE_CITY: ID = 2;
pub const TYPE_WEAPON: ID = 3;
pub const TYPE_MAGIC: ID = 4;
pub static IDS: [AtomicUsize; 5] = [
    AtomicUsize::new (0),
    AtomicUsize::new (0),
    AtomicUsize::new (0),
    AtomicUsize::new (0),
    AtomicUsize::new (0),
];

pub trait Unique {
    fn assign_id () -> ID;
    fn get_id (&self) -> ID;
    fn get_type (&self) -> ID;
}

pub trait Timed {
    fn get_duration (&self) -> u16;
    fn dec_duration (&mut self) -> bool;
}

pub trait Modifiable {
    fn add_modifier (&mut self, modifier: Modifier) -> bool;
    fn dec_durations (&mut self) -> ();
}

#[derive (Debug)]
#[derive (Clone, Copy)]
pub enum UnitStatistic {
    MRL, // morale - willingness to fight (percentage)
    HLT, // manpower - number of soldiers
    SPL, // supply - proportion of soldiers equipped (percentage)
    ATK, // attack – physical damage
    DEF, // defence – physical resistance
    MAG, // magic – magical damage and resistance
    MOV, // manoeuvre – speed and movement
    ORG, // cohesion – modifier for formation effects and subordinate units (percentage)
    Length
}

#[derive (Debug)]
#[derive (Clone, Copy)]
pub enum WeaponStatistic {
    DMG, // damage - base damage
    SLH, // slash – modifier for physical damage, strong against manpower
    PRC, // pierce – modifier for physical damage, strong against morale
    DCY, // decay – modifier for magical damage
    Length
}

#[derive (Debug)]
#[derive (Clone, Copy)]
pub enum Statistic {
    Unit (UnitStatistic),
    Weapon (WeaponStatistic),
    Tile
}

#[derive (Debug)]
#[derive (Clone, Copy)]
pub enum Area {
    Single,
    Radial (u8), // radius
    Path (u8) // width
}

#[derive (Debug)]
#[derive (Clone, Copy)]
pub enum Target {
    Ally (bool), // false = ally, true = self
    Enemy,
    All (bool), // false = enemies, true = allies
    Map
}

#[derive (Debug)]
#[derive (Clone, Copy)]
pub enum Capacity {
    Constant (u16, u16, u16), // current, maximum, base
    Quantity (u16, u16) // current, maximum
}

#[derive (Debug)]
#[derive (Clone, Copy)]
pub enum Direction {
    Up,
    Right,
    Down,
    Left,
    Length
}

#[derive (Debug)]
pub enum Condition {
    OnHit,
    OnAttack
}

#[derive (Debug)]
pub struct Information {
    name: String,
    descriptions: Vec<String>,
    current_description: usize
}

#[derive (Debug)]
#[derive (Clone, Copy)]
pub struct Modifier {
    id: ID,
    adjustments: Adjustments,
    duration: Capacity,
    can_stack: bool // for tiles: false = flat change, true = set to constant
}

#[derive (Debug)]
pub struct Status {
    modifier_id: ID,
    // trigger: Condition, // TODO: triggered statuses: on hit -> reflect damage, on attack -> apply modifier, against specific units -> apply modifier, what else?
    duration: Capacity,
    target: Target,
    next: Option<Box<Status>>
}

#[derive (Debug)]
pub struct DuplicateMap<T, U> {
    map_first: HashMap<T, U>,
    map_second: HashMap<U, T>
}

#[derive (Debug)]
pub struct DuplicateCollectionMap<T, U> {
    map_first: HashMap<T, HashSet<U>>,
    map_second: HashMap<U, T>
}

impl Information {
    pub fn new (name: String, descriptions: Vec<String>, current_description: usize) -> Self {
        Self { name, descriptions, current_description }
    }

    pub fn new_test () -> Self {
        static ID: AtomicUsize = AtomicUsize::new (0);
        let name: String = format! ("{}", ID.fetch_add (1, Ordering::SeqCst));
        let descriptions: Vec<String> = Vec::new ();
        let current_description: usize = 0;

        Self { name, descriptions, current_description }
    }

    pub fn get_name (&self) -> &str {
        &self.name
    }

    pub fn get_description (&self) -> &str {
        &self.descriptions[self.current_description]
    }
}

impl Modifier {
    pub const fn new (id: ID, adjustments: Adjustments, duration: Capacity, can_stack: bool) -> Self {
        Self { id, adjustments, duration, can_stack }
    }

    pub fn get_adjustments (&self) -> Adjustments {
        self.adjustments
    }

    pub fn can_stack (&self) -> bool {
        self.can_stack
    }
}

impl Status {
    pub const fn new (modifier_id: ID, duration: Capacity, target: Target, next: Option<Box<Status>>) -> Self {
        Self { modifier_id, duration, target, next }
    }

    pub fn get_modifier_id (&self) -> ID {
        self.modifier_id
    }
}

impl<T, U> DuplicateMap<T, U>
where T: Clone + std::fmt::Debug + Eq + Hash, U: Clone + std::fmt::Debug + Eq + Hash {
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
                self.map_first.remove (&k);

                self.map_second.remove (&destination)
            }
            None => None
        };

        self.map_second.remove (&original_first);
        self.insert ((value, destination));

        Some ((original_first, original_second))
    }

    pub fn replace_second (&mut self, value: U, destination: T) -> Option<(T, Option<U>)> {
        let original_second: T = self.map_second.remove (&value)?;
        let original_first: Option<U> = match self.map_first.get (&destination) {
            Some (k) => {
                self.map_second.remove (&k);

                self.map_first.remove (&destination)
            }
            None => None
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

impl<T, U> DuplicateCollectionMap<T, U>
where T: Clone + std::fmt::Debug + Eq + Hash, U: Clone + std::fmt::Debug + Eq + Hash {
    pub fn new (collection: impl IntoIterator<Item = T>) -> Self {
        let mut map_first: HashMap<T, HashSet<U>> = HashMap::new ();
        let map_second: HashMap<U, T> = HashMap::new ();

        for key in collection {
            map_first.insert (key, HashSet::new ());
        }

        Self { map_first, map_second }
    }

    pub fn insert (&mut self, values: (T, U)) -> bool {
        assert! (self.map_first.contains_key (&values.0));

        if self.map_second.contains_key (&values.1) {
            false
        } else {
            let collection_first: &mut HashSet<U> = self.map_first.get_mut (&values.0)
                    .expect (&format! ("Collection not found for key {:?}", values.0));
            let collection_first: bool = collection_first.insert (values.1.clone ());
            let original_second: Option<T> = self.map_second.insert (values.1, values.0);
    
            assert_eq! (collection_first, original_second.is_none ());
    
            collection_first
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

    pub fn remove (&mut self, key: &U) -> bool {
        let key_first: &T = match self.map_second.get (key) {
            Some (k) => k,
            None => return false
        };
        let collection_first: &mut HashSet<U> = self.map_first.get_mut (key_first)
                .expect (&format! ("Collection not found for key {:?}", key_first));
        let collection_first: bool = collection_first.remove (key);
        let original_second: Option<T> = self.map_second.remove (key);

        assert_eq! (collection_first, original_second.is_some ());

        collection_first
    }

    pub fn replace (&mut self, value: U, destination: T) -> Option<T> {
        assert! (self.map_first.contains_key (&destination));

        let key_old: T = self.map_second.get (&value)?.clone ();
        let collection_old: &mut HashSet<U> = self.map_first.get_mut (&key_old)?;

        if collection_old.remove (&value) {
            let collection_new: &mut HashSet<U> = self.map_first.get_mut (&destination)
                    .expect (&format! ("Collection not found for key {:?}", value));

            collection_new.insert (value.clone ());
            self.map_second.insert (value, destination);

            Some (key_old)
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

// pub fn move_cursor (&mut self, direction: Direction) -> Option<Cursor> {
//     match direction {
//         Direction::Up => if self.cursor.0 > 0 {
//             self.cursor.0 -= 1;

//             Some (self.cursor)
//         } else {
//             None
//         }
//         Direction::Right => if self.cursor.1 < self.map[0].len () - 1 {
//             self.cursor.1 += 1;

//             Some (self.cursor)
//         } else {
//             None
//         }
//         Direction::Down => if self.cursor.0 < self.map.len () - 1 {
//             self.cursor.0 += 1;

//             Some (self.cursor)
//         } else {
//             None
//         }
//         Direction::Left => if self.cursor.1 > 0 {
//             self.cursor.1 -= 1;

//             Some (self.cursor)
//         } else {
//             None
//         }
//     }
// }

impl Timed for Modifier {
    fn get_duration (&self) -> u16 {
        match self.duration {
            Capacity::Constant (d, _, _) => d,
            Capacity::Quantity (d, _) => d
        }
    }

    fn dec_duration (&mut self) -> bool {
        match self.duration {
            Capacity::Constant (_, _, _) => false,
            Capacity::Quantity (d, m) => {
                let duration: u16 = d.checked_sub (1).unwrap_or (0);

                self.duration = Capacity::Quantity (duration, m);

                duration == 0
            }
        }
    }
}

impl Timed for Status {
    fn get_duration (&self) -> u16 {
        match self.duration {
            Capacity::Constant (d, _, _) => d,
            Capacity::Quantity (d, _) => d
        }
    }

    fn dec_duration (&mut self) -> bool {
        match self.duration {
            Capacity::Constant (_, _, _) => false,
            Capacity::Quantity (d, m) => {
                let duration: u16 = d.checked_sub (1).unwrap_or (0);

                self.duration = Capacity::Quantity (duration, m);

                duration == 0
            }
        }
    }
}

impl PartialEq for Modifier {
    fn eq (&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl fmt::Display for Information {
    fn fmt (&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write! (f, "{}\n{}", self.name, self.descriptions[self.current_description])
    }
}

#[cfg (test)]
mod tests {
    use super::*;
    use crate::engine::map::Location;

    #[test]
    fn duplicate_map_insert () {
        let mut map: DuplicateMap<ID, Location> = DuplicateMap::new ();

        // Test empty insert
        assert_eq! (map.insert ((0, (0, 0))).unwrap (), (None, None));
        // Test non-colliding insert
        assert_eq! (map.insert ((1, (1, 1))).unwrap (), (None, None));
        // Test colliding insert
        assert_eq! (map.insert ((0, (1, 1))), None);
        assert_eq! (map.insert ((1, (0, 0))), None);
    }

    #[test]
    fn duplicate_map_get () {
        let mut map: DuplicateMap<ID, Location> = DuplicateMap::new ();

        // Test empty get
        assert_eq! (map.get_first (&0), None);
        assert_eq! (map.get_second (&(0, 0)), None);
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
    fn duplicate_map_remove () {
        let mut map: DuplicateMap<ID, Location> = DuplicateMap::new ();

        // Test empty remove
        assert_eq! (map.remove_first (&0), None);
        assert_eq! (map.remove_second (&(0, 0)), None);
        // Test non-empty remove
        map.insert ((0, (0, 0)));
        assert_eq! (map.remove_first (&0).unwrap (), (0, 0));
        assert_eq! (map.get_first (&0), None);
        assert_eq! (map.get_second (&(0, 0)), None);
        map.insert ((1, (1, 1)));
        assert_eq! (map.remove_second (&(1, 1)).unwrap (), 1);
        assert_eq! (map.get_first (&1), None);
        assert_eq! (map.get_second (&(1, 1)), None);
    }

    #[test]
    fn duplicate_map_replace () {
        let mut map: DuplicateMap<ID, Location> = DuplicateMap::new ();

        // Test empty replace
        assert_eq! (map.replace_first (0, (0, 0)), None);
        assert_eq! (map.replace_second ((0, 0), 0), None);
        // Test partial replace
        map.insert ((1, (0, 0)));
        assert_eq! (map.replace_first (1, (1, 1)).unwrap (), ((0, 0), None));
        assert_eq! (map.get_first (&1).unwrap (), &(1, 1));
        assert_eq! (map.get_second (&(1, 1)).unwrap (), &1);
        assert_eq! (map.get_second (&(0, 0)), None);
        map.insert ((0, (2, 2)));
        assert_eq! (map.replace_second ((2, 2), 2).unwrap (), (0, None));
        assert_eq! (map.get_first (&2).unwrap (), &(2, 2));
        assert_eq! (map.get_second (&(2, 2)).unwrap (), &2);
        assert_eq! (map.get_first (&0), None);
        // Test complete replace
        assert_eq! (map.replace_first (1, (2, 2)).unwrap (), ((1, 1), Some (2)));
        assert_eq! (map.get_first (&1).unwrap (), &(2, 2));
        assert_eq! (map.get_second (&(2, 2)).unwrap (), &1);
        assert_eq! (map.get_first (&2), None);
        assert_eq! (map.get_second (&(1, 1)), None);
        map.insert ((3, (3, 3)));
        assert_eq! (map.replace_second ((2, 2), 3).unwrap (), (1, Some ((3, 3))));
        assert_eq! (map.get_first (&3).unwrap (), &(2, 2));
        assert_eq! (map.get_second (&(2, 2)).unwrap (), &3);
        assert_eq! (map.get_first (&1), None);
        assert_eq! (map.get_second (&(3, 3)), None);
    }

    #[test]
    fn duplicate_map_contains_key () {
        let mut map: DuplicateMap<ID, Location> = DuplicateMap::new ();

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

    #[test]
    fn duplicate_collection_map_insert () {
        let ids: Vec<ID> = vec![0, 1];
        let mut map: DuplicateCollectionMap<ID, Location> = DuplicateCollectionMap::new (ids);

        // Test empty insert
        assert_eq! (map.insert ((0, (0, 0))), true);
        // Test non-colliding insert
        assert_eq! (map.insert ((1, (1, 1))), true);
        // Test colliding insert
        assert_eq! (map.insert ((1, (0, 0))), false);
        assert_eq! (map.insert ((1, (2, 2))), true);
    }

    #[test]
    fn duplicate_collection_map_get () {
        let ids: Vec<ID> = vec![0, 1];
        let mut map: DuplicateCollectionMap<ID, Location> = DuplicateCollectionMap::new (ids);

        // Test empty get
        assert_eq! (map.get_first (&0).unwrap ().len (), 0);
        assert_eq! (map.get_second (&(0, 0)), None);
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
    fn duplicate_collection_map_remove () {
        let ids: Vec<ID> = vec![0];
        let mut map: DuplicateCollectionMap<ID, Location> = DuplicateCollectionMap::new (ids);

        // Test empty remove
        assert_eq! (map.remove (&(0, 0)), false);
        // Test non-empty remove
        map.insert ((0, (0, 0)));
        assert_eq! (map.remove (&(0, 0)), true);
        assert_eq! (map.get_first (&0).unwrap ().len (), 0);
        assert_eq! (map.get_second (&(0, 0)), None);
        assert_eq! (map.get_collection_second (&(0, 0)), None);
    }

    #[test]
    fn duplicate_collection_map_replace () {
        let ids: Vec<ID> = vec![0, 1, 2, 3];
        let mut map: DuplicateCollectionMap<ID, Location> = DuplicateCollectionMap::new (ids);

        // Test empty replace
        assert_eq! (map.replace ((0, 0), 0), None);
        // Test partial replace
        map.insert ((0, (1, 1)));
        assert_eq! (map.replace ((1, 1), 1).unwrap (), 0);
        assert_eq! (map.get_first (&1).unwrap ().len (), 1);
        assert_eq! (map.get_second (&(1, 1)).unwrap (), &1);
        assert_eq! (map.get_collection_second (&(1, 1)).unwrap ().len (), 1);
        assert_eq! (map.get_second (&(0, 0)), None);
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
    fn duplicate_collection_map_contains_key () {
        let ids: Vec<ID> = vec![0, 1];
        let mut map: DuplicateCollectionMap<ID, Location> = DuplicateCollectionMap::new (ids);

        // Test empty contains
        assert_eq! (map.contains_key_first (&0), true);
        assert_eq! (map.contains_key_second (&(0, 0)), false);
        // Test non-empty contains
        map.insert ((0, (0, 0)));
        assert_eq! (map.contains_key_first (&0), true);
        assert_eq! (map.contains_key_second (&(0, 0)), true);
        assert_eq! (map.contains_key_first (&1), true);
        assert_eq! (map.contains_key_second (&(1, 1)), false);
        assert_eq! (map.contains_key_first (&2), false);
    }
}
