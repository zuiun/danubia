use crate::character::UnitStatistic;
use crate::common::{ID, Target};
use crate::Scene;
use std::rc::Rc;

mod appliable_kind;
pub use self::appliable_kind::AppliableKind;
mod attribute;
pub use self::attribute::Attribute;
mod effect;
pub use self::effect::Effect;
mod modifier;
pub use self::modifier::Modifier;

pub type Adjustment = (StatisticKind, u16, bool); // statistic, change (value depends on context), is add

pub trait Appliable {
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
     * Creates an ownable Effect from self
     * Panics if creation fails
     *
     * Pre: self is an Effect
     * Post: None
     * Return: Effect = copy of self
     */
    fn effect (&self) -> Effect;
    /*
     * Creates an ownable Attribute from self
     * Panics if creation fails
     *
     * Pre: self is an Attribute
     * Post: None
     * Return: Attribute = copy of self
     */
    fn attribute (&self) -> Attribute;
    /*
     * Creates an AppliableKind representation of self
     *
     * Pre: None
     * Post: None
     * Return: AppliableKind = self's kind
     */
    fn kind (&self) -> AppliableKind;
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
    fn get_applier_id (&self) -> Option<ID>;
    fn set_applier_id (&mut self, applier_id: ID);
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

pub trait Dynamic {
    /*
     * Adds appliable to self
     * Fails if appliable isn't applicable to self
     * Targeted Attribute should use this
     *
     * appliable: Box<dyn Appliable> = appliable to add
     *
     * Pre: None
     * Post: None
     * Return: bool = false -> add failed, true -> add succeeded
     */
    fn add_appliable (&mut self, appliable: Box<dyn Appliable>) -> bool;
    /*
     * Adds attribute to self
     * Fails if attribute isn't applicable to self
     * Non-targeted Attribute should use this
     *
     * attribute: Attribute = attribute to add
     *
     * Pre: None
     * Post: None
     * Return: bool = false -> add failed, true -> add succeeded
     */
    fn add_attribute (&mut self, attribute: Attribute) -> bool;
    /*
     * Removes modifier_id from self
     * Fails if modifier_id isn't applied to self
     * Attribute should use this
     *
     * modifier_id: ID = modifier to remove
     *
     * Pre: None
     * Post: None
     * Return: bool = false -> remove failed, true -> remove succeeded
     */
    fn remove_modifier (&mut self, modifier_id: &ID) -> bool;
    /*
     * Removes attribute_id from self
     * Fails if attribute_id isn't applied to self
     * Changeable should use this
     *
     * attribute_id: ID = attribute to remove
     *
     * Pre: None
     * Post: None
     * Return: bool = false -> remove failed, true -> remove succeeded
     */
    fn remove_attribute (&mut self, attribute_id: &ID) -> bool;
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
pub enum StatisticKind {
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
    None, // units and tiles
}
