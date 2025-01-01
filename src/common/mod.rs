mod information;
pub use self::information::Information;

pub type ID = usize;

pub const ID_UNINITIALISED: ID = ID::MAX;
pub const DURATION_PERMANENT: u16 = u16::MAX;

// pub fn checked_sub_or<T> (left: T, right: T, default: T, minimum: T) -> T
//         where T: Sub<Output = T> + PartialOrd + Copy {
//     let difference: T = if left < right {
//         default
//     } else {
//         left - right
//     };

//     if difference < minimum {
//         minimum
//     } else {
//         difference
//     }
// }

pub trait Timed {
    /*
     * Gets self's remaining duration
     *
     * Pre: None
     * Post: None
     * Return: u16 = permanent Timed -> DURATION_PERMANENT, limited Timed -> remaining duration
     */
    fn get_duration (&self) -> u16;
    /*
     * Decreases self's remaining duration
     *
     * Pre: None
     * Post: self's remaining duration is unchanged for permanent Timed
     * Return: bool = false -> not expired, true -> expired
     */
    fn dec_duration (&mut self) -> bool;
}

// Full range of targets only allowed for skills and magics
// Statuses only affect this (None), enemy (OnHit/OnAttack), or map (OnOccupy)
#[derive (Debug)]
#[derive (Clone, Copy)]
pub enum Target {
    This,
    Ally,
    Allies,
    Enemy,
    Enemies,
    // All,
    Map,
}

#[derive (Debug)]
#[derive (Clone, Copy)]
pub enum Capacity {
    Constant (u16, u16, u16), // current, maximum, base
    Quantity (u16, u16), // current, maximum
}
