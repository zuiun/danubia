mod effect;
pub use self::effect::Effect;

mod modifier;
pub use self::modifier::Modifier;
pub use self::modifier::ModifierBuilder;

mod status;
pub use self::status::Status;

use std::rc::Rc;
use crate::engine::Lists;
use crate::engine::character::{UnitStatistic, WeaponStatistic};
use crate::engine::common::{Area, ID, Target};

pub type Adjustment = (Statistic, u16, bool); // statistic, change (value depends on context), is add
pub type Adjustments = [Option<Adjustment>; 4]; // Any more than 4 is probably excessive

pub trait Appliable {
    fn effect (&self) -> Effect;
    fn modifier (&self) -> Modifier;
    fn get_change (&self) -> Change;
    fn get_adjustments (&self) -> Adjustments;
    fn can_stack_or_is_flat (&self) -> bool;
}

pub trait Applier {
    fn try_yield_appliable (&self, lists: Rc<Lists>) -> Option<Box<dyn Appliable>>;
    fn get_target (&self) -> Option<Target>;
}

pub trait Changeable {
    fn add_appliable (&mut self, appliable: Box<dyn Appliable>) -> bool;
    fn add_status (&mut self, status: Status) -> bool;
    fn dec_durations (&mut self) -> ();
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

#[derive (Debug)]
#[derive (Clone, Copy)]
pub enum Change {
    Modifier (ID, bool), // modifier, is flat
    Effect (ID), // effect
}
