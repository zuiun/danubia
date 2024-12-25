pub use self::duplicate_map::DuplicateNaturalMap;
pub use self::duplicate_map::DuplicateInnerMap;
pub use self::duplicate_map::DuplicateCrossMap;
mod duplicate_map;

use core::fmt::Debug;
use std::{fmt, sync::atomic::{AtomicUsize, Ordering}};

pub type ID = usize; // Due to event values, ID is assumed to be an u8
pub type Adjustment = (Statistic, u16, bool); // statistic, change (value depends on context), is add
pub type Adjustments = [Option<Adjustment>; 4]; // Any more than 4 is probably excessive

pub const ID_UNINITIALISED: ID = ID::MAX;
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

impl Information {
    pub fn new (name: String, descriptions: Vec<String>, current_description: usize) -> Self {
        Self { name, descriptions, current_description }
    }

    pub fn debug () -> Self {
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
