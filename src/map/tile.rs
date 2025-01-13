use super::{COST_IMPASSABLE, COST_MINIMUM};
use crate::common::{ID, Target, Timed};
use crate::dynamic::{Adjustment, Appliable, AppliableKind, Applier, Attribute, Dynamic, Modifier, StatisticKind, Trigger};
use crate::Scene;
use std::rc::Rc;

const CLIMB_MAX: u8 = 2;

#[derive (Debug)]
#[derive (Clone)]
pub struct Tile {
    scene: Rc<Scene>,
    modifier: Option<Modifier>,
    // modifier_weather: Option<Modifier>,
    attribute: Option<Attribute>,
    terrain_id: ID,
    height: u8,
    city_id: Option<ID>,
    is_recruited: bool,
    applier_id_modifier: Option<ID>,
    applier_id_attribute: Option<ID>,
}

impl Tile {
    pub fn new (scene: Rc<Scene>, terrain_id: ID, height: u8, city_id: Option<ID>) -> Self {
        let modifier: Option<Modifier> = None;
        let attribute: Option<Attribute> = None;
        let is_recruited: bool = false;
        let applier_id_modifier: Option<ID> = None;
        let applier_id_attribute: Option<ID> = None;

        Self { scene, modifier, attribute, terrain_id, height, city_id, is_recruited, applier_id_modifier, applier_id_attribute }
    }

    pub fn get_cost (&self) -> u8 {
        let cost: u8 = self.scene.get_terrain (&self.terrain_id).get_cost ();

        if let Some (modifier) = self.modifier {
            let (statistic, value, is_add): Adjustment = modifier.get_adjustments ()[0];

            match statistic {
                StatisticKind::Tile (is_flat) => if is_flat {
                    value as u8
                } else if is_add {
                    cost + (value as u8)
                } else {
                    u8::max (cost.saturating_sub (value as u8), COST_MINIMUM)
                }
                _ => panic! ("Invalid statistic {:?}", statistic),
            }
        } else {
            cost
        }
    }

    pub fn is_impassable (&self) -> bool {
        self.get_cost () == COST_IMPASSABLE
    }

    pub fn try_climb (&self, other: &Tile) -> Option<u8> {
        let climb: u8 = self.height.abs_diff (other.height);

        if climb < CLIMB_MAX {
            Some (climb)
        } else {
            None
        }
    }

    pub fn find_cost (&self, other: &Tile) -> u8 {
        if self.is_impassable () || other.is_impassable () {
            COST_IMPASSABLE
        } else {
            self.try_climb (other).map_or (COST_IMPASSABLE, |c: u8| other.get_cost () + c)
        }
    }

    pub fn get_terrain_id (&self) -> ID {
        self.terrain_id
    }

    pub fn get_modifier (&self) -> Option<Modifier> {
        self.modifier
    }

    pub fn get_height (&self) -> u8 {
        self.height
    }

    pub fn get_city_id (&self) -> Option<ID> {
        self.city_id
    }

    pub fn is_recruited (&self) -> bool {
        self.is_recruited
    }

    pub fn get_applier_id_modifier (&self) -> Option<ID> {
        self.applier_id_modifier
    }

    pub fn get_applier_id_attribute (&self) -> Option<ID> {
        self.applier_id_attribute
    }

    pub fn set_recruited (&mut self, is_recruited: bool) {
        self.is_recruited = is_recruited;
    }
}

impl Dynamic for Tile {
    fn add_appliable (&mut self, appliable: Box<dyn Appliable>) -> bool {
        let kind: AppliableKind = appliable.kind ();

        match kind {
            AppliableKind::Modifier ( .. ) => {
                let modifier: Modifier = appliable.modifier ();
                let adjustment: Adjustment = modifier.get_adjustments ()[0];
    
                if let StatisticKind::Tile ( .. ) = adjustment.0 {
                    self.modifier = Some (modifier);
                    self.applier_id_modifier = modifier.get_applier_id ();
    
                    true
                } else {
                    panic! ("Invalid statistic kind {:?}", adjustment.0)
                }
            }
            AppliableKind::Effect ( .. ) => panic! ("Invalid appliable kind {:?}", kind),
            AppliableKind::Attribute ( .. ) => {
                let attribute: Attribute = appliable.attribute ();
                let kind: AppliableKind = attribute.get_kind ();

                if let AppliableKind::Modifier ( .. ) = attribute.get_kind () {
                    let trigger: Trigger = attribute.get_trigger ();

                    if let Trigger::OnOccupy = trigger {
                        self.modifier = None;
                    } else {
                        panic! ("Invalid trigger {:?}", trigger)
                    }

                    self.attribute = Some (attribute);
                    self.applier_id_attribute = attribute.get_applier_id ();

                    true
                } else {
                    panic! ("Invalid appliable kind {:?}", kind)
                }
            }
        }
    }

    fn remove_appliable (&mut self, appliable: AppliableKind) -> bool {
        match appliable {
            AppliableKind::Modifier (modifier_id) => {
                if let Some (modifier) = self.modifier {
                    if modifier.get_id () == modifier_id {
                        self.modifier = None;
                        self.applier_id_modifier = None;
            
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            AppliableKind::Effect ( .. ) => unimplemented! (),
            AppliableKind::Attribute (attribute_id) => {
                if let Some (attribute) = self.attribute {
                    if attribute.get_id () == attribute_id {
                        if let AppliableKind::Modifier (m) = attribute.get_kind () {
                            self.remove_appliable (AppliableKind::Modifier (m));
                        }
        
                        self.attribute = None;
                        self.applier_id_attribute = None;
        
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
        }
    }

    fn decrement_durations (&mut self) {
        if let Some (mut m) = self.modifier {
            self.modifier = if m.decrement_duration () {
                Some (m)
            } else {
                m.get_next_id ().map (|n: ID| *self.scene.get_modifier (&n))
            };
        }

        if let Some (mut s) = self.attribute {
            self.attribute = if s.decrement_duration () {
                Some (s)
            } else {
                None
            };
        }
    }
}

impl Applier for Tile {
    fn try_yield_appliable (&self, scene: Rc<Scene>) -> Option<Box<dyn Appliable>> {
        self.attribute.and_then (|s: Attribute| s.try_yield_appliable (scene))
    }

    fn get_target (&self) -> Target {
        Target::Map
    }
}

#[derive (Debug)]
pub struct TileBuilder {
    terrain_id: ID,
    height: u8,
    city_id: Option<ID>,
}

impl TileBuilder {
    pub const fn new (terrain_id: ID, height: u8, city_id: Option<ID>) -> Self {
        Self { terrain_id, height, city_id }
    }

    pub fn build (&self, scene: Rc<Scene>) -> Tile {
        Tile::new (Rc::clone (&scene), self.terrain_id, self.height, self.city_id)
    }
}

#[cfg (test)]
mod tests {
    use super::*;
    use crate::dynamic::Modifier;
    use crate::tests::generate_scene;

    fn generate_modifiers () -> (Box<Modifier>, Box<Modifier>, Box<Modifier>) {
        let scene = generate_scene ();
        let modifier_0 = *scene.get_modifier (&0);
        let modifier_0 = Box::new (modifier_0);
        let modifier_1 = *scene.get_modifier (&1);
        let modifier_1 = Box::new (modifier_1);
        let modifier_2 = *scene.get_modifier (&2);
        let modifier_2 = Box::new (modifier_2);

        (modifier_0, modifier_1, modifier_2)
    }

    fn generate_attributes () -> (Box<Attribute>, Box<Attribute>, Box<Attribute>) {
        let scene = generate_scene ();
        let attribute_2 = *scene.get_attribute (&2);
        let attribute_2 = Box::new (attribute_2);
        let attribute_3 = *scene.get_attribute (&3);
        let attribute_3 = Box::new (attribute_3);
        let attribute_4 = *scene.get_attribute (&4);
        let attribute_4 = Box::new (attribute_4);

        (attribute_2, attribute_3, attribute_4)
    }

    #[test]
    fn tile_get_cost () {
        let scene = generate_scene ();
        let tile_0 = Tile::new (Rc::clone (&scene), 0, 0, None);
        let tile_1 = Tile::new (Rc::clone (&scene), 1, 0, None);
        let tile_2 = Tile::new (Rc::clone (&scene), 2, 0, None);

        assert_eq! (tile_0.get_cost (), 1);
        assert_eq! (tile_1.get_cost (), 2);
        assert_eq! (tile_2.get_cost (), 0);
    }

    #[test]
    fn tile_is_impassable () {
        let scene = generate_scene ();
        let tile_0 = Tile::new (Rc::clone (&scene), 0, 0, None);
        let tile_2 = Tile::new (Rc::clone (&scene), 2, 0, None);

        // Test passable tile
        assert! (!tile_0.is_impassable ());
        // Test impassable tile
        assert! (tile_2.is_impassable ());
    }

    #[test]
    fn tile_try_climb () {
        let scene = generate_scene ();
        let tile_0 = Tile::new (Rc::clone (&scene), 0, 0, None);
        let tile_1_0 = Tile::new (Rc::clone (&scene), 1, 0, None);
        let tile_1_1 = Tile::new (Rc::clone (&scene), 1, 1, None);
        let tile_1_2 = Tile::new (Rc::clone (&scene), 1, 2, None);

        // Test impassable climb
        assert! (tile_0.try_climb (&tile_1_2).is_none ());
        assert! (tile_1_2.try_climb (&tile_0).is_none ());
        // Test passable climb
        assert_eq! (tile_0.try_climb (&tile_1_0).unwrap (), 0);
        assert_eq! (tile_1_0.try_climb (&tile_0).unwrap (), 0);
        assert_eq! (tile_1_0.try_climb (&tile_1_1).unwrap (), 1);
        assert_eq! (tile_1_1.try_climb (&tile_1_0).unwrap (), 1);
    }

    #[test]
    fn tile_find_cost () {
        let scene = generate_scene ();
        let tile_0 = Tile::new (Rc::clone (&scene), 0, 0, None);
        let tile_1_0 = Tile::new (Rc::clone (&scene), 1, 0, None);
        let tile_1_1 = Tile::new (Rc::clone (&scene), 1, 1, None);
        let tile_2 = Tile::new (Rc::clone (&scene), 2, 0, None);

        // Test impassable cost
        assert_eq! (tile_0.find_cost (&tile_2), 0);
        assert_eq! (tile_2.find_cost (&tile_0), 0);
        // Test passable cost
        assert_eq! (tile_0.find_cost (&tile_1_0), 2);
        assert_eq! (tile_1_0.find_cost (&tile_0), 1);
        assert_eq! (tile_0.find_cost (&tile_1_1), 3);
        assert_eq! (tile_1_1.find_cost (&tile_0), 2);
    }

    #[test]
    fn tile_add_appliable () {
        let scene = generate_scene ();
        let mut tile = Tile::new (Rc::clone (&scene), 1, 0, None);
        let (modifier_0, modifier_1, modifier_2) = generate_modifiers ();
        let (attribute_2, _, _) = generate_attributes ();

        // Test additive modifier
        assert_eq! (tile.get_cost (), 2);
        assert! (tile.add_appliable (modifier_0));
        assert! (tile.modifier.is_some ());
        assert_eq! (tile.get_cost (), 3);
        // Test subtractive modifier
        assert! (tile.add_appliable (modifier_1));
        assert! (tile.modifier.is_some ());
        assert_eq! (tile.get_cost (), 1);
        // Test constant modifier
        assert! (tile.add_appliable (modifier_2));
        assert! (tile.modifier.is_some ());
        assert_eq! (tile.get_cost (), 1);

        // Test attribute
        assert! (tile.add_appliable (attribute_2));
        assert! (tile.attribute.is_some ());
        assert! (tile.try_yield_appliable (Rc::clone (&scene)).is_some ());
    }

    #[test]
    fn tile_remove_appliable () {
        let scene = generate_scene ();
        let mut tile = Tile::new (Rc::clone (&scene), 1, 0, None);
        let (modifier_0, _, _) = generate_modifiers ();
        let (_, attribute_3, _) = generate_attributes ();

        // Test empty remove
        assert! (!tile.remove_appliable (AppliableKind::Modifier (0)));
        assert! (tile.modifier.is_none ());
        // Test non-empty remove
        tile.add_appliable (modifier_0);
        assert_eq! (tile.get_cost (), 3);
        assert! (tile.remove_appliable (AppliableKind::Modifier (0)));
        assert_eq! (tile.get_cost (), 2);
        assert! (tile.modifier.is_none ());

        // Test empty remove
        assert! (!tile.remove_appliable (AppliableKind::Attribute (0)));
        assert! (tile.attribute.is_none ());
        assert! (tile.modifier.is_none ());
        // Test non-empty remove
        tile.add_appliable (attribute_3);
        assert! (tile.remove_appliable (AppliableKind::Attribute (3)));
        assert! (tile.attribute.is_none ());
    }

    #[test]
    fn tile_decrement_durations () {
        let scene = generate_scene ();
        let mut tile = Tile::new (Rc::clone (&scene), 0, 0, None);
        let (modifier_0, modifier_1, modifier_2) = generate_modifiers ();
        let (attribute_2, attribute_3, _) = generate_attributes ();

        // Test empty modifier
        tile.decrement_durations ();
        assert! (tile.modifier.is_none ());
        // Test timed modifier
        tile.add_appliable (modifier_0);
        tile.decrement_durations ();
        assert! (tile.modifier.is_some ());
        tile.decrement_durations ();
        assert! (tile.modifier.is_some ());
        tile.decrement_durations ();
        assert! (tile.modifier.is_none ());
        // Test permanent modifier
        tile.add_appliable (modifier_1);
        tile.decrement_durations ();
        assert! (tile.modifier.is_some ());
        tile.decrement_durations ();
        assert! (tile.modifier.is_some ());
        // Test linked modifier
        tile.add_appliable (modifier_2);
        tile.decrement_durations ();
        assert! (tile.modifier.is_some ());
        assert_eq! (tile.modifier.unwrap ().get_next_id ().unwrap (), 0);
        tile.decrement_durations ();
        assert! (tile.modifier.is_some ());
        assert! (tile.modifier.unwrap ().get_next_id ().is_none ());

        // Test empty attribute
        tile.decrement_durations ();
        assert! (tile.attribute.is_none ());
        // Test timed attribute
        tile.add_appliable (attribute_2);
        tile.decrement_durations ();
        assert! (tile.attribute.is_some ());
        tile.decrement_durations ();
        assert! (tile.attribute.is_some ());
        tile.decrement_durations ();
        assert! (tile.attribute.is_none ());
        // Test permanent attribute
        tile.add_appliable (attribute_3);
        tile.decrement_durations ();
        assert! (tile.attribute.is_some ());
        tile.decrement_durations ();
        assert! (tile.attribute.is_some ());
    }
}
