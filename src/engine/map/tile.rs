use std::fmt;
use std::rc::Rc;
use crate::engine::Lists;
use crate::engine::common::{Target, Timed, ID};
use crate::engine::dynamic::{Adjustment, Appliable, Applier, Change, Changeable, Effect, Modifier, ModifierBuilder, StatisticType, Status, Trigger};
use super::{COST_IMPASSABLE, COST_MINIMUM};

const CLIMB_MAX: u8 = 2;

#[derive (Debug)]
pub struct Tile {
    lists: Rc<Lists>,
    modifier: Option<Modifier>,
    status: Option<Status>,
    terrain_id: ID,
    height: u8,
    city_id: Option<ID>,
}

impl Tile {
    pub fn new (lists: Rc<Lists>, terrain_id: ID, height: u8, city_id: Option<ID>) -> Self {
        let modifier: Option<Modifier> = None;
        let status: Option<Status> = None;

        Self { lists, modifier, status, terrain_id, height, city_id }
    }

    pub fn get_cost (&self) -> u8 {
        let cost: u8 = self.lists.get_terrain (&self.terrain_id).get_cost ();

        match self.modifier {
            Some (m) => {
                let adjustment: Adjustment = m.get_adjustments ()[0]
                        .expect (&format! ("Adjustment not found for modifier {:?}", m));

                match adjustment.0 {
                    StatisticType::Tile (f) => if f {
                        adjustment.1 as u8
                    } else {
                        if adjustment.2 {
                            cost + (adjustment.1 as u8)
                        } else {
                            u8::max (cost.checked_sub (adjustment.1 as u8).unwrap_or (COST_MINIMUM), COST_MINIMUM)
                        }
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
}

impl Changeable for Tile {
    fn add_appliable (&mut self, appliable: Box<dyn Appliable>) -> bool {
        if let Change::Modifier (_, _) = appliable.get_change () {
            let modifier: Modifier = appliable.modifier ();
            let adjustment: Adjustment = modifier.get_adjustments ()[0]
                    .expect (&format! ("Adjustment not found for modifier {:?}", modifier));

            if let StatisticType::Tile (_) = adjustment.0 {
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
        if let Change::Modifier (m, s) = status.get_change () {
            let appliable: Box<dyn Appliable> = status.try_yield_appliable (Rc::clone (&self.lists))
                    .expect (&format! ("Appliable not found for status {:?}", status));
            let target: Target = status.get_target ()
                    .expect (&format! ("Target not found for status {:?}", status));

            if let Target::Map = target {
                match status.get_trigger () {
                    Trigger::OnOccupy => self.modifier = None,
                    Trigger::None => { self.add_appliable (appliable); }
                    _ => (),
                }
            }

            self.status = Some (status);

            true
        } else {
            false
        }
    }

    fn dec_durations (&mut self) -> () {
        if let Some (ref mut m) = self.modifier {
            if m.dec_duration () {
                self.modifier = None;
            }
        }

        if let Some (ref mut s) = self.status {
            if s.dec_duration () {
                self.status = s.get_next_id ().and_then (|n: ID|
                    Some (self.lists.get_status (&n).clone ())
                );
            }
        }
    }
}

impl Applier for Tile {
    fn try_yield_appliable (&self, lists: Rc<Lists>) -> Option<Box<dyn Appliable>> {
        self.status.and_then (|s| s.try_yield_appliable (lists))
    }

    fn get_target (&self) -> Option<Target> {
        self.status.and_then (|s: Status| s.get_target ())
    }
}

impl fmt::Display for Tile {
    fn fmt (&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write! (f, "{}.{}", self.terrain_id, self.height)
    }
}

pub struct TileBuilder {
    lists: Rc<Lists>,
}

impl TileBuilder {
    pub fn new (lists: Rc<Lists>) -> Self {
        Self { lists }
    }

    pub fn build (&self, terrain_id: ID, height: u8, city_id: Option<ID>) -> Tile {
        Tile::new (Rc::clone (&self.lists), terrain_id, height, city_id)
    }
}

#[cfg (test)]
mod tests {
    use super::*;
    use crate::engine::common::DURATION_PERMANENT;
    use crate::engine::tests::generate_lists;

    fn generate_modifiers () -> (Box<Modifier>, Box<Modifier>, Box<Modifier>) {
        let lists: Rc<Lists> = generate_lists ();
        let modifier_builder_0: &ModifierBuilder = lists.get_modifier_builder (&0);
        let modifier_0: Modifier = modifier_builder_0.build (2, false);
        let modifier_0: Box<Modifier> = Box::new (modifier_0);
        let modifier_builder_1: &ModifierBuilder = lists.get_modifier_builder (&1);
        let modifier_1: Modifier = modifier_builder_1.build (DURATION_PERMANENT, false);
        let modifier_1: Box<Modifier> = Box::new (modifier_1);
        let modifier_builder_2: &ModifierBuilder = lists.get_modifier_builder (&2);
        let modifier_2: Modifier = modifier_builder_2.build (1, false);
        let modifier_2: Box<Modifier> = Box::new (modifier_2);

        (modifier_0, modifier_1, modifier_2)
    }

    fn generate_statuses () -> (Status, Status, Status) {
        let lists: Rc<Lists> = generate_lists ();
        let status_2: Status = lists.get_status (&2).clone ();
        let status_3: Status = lists.get_status (&3).clone ();
        let status_4: Status = lists.get_status (&4).clone ();

        (status_2, status_3, status_4)
    }

    #[test]
    fn tile_get_cost () {
        let lists: Rc<Lists> = generate_lists ();
        let tile_0: Tile = Tile::new (Rc::clone (&lists), 0, 0, None);
        let tile_1: Tile = Tile::new (Rc::clone (&lists), 1, 0, None);
        let tile_2: Tile = Tile::new (Rc::clone (&lists), 2, 0, None);

        assert_eq! (tile_0.get_cost (), 1);
        assert_eq! (tile_1.get_cost (), 2);
        assert_eq! (tile_2.get_cost (), 0);
    }

    #[test]
    fn tile_is_impassable () {
        let lists: Rc<Lists> = generate_lists ();
        let tile_0: Tile = Tile::new (Rc::clone (&lists), 0, 0, None);
        let tile_2: Tile = Tile::new (Rc::clone (&lists), 2, 0, None);

        // Test passable tile
        assert! (!tile_0.is_impassable ());
        // Test impassable tile
        assert! (tile_2.is_impassable ());
    }

    #[test]
    fn tile_try_climb () {
        let lists: Rc<Lists> = generate_lists ();
        let tile_0: Tile = Tile::new (Rc::clone (&lists), 0, 0, None);
        let tile_1_0: Tile = Tile::new (Rc::clone (&lists), 1, 0, None);
        let tile_1_1: Tile = Tile::new (Rc::clone (&lists), 1, 1, None);
        let tile_1_2: Tile = Tile::new (Rc::clone (&lists), 1, 2, None);

        // Test impassable climb
        assert_eq! (tile_0.try_climb (&tile_1_2), None);
        assert_eq! (tile_1_2.try_climb (&tile_0), None);
        // Test passable climb
        assert_eq! (tile_0.try_climb (&tile_1_0).unwrap (), 0);
        assert_eq! (tile_1_0.try_climb (&tile_0).unwrap (), 0);
        assert_eq! (tile_1_0.try_climb (&tile_1_1).unwrap (), 1);
        assert_eq! (tile_1_1.try_climb (&tile_1_0).unwrap (), 1);
    }

    #[test]
    fn tile_find_cost () {
        let lists: Rc<Lists> = generate_lists ();
        let tile_0: Tile = Tile::new (Rc::clone (&lists), 0, 0, None);
        let tile_1_0: Tile = Tile::new (Rc::clone (&lists), 1, 0, None);
        let tile_1_1: Tile = Tile::new (Rc::clone (&lists), 1, 1, None);
        let tile_2: Tile = Tile::new (Rc::clone (&lists), 2, 0, None);

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
        let lists: Rc<Lists> = generate_lists ();
        let mut tile: Tile = Tile::new (Rc::clone (&lists), 0, 0, None);
        let (modifier_0, modifier_1, modifier_2): (Box<Modifier>, Box<Modifier>, Box<Modifier>) = generate_modifiers ();

        // Test additive modifier
        assert_eq! (tile.add_appliable (modifier_0), true);
        assert! (matches! (tile.modifier, Some { .. }));
        assert_eq! (tile.get_cost (), 2);
        // Test subtractive modifier
        assert_eq! (tile.add_appliable (modifier_1), true);
        assert! (matches! (tile.modifier, Some { .. }));
        assert_eq! (tile.get_cost (), 1);
        tile.terrain_id = 1;
        assert_eq! (tile.get_cost (), 1);
        // Test constant modifier
        assert_eq! (tile.add_appliable (modifier_2), true);
        assert! (matches! (tile.modifier, Some { .. }));
        assert_eq! (tile.get_cost (), 1);
    }

    
    #[test]
    fn tile_add_status () {
        let lists: Rc<Lists> = generate_lists ();
        let mut tile: Tile = Tile::new (Rc::clone (&lists), 0, 0, None);
        let (status_2, status_3, _): (Status, Status, _) = generate_statuses ();

        // Test tile status
        assert_eq! (tile.add_status (status_3), true);
        assert! (matches! (tile.status, Some { .. }));
        assert_eq! (tile.get_cost (), 1);
        tile.terrain_id = 1;
        assert_eq! (tile.get_cost (), 1);
        // Test applier status
        assert_eq! (tile.add_status (status_2), true);
        assert! (matches! (tile.status, Some { .. }));
        assert_eq! (tile.get_cost (), 2);
        assert! (matches! (tile.try_yield_appliable (Rc::clone (&lists)), Some { .. }));
    }

    #[test]
    fn tile_dec_durations () {
        let lists: Rc<Lists> = generate_lists ();
        let mut tile: Tile = Tile::new (Rc::clone (&lists), 0, 0, None);
        let (modifier_0, modifier_1, _): (Box<Modifier>, Box<Modifier>, _) = generate_modifiers ();
        let (status_2, status_3, status_4): (Status, Status, Status) = generate_statuses ();

        // Test empty modifier
        tile.dec_durations ();
        assert_eq! (tile.modifier, None);
        // Test timed modifier
        tile.add_appliable (modifier_0);
        tile.dec_durations ();
        assert! (matches! (tile.modifier, Some { .. }));
        tile.dec_durations ();
        assert! (matches! (tile.modifier, Some { .. }));
        tile.dec_durations ();
        assert_eq! (tile.modifier, None);
        // Test permanent modifier
        tile.add_appliable (modifier_1);
        tile.dec_durations ();
        assert! (matches! (tile.modifier, Some { .. }));
        tile.dec_durations ();
        assert! (matches! (tile.modifier, Some { .. }));

        // Test empty status
        tile.dec_durations ();
        assert! (matches! (tile.status, None));
        // Test timed status
        tile.add_status (status_2);
        tile.dec_durations ();
        assert! (matches! (tile.status, Some { .. }));
        tile.dec_durations ();
        assert! (matches! (tile.status, Some { .. }));
        tile.dec_durations ();
        assert! (matches! (tile.status, None));
        // Test permanent status
        tile.add_status (status_3);
        tile.dec_durations ();
        assert! (matches! (tile.status, Some { .. }));
        tile.dec_durations ();
        assert! (matches! (tile.status, Some { .. }));
        // Test linked status
        tile.add_status (status_4);
        tile.dec_durations ();
        assert! (matches! (tile.status, Some { .. }));
        assert! (matches! (tile.status.unwrap ().get_next_id ().unwrap (), 3));
        tile.dec_durations ();
        assert! (matches! (tile.status, Some { .. }));
        assert! (matches! (tile.status.unwrap ().get_next_id ().unwrap (), 3));
        tile.dec_durations ();
        assert! (matches! (tile.status, Some { .. }));
        assert_eq! (tile.status.unwrap ().get_next_id (), None);
    }
}
