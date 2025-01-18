use super::Tool;
use crate::common::{ID, Scene, Target, Timed};
use crate::dynamic::{Appliable, AppliableKind, Applier, Attribute, Dynamic, Trigger};
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
    id: ID,
    statistics: WeaponStatistics,
    area: Area,
    range: u8,
    attribute_on_attack: Option<Attribute>,
}

impl Weapon {
    pub const fn new (id: ID, statistics: WeaponStatistics, area: Area, range: u8) -> Self {
        let attribute_on_attack: Option<Attribute> = None;

        Self { id, statistics, area, range, attribute_on_attack }
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
    fn add_appliable (&mut self, appliable: Box<dyn Appliable>) -> bool {
        let kind: AppliableKind = appliable.kind ();

        if let AppliableKind::Attribute ( .. ) = kind {
            let attribute = appliable.attribute ();
            let trigger: Trigger = attribute.get_trigger ();

            if let Trigger::OnAttack = trigger {
                self.attribute_on_attack = Some (attribute);

                true
            } else {
                panic! ("Invalid trigger {:?}", trigger)
            }
        } else {
            panic! ("Invalid appliable kind {:?}", kind)
        }
    }

    fn remove_appliable (&mut self, appliable: AppliableKind) -> bool {
        if let AppliableKind::Attribute (attribute_id) = appliable {
            if let Some (attribute) = self.attribute_on_attack {
                if attribute.get_id () == attribute_id {
                    self.attribute_on_attack = None;
    
                    true
                } else {
                    false
                }
            } else {
                false
            }
        } else {
            panic! ("Invalid appliable kind {:?}", appliable)
        }
    }

    fn decrement_durations (&mut self) {
        if let Some (mut attribute) = self.attribute_on_attack {
            self.attribute_on_attack = if attribute.decrement_duration () {
                Some (attribute)
            } else {
                None
            };
        }
    }
}

impl Applier for Weapon {
    fn try_yield_appliable (&self, scene: Rc<Scene>) -> Option<Box<dyn Appliable>> {
        self.attribute_on_attack.and_then (|s: Attribute| s.try_yield_appliable (scene))
    }

    fn get_target (&self) -> Target {
        Target::Enemy
    }
}

#[cfg (test)]
mod tests {
    use super::*;
    use crate::tests::generate_scene;

    fn generate_attributes () -> (Attribute, Attribute) {
        let scene = generate_scene ();
        let attribute_6 = *scene.get_attribute (&6);
        let attribute_7 = *scene.get_attribute (&7);

        (attribute_6, attribute_7)
    }

    #[test]
    fn weapon_add_appliable () {
        let scene = generate_scene ();
        let mut weapon = *scene.get_weapon (&0);
        let (attribute_6, _) = generate_attributes ();
        let attribute_6 = Box::new (attribute_6);

        assert! (weapon.add_appliable (attribute_6));
        assert! (weapon.attribute_on_attack.is_some ());
    }

    #[test]
    fn weapon_remove_appliable () {
        let scene = generate_scene ();
        let mut weapon = *scene.get_weapon (&0);
        let (attribute_6, _) = generate_attributes ();
        let attribute_6 = Box::new (attribute_6);
    
        // Test empty remove
        assert! (!weapon.remove_appliable (AppliableKind::Attribute (6)));
        assert! (weapon.attribute_on_attack.is_none ());
        // Test non-empty remove
        weapon.add_appliable (attribute_6);
        assert! (weapon.remove_appliable (AppliableKind::Attribute (6)));
        assert! (weapon.attribute_on_attack.is_none ());
    }

    #[test]
    fn weapon_decrement_durations () {
        let scene = generate_scene ();
        let mut weapon = *scene.get_weapon (&0);
        let (attribute_6, attribute_7) = generate_attributes ();
        let attribute_6 = Box::new (attribute_6);
        let attribute_7 = Box::new (attribute_7);

        // Test empty attribute
        weapon.decrement_durations ();
        assert! (weapon.attribute_on_attack.is_none ());
        // Test timed attribute
        weapon.add_appliable (attribute_6);
        weapon.decrement_durations ();
        assert! (weapon.attribute_on_attack.is_some ());
        weapon.decrement_durations ();
        assert! (weapon.attribute_on_attack.is_some ());
        weapon.decrement_durations ();
        assert! (weapon.attribute_on_attack.is_none ());
        // Test permanent attribute
        weapon.add_appliable (attribute_7);
        weapon.decrement_durations ();
        assert! (weapon.attribute_on_attack.is_some ());
        weapon.decrement_durations ();
        assert! (weapon.attribute_on_attack.is_some ());
    }
}
