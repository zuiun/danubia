mod information;
pub use self::information::Information;

pub type ID = usize;

pub const ID_UNINITIALISED: ID = ID::MAX;
pub const DURATION_PERMANENT: u16 = u16::MAX;
pub const MULTIPLIER_ATTACK: f32 = 1.0;
pub const MULTIPLIER_SKILL: f32 = 1.4;
pub const MULTIPLIER_MAGIC: f32 = 1.4;
pub const MULTIPLIER_WAIT: f32 = 0.67;

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
    fn decrement_duration (&mut self) -> bool;
}

/*
 * Weapons only target Enemy or Enemies
 * Skills only target This, Ally, or Allies
 * Magics only target This or Map
 * Statuses only target This (None), Enemy (OnHit/OnAttack), or Map (OnOccupy)
 */
#[derive (Debug)]
#[derive (Clone, Copy)]
pub enum Target {
    This,
    Ally,
    Allies,
    Enemy,
    Enemies,
    // All,
    Map (ID), // applier unit
}

// #[derive (Debug)]
// #[derive (Clone, Copy)]
// pub enum Target {
//     Unit (ID),
//     Map (Location),
// }

#[derive (Debug)]
#[derive (Clone, Copy)]
pub enum Capacity {
    Constant (u16, u16, u16), // current, maximum, base
    Quantity (u16, u16), // current, maximum
}
