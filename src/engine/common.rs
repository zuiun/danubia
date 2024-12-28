use std::ops::Sub;
use crate::engine::character::{UnitStatistic, WeaponStatistic};

pub type ID = usize; // Due to event values, ID is assumed to be at most an u8

pub const ID_UNINITIALISED: ID = ID::MAX;
pub const FACTION_NONE: ID = 0;
pub const DURATION_PERMANENT: u16 = u16::MAX;

pub fn checked_sub_or<T> (left: T, right: T, default: T, minimum: T) -> T
        where T: Sub<Output = T> + PartialOrd + Copy {
    let difference: T = if left < right {
        default
    } else {
        left - right
    };

    if difference < minimum {
        minimum
    } else {
        difference
    }
}

pub trait Timed {
    fn get_duration (&self) -> u16;
    fn dec_duration (&mut self) -> bool;
}

#[derive (Debug)]
#[derive (Clone, Copy)]
pub enum Area {
    Single,
    Radial (u8), // radius
    Path (u8), // width
}

// Full range of targets only allowed for skills and magics
// Statuses only affect this (None), enemy (OnHit/OnAttack), or map (OnOccupy)
#[derive (Debug)]
#[derive (Clone, Copy)]
pub enum Target {
    This,
    Ally,
    Enemy,
    Enemies,
    Allies,
    Map,
}

#[derive (Debug)]
#[derive (Clone, Copy)]
pub enum Capacity {
    Constant (u16, u16, u16), // current, maximum, base
    Quantity (u16, u16), // current, maximum
}
