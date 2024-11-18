use std::{collections::{HashMap, HashSet}, fmt, hash::Hash};

pub const HLT: usize = 0; // Morale – Willingness to fight
pub const STR: usize = 1; // Strength – Ability to fight
pub const ATK: usize = 2; // Attack – Physical damage
pub const DEF: usize = 3; // Defence – Physical resistance
pub const MAG: usize = 4; // Magic – Magical damage and resistance
pub const MOV: usize = 5; // Manoeuvre – Speed and movement
pub const ORG: usize = 6; // Cohesion – Unit modifier for formation effects and subordinate units
pub const SLH: usize = 0; // Slash – Weapon modifier for physical damage, strong against strength
pub const PRC: usize = 1; // Pierce – Weapon modifier for physical damage, strong against morale
pub const DCY: usize = 2; // Decay – Weapon modifier for magical damage

pub type ID = u8; // Up to 256 unique entities
pub type Location = (usize, usize);
pub type Movement = (isize, isize);
// pub type Statistics = [Option<Statistic>; ORG + 1];
pub type Adjustments = [Option<i8>; ORG + 1];

#[derive (Debug)]
pub enum Area {
    Single,
    Radial (u8), // radius
    Path (u8) // width
}

#[derive (Debug)]
pub enum Target {
    Ally (bool), // false = ally, true = self
    Allies (bool), // false = allies, true = self and allies
    All (bool), // false = enemies, true = allies and enemies
    Enemy
}

#[derive (Debug)]
pub enum Value {
    Constant (u8, u8), // current, base
    Capacity (u8, u8) // current, maximum
}

#[derive (Debug)]
pub enum Direction {
    Up,
    Right,
    Down,
    Left
}

#[derive (Debug)]
pub struct Information {
    name: String,
    descriptions: Vec<String>,
    current_description: usize
}

#[derive (Debug)]
pub struct Statistic {
    information: Information,
    value: Value
}

#[derive (Debug)]
pub struct Modifier {
    information: Information,
    adjustments: Adjustments
}

#[derive (Debug)]
pub struct Effect {
    information: Information,
    modifier: Modifier,
    duration: u8,
    next: Option<Box<Effect>>
}

#[derive (Debug)]
pub struct DuplicateMap<T, U> {
    map_first: HashMap<T, U>,
    map_second: HashMap<U, T>,
    map_first_collection: HashMap<T, HashSet<U>>,
    is_collection: bool
}

impl Information {
    pub fn new (name: String, descriptions: Vec<String>, current_description: usize) -> Self {
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
    pub fn new (information: Information, adjustments: Adjustments) -> Self {
        Self { information, adjustments }
    }
}

impl<T, U> DuplicateMap<T, U>
where T: Clone + std::fmt::Debug + Eq + Hash, U: Clone + Eq + Hash {
    pub fn new (collection: Option<Vec<T>>) -> Self {
        let map_first: HashMap<T, U> = HashMap::new ();
        let map_second: HashMap<U, T> = HashMap::new ();
        let mut map_first_collection: HashMap<T, HashSet<U>> = HashMap::new ();
        let is_collection: bool = match collection {
            Some (v) => {
                v.into_iter ().map (|t| map_first_collection.insert (t, HashSet::new ())).collect::<Vec<_>> ();

                true
            }
            None => false
        };

        Self { map_first, map_second, map_first_collection, is_collection }
    }

    pub fn insert (&mut self, values: (&T, &U)) -> Option<(U, T)> {
        assert! (!self.is_collection);

        let first_original: Option<U> = self.map_first.insert (values.0.clone (), values.1.clone ());
        let second_original: Option<T> = self.map_second.insert (values.1.clone (), values.0.clone ());

        assert_eq! (first_original.is_some (), second_original.is_some ());

        if first_original.is_some () && second_original.is_some () {
            Some ((first_original.unwrap (), second_original.unwrap ()))
        } else {
            None
        }
    }

    pub fn insert_collection (&mut self, values: (&T, &U)) -> Option<T> {
        assert! (self.is_collection);

        let first_collection: &mut HashSet<U> = match self.map_first_collection.get_mut (values.0) {
            Some (c) => c,
            None => panic! ("Collection not found for key {:?}", values.0)
        };
        let first_collection: bool = first_collection.insert (values.1.clone ());
        let second_original: Option<T> = self.map_second.insert (values.1.clone (), values.0.clone ());

        assert_eq! (first_collection, second_original.is_some ());

        if first_collection && second_original.is_some () {
            Some (second_original.unwrap ())
        } else {
            None
        }
    }

    pub fn get (&self, keys: (Option<&T>, Option<&U>)) -> (Option<&U>, Option<&T>) {
        assert! (!self.is_collection);

        let first_value: Option<&U> = match keys.0 {
            Some (k) => self.map_first.get (k),
            None => None
        };
        let second_value: Option<&T> = match keys.1 {
            Some (k) => self.map_second.get (k),
            None => None
        };

        (first_value, second_value)
    }

    pub fn get_collection (&self, keys: (Option<&T>, Option<&U>)) -> (Option<&HashSet<U>>, Option<&T>) {
        assert! (self.is_collection);

        let first_value: Option<&HashSet<U>> = match keys.0 {
            Some (k) => self.map_first_collection.get (k),
            None => None
        };
        let second_value: Option<&T> = match keys.1 {
            Some (k) => self.map_second.get (k),
            None => None
        };

        (first_value, second_value)
    }

    pub fn remove (&mut self, keys: (&T, &U)) -> bool {
        assert! (!self.is_collection);

        let first_original: Option<U> = self.map_first.remove (keys.0);
        let second_original: Option<T> = self.map_second.remove (keys.1);

        assert_eq! (first_original.is_some (), second_original.is_some ());

        first_original.is_some () && second_original.is_some ()
    }

    pub fn remove_collection (&mut self, keys: (&T, &U)) -> bool {
        assert! (self.is_collection);

        let first_collection: &mut HashSet<U> = match self.map_first_collection.get_mut (keys.0) {
            Some (c) => c,
            None => panic! ("Collection not found for key {:?}", keys.0)
        };
        let first_collection: bool = first_collection.remove (keys.1);
        let second_original: Option<T> = self.map_second.remove (keys.1);

        assert_eq! (first_collection, second_original.is_some ());

        first_collection
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

// pub fn get_cursor (&self) -> Cursor {
//     self.cursor
// }

impl fmt::Display for Information {
    fn fmt (&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write! (f, "{}\n{}", self.name, self.descriptions[self.current_description])
    }
}

impl fmt::Display for Effect {
    fn fmt (&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write! (f, "{}", self.information)
    }
}
