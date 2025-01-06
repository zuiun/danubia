use crate::dynamic::Applier;
use crate::map::Area;

mod faction;
pub use self::faction::Faction;
pub use self::faction::FactionBuilder;
mod magic;
pub use self::magic::Element;
pub use self::magic::Magic;
mod skill;
pub use self::skill::Activity;
pub use self::skill::Skill;
pub mod unit;
pub use self::unit::UnitStatistic;
pub use self::unit::Rank;
pub use self::unit::UnitStatistics;
pub use self::unit::Unit;
pub use self::unit::UnitBuilder;
mod weapon;
pub use self::weapon::WeaponStatistic;
pub use self::weapon::Weapon;

pub trait Tool: Applier {
    fn get_area (&self) -> Area;
    fn get_range (&self) -> u8;
}
