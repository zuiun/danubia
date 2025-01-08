use crate::character::UnitStatistic;
use crate::common::{ID, Target};
use crate::Scene;
use std::rc::Rc;

mod change;
pub use self::change::Change;
mod effect;
pub use self::effect::Effect;
mod modifier;
pub use self::modifier::Modifier;
pub use self::modifier::ModifierBuilder;
mod status;
pub use self::status::Status;

pub type Adjustment = (Statistic, u16, bool); // statistic, change (value depends on context), is add

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
     * Creates a Change representation of self
     *
     * Pre: None
     * Post: None
     * Return: Change = self's type
     */
    fn change (&self) -> Change;
    /*
     * Gets self's statistic adjustments
     *
     * Pre: None
     * Post: None
     * Return: Adjustments = self's adjustments
     */
    fn get_adjustments (&self) -> &[Adjustment];
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
     * scene: Rc<Scene> = lists of all game objects
     *
     * Pre: None
     * Post: None
     * Return: Option<Box<dyn Appliable>> = None -> change unavailable, Some (change) -> change available
     */
    fn try_yield_appliable (&self, scene: Rc<Scene>) -> Option<Box<dyn Appliable>>;
    /*
     * Gets self's target
     *
     * Pre: None
     * Post: None
     * Return: Target
     */
    fn get_target (&self) -> Target;
}

pub trait Changeable {
    /*
     * Adds appliable to self
     * Fails if appliable isn't applicable to self
     * Targeted Status should use this
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
     * Non-targeted Status should use this
     *
     * status: Status = status to add
     *
     * Pre: None
     * Post: None
     * Return: bool = false -> add failed, true -> add succeeded
     */
    fn add_status (&mut self, status: Status) -> bool;
    /*
     * Removes modifier_id from self
     * Fails if modifier_id isn't applied to self
     * Status should use this
     *
     * modifier_id: ID = modifier to remove
     *
     * Pre: None
     * Post: None
     * Return: bool = false -> remove failed, true -> remove succeeded
     */
    fn remove_modifier (&mut self, modifier_id: &ID) -> bool;
    /*
     * Removes status_id from self
     * Fails if status_id isn't applied to self
     * Changeable should use this
     *
     * status_id: ID = status to remove
     *
     * Pre: None
     * Post: None
     * Return: bool = false -> remove failed, true -> remove succeeded
     */
    fn remove_status (&mut self, status_id: &ID) -> bool;
    /*
     * Decreases all of self's Timed's remaining durations
     *
     * Pre: None
     * Post: Timed's remaining duration is unchanged for permanent Timed
     */
    fn decrement_durations (&mut self);
}

#[derive (Debug)]
#[derive (Clone, Copy)]
pub enum Statistic {
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
