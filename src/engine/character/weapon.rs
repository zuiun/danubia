use std::rc::Rc;
use crate::engine::Lists;
use crate::engine::common::{Area, Target, Timed};
use crate::engine::dynamic::{Appliable, Applier, Change, Changeable, Status, Trigger};

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
}

impl Changeable for Weapon {
    fn add_appliable (&mut self, appliable: Box<dyn Appliable>) -> bool {
        unimplemented! ();

        // if let Change::Modifier (_, _) = appliable.get_change () {
        //     let modifier: Modifier = appliable.modifier ();
        //     let adjustment: Adjustment = modifier.get_adjustments ()[0].expect (&format! ("Adjustment not found for modifier {:?}", modifier));

        //     if let Statistic::Tile (_) = adjustment.0 {
        //         self.modifier = Some (modifier);

        //         true
        //     } else {
        //         false
        //     }
        // } else {
        //     false
        // }
    }

    fn add_status (&mut self, status: Status) -> bool {
        assert! (status.get_next_id ().is_none ()); // Weapons don't support linked statuses

        if let Change::Modifier (_, _) = status.get_change () {
            let target: Target = status.get_target ()
                    .expect (&format! ("Target not found for status {:?}", status));

            if let Target::Enemy = target {
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

    fn dec_durations (&mut self) -> () {
        if let Some (ref mut s) = self.status {
            if s.dec_duration () {
                self.status = None;
            }
        }
    }
}

impl Applier for Weapon {
    fn try_yield_appliable (&self, lists: Rc<Lists>) -> Option<Box<dyn Appliable>> {
        self.status.and_then (|s: Status| s.try_yield_appliable (lists))
    }

    fn get_target (&self) -> Option<Target> {
        Some (Target::Enemy)
    }
}

#[cfg (test)]
mod tests {
    use super::*;
    use std::rc::Rc;
    use crate::engine::Lists;
    use crate::engine::tests::generate_lists;

    fn generate_statuses () -> (Status, Status) {
        let lists: Rc<Lists> = generate_lists ();
        let status_6: Status = lists.get_status (&6).clone ();
        let status_7: Status = lists.get_status (&7).clone ();

        (status_6, status_7)
    }

    #[test]
    fn weapon_dec_durations () {
        let lists: Rc<Lists> = generate_lists ();
        let mut weapon: Weapon = lists.get_weapon (&0).clone ();
        let (status_6, status_7): (Status, Status) = generate_statuses ();

        // Test empty status
        weapon.dec_durations ();
        assert! (matches! (weapon.status, None));
        // Test timed status
        weapon.add_status (status_6);
        weapon.dec_durations ();
        assert! (matches! (weapon.status, Some { .. }));
        weapon.dec_durations ();
        assert! (matches! (weapon.status, Some { .. }));
        weapon.dec_durations ();
        assert! (matches! (weapon.status, None));
        // Test permanent status
        weapon.add_status (status_7);
        weapon.dec_durations ();
        assert! (matches! (weapon.status, Some { .. }));
        weapon.dec_durations ();
        assert! (matches! (weapon.status, Some { .. }));
    }
}
