use crate::dynamic::Applier;
use crate::map::Area;

mod faction;
pub use self::faction::*;
mod magic;
pub use self::magic::*;
mod skill;
pub use self::skill::*;
mod unit;
pub use self::unit::*;
mod weapon;
pub use self::weapon::*;

pub trait Tool: Applier {
    fn get_area (&self) -> Area;
    fn get_range (&self) -> u8;
}
