use super::Tool;
use crate::common::{ID, Target, Timed};
use crate::dynamic::{Appliable, AppliableKind, Applier, Dynamic, Status, Trigger};
use crate::map::Area;
use crate::Scene;
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
    id: ID,
    statistics: WeaponStatistics,
    area: Area,
    range: u8,
    status: Option<Status>,
}

impl Weapon {
    pub const fn new (id: ID, statistics: WeaponStatistics, area: Area, range: u8) -> Self {
        let status: Option<Status> = None;

        Self { id, statistics, area, range, status }
    }

    pub fn get_statistic (&self, statistic: WeaponStatistic) -> u16 {
        self.statistics[statistic as usize] as u16
    }

    pub fn get_id (&self) -> ID {
        self.id
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

impl Dynamic for Weapon {
    fn add_appliable (&mut self, _appliable: Box<dyn Appliable>) -> bool {
        unimplemented! ()
    }

    fn add_status (&mut self, status: Status) -> bool {
        assert! (status.get_next_id ().is_none ()); // Weapons don't support linked statuses

        let kind: AppliableKind = status.get_kind ();

        if let AppliableKind::Modifier ( .. ) = kind {
            let target: Target = status.get_target ();

            if let Target::Enemy = target {
                let trigger: Trigger = status.get_trigger ();

                if let Trigger::OnAttack = trigger {
                    self.status = Some (status);

                    true
                } else {
                    panic! ("Invalid trigger {:?}", trigger)
                }
            } else {
                panic! ("Invalid target {:?}", target);
            }
        } else {
            panic! ("Invalid appliable kind {:?}", kind)
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
        if let Some (mut status) = self.status {
            self.status = if status.decrement_duration () {
                None
            } else {
                Some (status)
            };
        }
    }
}

impl Applier for Weapon {
    fn try_yield_appliable (&self, scene: Rc<Scene>) -> Option<Box<dyn Appliable>> {
        self.status.and_then (|s: Status| s.try_yield_appliable (scene))
    }

    fn get_target (&self) -> Target {
        Target::Enemy
    }
}

#[cfg (test)]
mod tests {
    use super::*;
    use crate::tests::generate_scene;

    fn generate_statuses () -> (Status, Status) {
        let scene = generate_scene ();
        let status_6 = *scene.get_status (&6);
        let status_7 = *scene.get_status (&7);

        (status_6, status_7)
    }

    #[test]
    fn weapon_add_status () {
        let scene = generate_scene ();
        let mut weapon = *scene.get_weapon (&0);
        let (status_6, _) = generate_statuses ();

        assert! (weapon.add_status (status_6));
        assert! (weapon.status.is_some ());
    }

    #[test]
    fn weapon_remove_status () {
        let scene = generate_scene ();
        let mut weapon = *scene.get_weapon (&0);
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
        let scene = generate_scene ();
        let mut weapon = *scene.get_weapon (&0);
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
