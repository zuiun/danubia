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
    map_first: HashMap<T, Option<U>>,
    map_second: HashMap<U, Option<T>>,
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
