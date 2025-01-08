use super::{COST_IMPASSABLE, COST_MINIMUM};
use crate::common::{Target, Timed, ID};
use crate::dynamic::{Adjustment, Appliable, Applier, Change, Changeable, Modifier, Statistic, Status, Trigger};
use crate::Scene;
use std::rc::Rc;

const CLIMB_MAX: u8 = 2;

#[derive (Debug)]
#[derive (Clone)]
pub struct Tile {
    scene: Rc<Scene>,
    modifier: Option<Modifier>,
    status: Option<Status>,
    terrain_id: ID,
    height: u8,
    city_id: Option<ID>,
    is_recruited: bool,
}

impl Tile {
    pub fn new (scene: Rc<Scene>, terrain_id: ID, height: u8, city_id: Option<ID>) -> Self {
        let modifier: Option<Modifier> = None;
        let status: Option<Status> = None;
        let is_recruited: bool = false;

        Self { scene, modifier, status, terrain_id, height, city_id, is_recruited }
    }

    pub fn get_cost (&self) -> u8 {
        let cost: u8 = self.scene.get_terrain (&self.terrain_id).get_cost ();

        match self.modifier {
            Some (m) => {
                let adjustment: Adjustment = m.get_adjustments ()[0];

                match adjustment.0 {
                    Statistic::Tile (f) => if f {
                        adjustment.1 as u8
                    } else if adjustment.2 {
                        cost + (adjustment.1 as u8)
                    } else {
                        u8::max (cost.saturating_sub (adjustment.1 as u8), COST_MINIMUM)
                    }
                    _ => panic! ("Invalid statistic {:?}", adjustment.0),
                }
            }
            None => cost,
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

    pub fn set_recruited (&mut self, is_recruited: bool) {
        self.is_recruited = is_recruited;
    }
}

impl Changeable for Tile {
    fn add_appliable (&mut self, appliable: Box<dyn Appliable>) -> bool {
        if let Change::Modifier ( .. ) = appliable.change () {
            let modifier: Modifier = appliable.modifier ();
            let adjustment: Adjustment = modifier.get_adjustments ()[0];

            if let Statistic::Tile ( .. ) = adjustment.0 {
                self.modifier = Some (modifier);

                true
            } else {
                false
            }
        } else {
            false
        }
    }

    fn add_status (&mut self, status: Status) -> bool {
        if let Change::Modifier ( .. ) = status.get_change () {
            let appliable: Box<dyn Appliable> = status.try_yield_appliable (Rc::clone (&self.scene))
                    .unwrap_or_else (|| panic! ("Appliable not found for status {:?}", status));

            if let Target::Map = status.get_target () {
                match status.get_trigger () {
                    Trigger::OnOccupy => { self.modifier = None; }
                    Trigger::None => { self.add_appliable (appliable); }
                    _ => return false,
                }

                self.status = Some (status);

                true
            } else {
                false
            }
        } else {
            false
        }
    }

    fn remove_modifier (&mut self, modifier_id: &ID) -> bool {
        let modifier: Option<Modifier> = self.modifier;

        if let Some (m) = modifier {
            if m.get_id () == *modifier_id {
                self.modifier = None;
    
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    fn remove_status (&mut self, status_id: &ID) -> bool {
        let status: Option<Status> = self.status;

        if let Some (s) = status {
            if s.get_id () == *status_id {
                if let Change::Modifier (m, _) = s.get_change () {
                    self.remove_modifier (&m);
                }

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
        if let Some (mut m) = self.modifier {
            let modifier: Option<Modifier> = if m.decrement_duration () {
                None
            } else {
                Some (m)
            };

            self.modifier = modifier;
        }

        if let Some (mut s) = self.status {
            let status: Option<Status> = if s.decrement_duration () {
                s.get_next_id ()
                        .map (|n: ID| *self.scene.get_status (&n))
            } else {
                Some (s)
            };

            self.status = status;
        }
    }
}

impl Applier for Tile {
    fn try_yield_appliable (&self, scene: Rc<Scene>) -> Option<Box<dyn Appliable>> {
        self.status.and_then (|s: Status| s.try_yield_appliable (scene))
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
    use crate::dynamic::ModifierBuilder;
    use crate::tests::generate_scene;

    fn generate_modifiers () -> (Box<Modifier>, Box<Modifier>, Box<Modifier>) {
        let scene = generate_scene ();
        let modifier_builder_0: &ModifierBuilder = scene.get_modifier_builder (&0);
        let modifier_0 = modifier_builder_0.build (false);
        let modifier_0 = Box::new (modifier_0);
        let modifier_builder_1: &ModifierBuilder = scene.get_modifier_builder (&1);
        let modifier_1 = modifier_builder_1.build (false);
        let modifier_1 = Box::new (modifier_1);
        let modifier_builder_2: &ModifierBuilder = scene.get_modifier_builder (&2);
        let modifier_2 = modifier_builder_2.build (false);
        let modifier_2 = Box::new (modifier_2);

        (modifier_0, modifier_1, modifier_2)
    }

    fn generate_statuses () -> (Status, Status, Status) {
        let scene = generate_scene ();
        let status_2 = *scene.get_status (&2);
        let status_3 = *scene.get_status (&3);
        let status_4 = *scene.get_status (&4);

        (status_2, status_3, status_4)
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
    }

    
    #[test]
    fn tile_add_status () {
        let scene = generate_scene ();
        let mut tile = Tile::new (Rc::clone (&scene), 1, 0, None);
        let (status_2, status_3, _) = generate_statuses ();

        // Test tile status
        assert_eq! (tile.get_cost (), 2);
        assert! (tile.add_status (status_3));
        assert! (tile.status.is_some ());
        assert_eq! (tile.get_cost (), 1);
        // Test applier status
        assert! (tile.add_status (status_2));
        assert! (tile.status.is_some ());
        assert! (tile.try_yield_appliable (Rc::clone (&scene)).is_some ());
    }

    #[test]
    fn tile_remove_modifier () {
        let scene = generate_scene ();
        let mut tile = Tile::new (Rc::clone (&scene), 1, 0, None);
        let (modifier_0, _, _) = generate_modifiers ();

        // Test empty remove
        assert! (!tile.remove_modifier (&0));
        assert! (tile.modifier.is_none ());
        // Test non-empty remove
        tile.add_appliable (modifier_0);
        assert_eq! (tile.get_cost (), 3);
        assert! (tile.remove_modifier (&0));
        assert_eq! (tile.get_cost (), 2);
        assert! (tile.modifier.is_none ());
    }

    #[test]
    fn tile_remove_status () {
        let scene = generate_scene ();
        let mut tile = Tile::new (Rc::clone (&scene), 1, 0, None);
        let (status_2, status_3, _) = generate_statuses ();

        // Test empty remove
        assert! (!tile.remove_status (&0));
        assert! (tile.status.is_none ());
        assert! (tile.modifier.is_none ());
        // Test non-empty remove
        tile.add_status (status_3);
        assert_eq! (tile.get_cost (), 1);
        assert! (tile.remove_status (&3));
        assert_eq! (tile.get_cost (), 2);
        assert! (tile.status.is_none ());
        assert! (tile.modifier.is_none ());
        // Test applier remove
        tile.add_status (status_2);
        assert! (tile.remove_status (&2));
        assert! (tile.status.is_none ());
        assert! (tile.modifier.is_none ());
    }

    #[test]
    fn tile_decrement_durations () {
        let scene = generate_scene ();
        let mut tile = Tile::new (Rc::clone (&scene), 0, 0, None);
        let (modifier_0, modifier_1, _) = generate_modifiers ();
        let (status_2, status_3, status_4) = generate_statuses ();

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

        // Test empty status
        tile.decrement_durations ();
        assert! (tile.status.is_none ());
        // Test timed status
        tile.add_status (status_2);
        tile.decrement_durations ();
        assert! (tile.status.is_some ());
        tile.decrement_durations ();
        assert! (tile.status.is_some ());
        tile.decrement_durations ();
        assert! (tile.status.is_none ());
        // Test permanent status
        tile.add_status (status_3);
        tile.decrement_durations ();
        assert! (tile.status.is_some ());
        tile.decrement_durations ();
        assert! (tile.status.is_some ());
        // Test linked status
        tile.add_status (status_4);
        tile.decrement_durations ();
        assert! (tile.status.is_some ());
        assert! (matches! (tile.status.unwrap ().get_next_id ().unwrap (), 3));
        tile.decrement_durations ();
        assert! (tile.status.is_some ());
        assert! (matches! (tile.status.unwrap ().get_next_id ().unwrap (), 3));
        tile.decrement_durations ();
        assert! (tile.status.is_some ());
        assert! (tile.status.unwrap ().get_next_id ().is_none ());
    }
}
