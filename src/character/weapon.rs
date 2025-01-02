use std::cell::Cell;
use std::rc::Rc;
use crate::Lists;
use crate::common::{ID, Target, Timed};
use crate::dynamic::{Appliable, Applier, Change, Changeable, Status, Trigger};
use crate::map::Area;
use super::Tool;

type WeaponStatistics = [u8; WeaponStatistic::Length as usize];

#[derive (Debug)]
#[derive (Clone, Copy)]
pub enum WeaponStatistic {
    DMG, // damage - base damage
    SLH, // slash – modifier for physical damage, strong against manpower
    PRC, // pierce – modifier for physical damage, strong against morale
    DCY, // decay – modifier for magical damage
    Length,
}

#[derive (Debug)]
#[derive (Clone)]
pub struct Weapon {
    statistics: WeaponStatistics,
    area: Area,
    range: u8,
    status: Cell<Option<Status>>,
}

impl Weapon {
    pub const fn new (statistics: WeaponStatistics, area: Area, range: u8) -> Self {
        let status: Option<Status> = None;
        let status: Cell<Option<Status>> = Cell::new (status);

        Self { statistics, area, range, status }
    }

    pub fn get_statistic (&self, statistic: WeaponStatistic) -> u16 {
        self.statistics[statistic as usize] as u16
    }

    pub fn get_target (&self) -> Target {
        match self.area {
            Area::Single => Target::Enemy,
            Area::Radial ( .. ) | Area::Path ( .. ) => Target::Enemies,
        }
    }
}

impl Tool for Weapon {
    fn get_area (&self) -> Area {
        self.area
    }

    fn get_range (&self) -> u8 {
        self.range
    }
}

impl Changeable for Weapon {
    fn add_appliable (&self, _appliable: Box<dyn Appliable>) -> bool {
        unimplemented! ()
    }

    fn add_status (&self, status: Status) -> bool {
        assert! (status.get_next_id ().is_none ()); // Weapons don't support linked statuses

        if let Change::Modifier ( .. ) = status.get_change () {
            let target: Target = status.get_target ();

            if let Target::Enemy = target {
                if let Trigger::OnAttack = status.get_trigger () {
                    self.status.replace (Some (status));

                    true
                } else {
                    false
                }
            } else {
                false
            }
        } else {
            false
        }
    }

    fn remove_modifier (&self, _modifier_id: &ID) -> bool {
        unimplemented! () 
    }

    fn remove_status (&self, status_id: &ID) -> bool {
        if let Some (s) = self.status.get () {
            if s.get_id () == *status_id {
                self.status.replace (None);

                true
            } else {
                false
            }
        } else {
            false
        }
    }

    fn dec_durations (&self) -> () {
        if let Some (mut s) = self.status.get () {
            let status: Option<Status> = if s.dec_duration () {
                None
            } else {
                Some (s)
            };

            self.status.replace (status);
        }
    }
}

impl Applier for Weapon {
    fn try_yield_appliable (&self, lists: Rc<Lists>) -> Option<Box<dyn Appliable>> {
        self.status.get ().and_then (|s: Status| s.try_yield_appliable (lists))
    }

    fn get_target (&self) -> Target {
        Target::Enemy
    }
}

#[cfg (test)]
mod tests {
    use super::*;
    use crate::tests::generate_lists;

    fn generate_statuses () -> (Status, Status) {
        let lists = generate_lists ();
        let status_6 = lists.get_status (&6).clone ();
        let status_7 = lists.get_status (&7).clone ();

        (status_6, status_7)
    }

    #[test]
    fn weapon_remove_status () {
        let lists = generate_lists ();
        let weapon = lists.get_weapon (&0).clone ();
        let (status_6, _) = generate_statuses ();
    
        // Test empty remove
        assert_eq! (weapon.remove_status (&6), false);
        assert_eq! (weapon.status.get (), None);
        // Test non-empty remove
        weapon.add_status (status_6);
        assert_eq! (weapon.remove_status (&6), true);
        assert_eq! (weapon.status.get (), None);
    }

    #[test]
    fn weapon_dec_durations () {
        let lists = generate_lists ();
        let weapon = lists.get_weapon (&0).clone ();
        let (status_6, status_7) = generate_statuses ();

        // Test empty status
        weapon.dec_durations ();
        assert! (matches! (weapon.status.get (), None));
        // Test timed status
        weapon.add_status (status_6);
        weapon.dec_durations ();
        assert! (matches! (weapon.status.get (), Some { .. }));
        weapon.dec_durations ();
        assert! (matches! (weapon.status.get (), Some { .. }));
        weapon.dec_durations ();
        assert! (matches! (weapon.status.get (), None));
        // Test permanent status
        weapon.add_status (status_7);
        weapon.dec_durations ();
        assert! (matches! (weapon.status.get (), Some { .. }));
        weapon.dec_durations ();
        assert! (matches! (weapon.status.get (), Some { .. }));
    }
}
