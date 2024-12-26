mod duplicate_map;
pub use self::duplicate_map::DuplicateInnerMap;
pub use self::duplicate_map::DuplicateOuterMap;
pub use self::duplicate_map::DuplicateCrossMap;

mod information;
pub use self::information::Information;

mod modifier;
pub use self::modifier::Modifiable;
pub use self::modifier::Modifier;

mod status;
pub use self::status::Status;

use std::sync::atomic::AtomicUsize;
use crate::engine::character::{UnitStatistic, WeaponStatistic};

pub type ID = usize; // Due to event values, ID is assumed to be at most an u8
pub type Adjustment = (Statistic, u16, bool); // statistic, change (value depends on context), is add
pub type Adjustments = [Option<Adjustment>; 4]; // Any more than 4 is probably excessive

pub const ID_UNINITIALISED: ID = ID::MAX;
pub const DURATION_PERMANENT: u16 = u16::MAX;

pub trait Timed {
    fn get_duration (&self) -> u16;
    fn dec_duration (&mut self) -> bool;
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
pub enum Condition {
    OnHit,
    OnAttack
}
