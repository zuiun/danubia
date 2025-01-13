use super::Tool;
use crate::common::{ID, Target, Timed};
use crate::dynamic::{Appliable, AppliableKind, Applier, Dynamic, Attribute, Trigger};
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
    attribute: Option<Attribute>,
}

impl Weapon {
    pub const fn new (id: ID, statistics: WeaponStatistics, area: Area, range: u8) -> Self {
        let attribute: Option<Attribute> = None;

        Self { id, statistics, area, range, attribute }
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

    fn add_attribute (&mut self, attribute: Attribute) -> bool {
        let kind: AppliableKind = attribute.get_kind ();

        if let AppliableKind::Modifier ( .. ) = kind {
            let target: Target = attribute.get_target ();

            if let Target::Enemy = target {
                let trigger: Trigger = attribute.get_trigger ();

                if let Trigger::OnAttack = trigger {
                    self.attribute = Some (attribute);

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

    fn remove_attribute (&mut self, attribute_id: &ID) -> bool {
        if let Some (s) = self.attribute {
            if s.get_id () == *attribute_id {
                self.attribute = None;

                true
            } else {
                false
            }
        } else {
            false
        }
    }

    fn decrement_durations (&mut self) {
        if let Some (mut attribute) = self.attribute {
            self.attribute = if attribute.decrement_duration () {
                Some (attribute)
            } else {
                None
            };
        }
    }
}

impl Applier for Weapon {
    fn try_yield_appliable (&self, scene: Rc<Scene>) -> Option<Box<dyn Appliable>> {
        self.attribute.and_then (|s: Attribute| s.try_yield_appliable (scene))
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
    fn weapon_add_attribute () {
        let scene = generate_scene ();
        let mut weapon = *scene.get_weapon (&0);
        let (attribute_6, _) = generate_attributes ();

        assert! (weapon.add_attribute (attribute_6));
        assert! (weapon.attribute.is_some ());
    }

    #[test]
    fn weapon_remove_attribute () {
        let scene = generate_scene ();
        let mut weapon = *scene.get_weapon (&0);
        let (attribute_6, _) = generate_attributes ();
    
        // Test empty remove
        assert! (!weapon.remove_attribute (&6));
        assert! (weapon.attribute.is_none ());
        // Test non-empty remove
        weapon.add_attribute (attribute_6);
        assert! (weapon.remove_attribute (&6));
        assert! (weapon.attribute.is_none ());
    }

    #[test]
    fn weapon_decrement_durations () {
        let scene = generate_scene ();
        let mut weapon = *scene.get_weapon (&0);
        let (attribute_6, attribute_7) = generate_attributes ();

        // Test empty attribute
        weapon.decrement_durations ();
        assert! (weapon.attribute.is_none ());
        // Test timed attribute
        weapon.add_attribute (attribute_6);
        weapon.decrement_durations ();
        assert! (weapon.attribute.is_some ());
        weapon.decrement_durations ();
        assert! (weapon.attribute.is_some ());
        weapon.decrement_durations ();
        assert! (weapon.attribute.is_none ());
        // Test permanent attribute
        weapon.add_attribute (attribute_7);
        weapon.decrement_durations ();
        assert! (weapon.attribute.is_some ());
        weapon.decrement_durations ();
        assert! (weapon.attribute.is_some ());
    }
}
