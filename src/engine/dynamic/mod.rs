mod effect;
pub use self::effect::Effect;

mod modifier;
pub use self::modifier::Modifier;
pub use self::modifier::ModifierBuilder;

mod status;
pub use self::status::Status;

use std::rc::Rc;
use crate::engine::Lists;
use crate::engine::character::UnitStatistic;
use crate::engine::common::{ID, Target};

pub type Adjustment = (StatisticType, u16, bool); // statistic, change (value depends on context), is add
pub type Adjustments = [Option<Adjustment>; 4]; // Any more than 4 is probably excessive

pub trait Appliable {
    /*
     * Creates an ownable Effect from self
     * Panics if creation fails
     *
     * Pre: self is an Effect
     * Post: None
     * Return: Effect = copy of self
     */
    fn effect (&self) -> Effect;
    /*
     * Creates an ownable Modifier from self
     * Panics if creation fails
     *
     * Pre: self is a Modifier
     * Post: None
     * Return: Modifier = copy of self
     */
    fn modifier (&self) -> Modifier;
    /*
     * Gets self's Appliable type
     *
     * Pre: None
     * Post: None
     * Return: Change = self's type
     */
    fn get_change (&self) -> Change;
    /*
     * Gets self's statistic adjustments
     *
     * Pre: None
     * Post: None
     * Return: Adjustments = self's adjustments
     */
    fn get_adjustments (&self) -> Adjustments;
    /*
     * Gets whether self can stack or is flat change
     * Modifier -> can stack
     * Effect -> is flat change
     *
     * Pre: None
     * Post: None
     * Return: bool = false -> can't stack or is percentage change, true -> can stack or is flat change
     */
    fn can_stack_or_is_flat (&self) -> bool;
}

pub trait Applier {
    /*
     * Gets self's change
     *
     * lists: Rc<Lists> = lists of all game objects
     *
     * Pre: None
     * Post: None
     * Return: Option<Box<dyn Appliable>> = None -> change unavailable, Some (change) -> change available
     */
    fn try_yield_appliable (&self, lists: Rc<Lists>) -> Option<Box<dyn Appliable>>;
    fn get_target (&self) -> Option<Target>;
}

pub trait Changeable {
    /*
     * Adds appliable to self
     * Fails if appliable isn't applicable to self
     *
     * appliable: Box<dyn Appliable> = appliable to add
     *
     * Pre: None
     * Post: None
     * Return: bool = false -> add failed, true -> add succeeded
     */
    fn add_appliable (&mut self, appliable: Box<dyn Appliable>) -> bool;
    /*
     * Adds status to self
     * Fails if status isn't applicable to self
     *
     * status: Status = status to add
     *
     * Pre: None
     * Post: None
     * Return: bool = false -> add failed, true -> add succeeded
     */
    fn add_status (&mut self, status: Status) -> bool;
    /*
     * Decreases all of self's Timed's remaining durations
     *
     * Pre: None
     * Post: Timed's remaining duration is unchanged for permanent Timed
     */
    fn dec_durations (&mut self) -> ();
}

#[derive (Debug)]
#[derive (Clone, Copy)]
pub enum StatisticType {
    Unit (UnitStatistic),
    Tile (bool), // false = set to constant, true = flat change
}

#[derive (Debug)]
#[derive (Clone, Copy)]
#[derive (Eq, Hash, PartialEq)]
pub enum Trigger {
    OnHit, // units only
    OnAttack, // units (weapons) only
    OnOccupy, // tiles only
    None,
}

#[derive (Debug)]
#[derive (Clone, Copy)]
pub enum Change {
    Modifier (ID, bool), // modifier, is flat
    Effect (ID), // effect
}
