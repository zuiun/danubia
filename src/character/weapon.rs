use super::Tool;
use crate::Lists;
use crate::common::{ID, Target, Timed};
use crate::dynamic::{Appliable, Applier, Change, Changeable, Status, Trigger};
use crate::map::Area;
use std::rc::Rc;

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
#[derive (Clone, Copy)]
pub struct Weapon {
    statistics: WeaponStatistics,
    area: Area,
    range: u8,
    status: Option<Status>,
}

impl Weapon {
    pub const fn new (statistics: WeaponStatistics, area: Area, range: u8) -> Self {
        let status: Option<Status> = None;

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
    fn add_appliable (&mut self, _appliable: Box<dyn Appliable>) -> bool {
        unimplemented! ()
    }

    fn add_status (&mut self, status: Status) -> bool {
        assert! (status.get_next_id ().is_none ()); // Weapons don't support linked statuses

        if let Change::Modifier ( .. ) = status.get_change () {
            if let Target::Enemy = status.get_target () {
                if let Trigger::OnAttack = status.get_trigger () {
                    self.status = Some (status);

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

    fn remove_modifier (&mut self, _modifier_id: &ID) -> bool {
        unimplemented! () 
    }

    fn remove_status (&mut self, status_id: &ID) -> bool {
        if let Some (s) = self.status {
            if s.get_id () == *status_id {
                self.status = None;

                true
            } else {
                false
            }
        } else {
            false
        }
    }

    fn decrement_durations (&mut self) {
        if let Some (mut s) = self.status {
            let status: Option<Status> = if s.decrement_duration () {
                None
            } else {
                Some (s)
            };

            self.status = status;
        }
    }
}

impl Applier for Weapon {
    fn try_yield_appliable (&self, lists: Rc<Lists>) -> Option<Box<dyn Appliable>> {
        self.status.and_then (|s: Status| s.try_yield_appliable (lists))
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
        let status_6 = *lists.get_status (&6);
        let status_7 = *lists.get_status (&7);

        (status_6, status_7)
    }

    #[test]
    fn weapon_add_status () {
        let lists = generate_lists ();
        let mut weapon = *lists.get_weapon (&0);
        let (status_6, _) = generate_statuses ();

        assert! (weapon.add_status (status_6));
        assert! (weapon.status.is_some ());
    }

    #[test]
    fn weapon_remove_status () {
        let lists = generate_lists ();
        let mut weapon = *lists.get_weapon (&0);
        let (status_6, _) = generate_statuses ();
    
        // Test empty remove
        assert! (!weapon.remove_status (&6));
        assert! (weapon.status.is_none ());
        // Test non-empty remove
        weapon.add_status (status_6);
        assert! (weapon.remove_status (&6));
        assert! (weapon.status.is_none ());
    }

    #[test]
    fn weapon_decrement_durations () {
        let lists = generate_lists ();
        let mut weapon = *lists.get_weapon (&0);
        let (status_6, status_7) = generate_statuses ();

        // Test empty status
        weapon.decrement_durations ();
        assert! (weapon.status.is_none ());
        // Test timed status
        weapon.add_status (status_6);
        weapon.decrement_durations ();
        assert! (weapon.status.is_some ());
        weapon.decrement_durations ();
        assert! (weapon.status.is_some ());
        weapon.decrement_durations ();
        assert! (weapon.status.is_none ());
        // Test permanent status
        weapon.add_status (status_7);
        weapon.decrement_durations ();
        assert! (weapon.status.is_some ());
        weapon.decrement_durations ();
        assert! (weapon.status.is_some ());
    }
}
