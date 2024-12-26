use std::{fmt, rc::Rc};
use crate::engine::Lists;
use crate::engine::common::{Adjustment, ID, Modifiable, Modifier, Statistic, Timed};
use super::{COST_IMPASSABLE, COST_MINIMUM};

const CLIMB_MAX: u8 = 2;

#[derive (Debug)]
pub struct Tile {
    lists: Rc<Lists>,
    modifier: Option<Modifier>,
    terrain_id: ID,
    height: u8,
    city_id: Option<ID>
}

impl Tile {
    pub fn new (lists: Rc<Lists>, terrain_id: ID, height: u8, city_id: Option<ID>) -> Self {
        let lists: Rc<Lists> = Rc::clone (&lists);
        let modifier: Option<Modifier> = None;

        Self { lists, modifier, terrain_id, height, city_id }
    }

    pub fn get_cost (&self) -> u8 {
        let cost: u8 = self.lists.get_terrain (&self.terrain_id).get_cost ();

        match self.modifier {
            Some (m) => {
                let adjustment: Adjustment = m.get_adjustments ()[0].expect (&format! ("Adjustment not found for modifier {:?}", m));

                match adjustment.0 {
                    Statistic::Tile => if m.can_stack () {
                        if adjustment.2 {
                            cost + (adjustment.1 as u8)
                        } else {
                            u8::max (cost.checked_sub (adjustment.1 as u8).unwrap_or (COST_MINIMUM), COST_MINIMUM)
                        }
                    } else {
                        adjustment.1 as u8
                    }
                    _ => panic! ("Invalid statistic {:?}", adjustment.0)
                }
            }
            None => cost
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

impl Modifiable for Tile {
    fn add_modifier (&mut self, modifier: Modifier) -> bool {
        let adjustment: Adjustment = modifier.get_adjustments ()[0].expect (&format! ("Adjustment not found for modifier {:?}", modifier));

        if modifier.get_duration () > 0 {
            assert! (matches! (adjustment.0, Statistic::Tile));

            self.modifier = Some (modifier);

            true
        } else {
            false
        }
    }

    fn remove_modifier (&mut self, _modifier_id: &ID) -> bool {
        match self.modifier {
            Some (_) => {
                self.modifier = None;

                true
            }
            None => false
        }
    }

    fn dec_durations (&mut self) -> () {
        if let Some (ref mut m) = self.modifier {
            if m.dec_duration () {
                self.modifier = None;
            }
        }
    }
}

impl fmt::Display for Tile {
    fn fmt (&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write! (f, "{}.{}", self.terrain_id, self.height)
    }
}

pub struct TileBuilder {
    lists: Rc<Lists>
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
    use crate::engine::tests::generate_lists;
    use crate::engine::common::ID_UNINITIALISED;

    fn generate_modifiers () -> (Modifier, Modifier, Modifier) {
        let lists: Rc<Lists> = generate_lists ();
        let modifier_0: Modifier = lists.get_modifier (&0).clone ();
        let modifier_1: Modifier = lists.get_modifier (&1).clone ();
        let modifier_2: Modifier = lists.get_modifier (&2).clone ();

        (modifier_0, modifier_1, modifier_2)
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
    fn tile_add_modifier () {
        let lists: Rc<Lists> = generate_lists ();
        let mut tile: Tile = Tile::new (Rc::clone (&lists), 0, 0, None);
        let (modifier_0, modifier_1, modifier_2): (Modifier, Modifier, Modifier) = generate_modifiers ();

        // Test additive modifier
        assert_eq! (tile.add_modifier (modifier_0), true);
        assert! (matches! (tile.modifier, Some { .. }));
        assert_eq! (tile.get_cost (), 2);
        // Test subtractive modifier
        assert_eq! (tile.add_modifier (modifier_1), true);
        assert! (matches! (tile.modifier, Some { .. }));
        assert_eq! (tile.get_cost (), 1);
        tile.terrain_id = 1;
        assert_eq! (tile.get_cost (), 1);
        // Test constant modifier
        assert_eq! (tile.add_modifier (modifier_2), true);
        assert! (matches! (tile.modifier, Some { .. }));
        assert_eq! (tile.get_cost (), 1);
    }

    #[test]
    fn tile_remove_modifier () {
        let lists: Rc<Lists> = generate_lists ();
        let mut tile: Tile = Tile::new (Rc::clone (&lists), 0, 0, None);
        let (modifier_0, _, _): (Modifier, _, _) = generate_modifiers ();

        // Test empty remove
        assert_eq! (tile.remove_modifier (&ID_UNINITIALISED), false);
        assert! (matches! (tile.modifier, None));
        assert_eq! (tile.get_cost (), 1);
        // Test non-empty remove
        tile.add_modifier (modifier_0);
        assert_eq! (tile.remove_modifier (&ID_UNINITIALISED), true);
        assert! (matches! (tile.modifier, None));
        assert_eq! (tile.get_cost (), 1);
        assert_eq! (tile.remove_modifier (&ID_UNINITIALISED), false);
        assert! (matches! (tile.modifier, None));
        assert_eq! (tile.get_cost (), 1);
    }

    #[test]
    fn tile_dec_durations () {
        let lists: Rc<Lists> = generate_lists ();
        let mut tile: Tile = Tile::new (Rc::clone (&lists), 0, 0, None);
        let (modifier_0, modifier_1, _): (Modifier, Modifier, _) = generate_modifiers ();

        // Test empty modifier
        tile.dec_durations ();
        assert! (matches! (tile.modifier, None));
        // Test timed modifier
        tile.add_modifier (modifier_0);
        tile.dec_durations ();
        assert! (matches! (tile.modifier, Some { .. }));
        tile.dec_durations ();
        assert! (matches! (tile.modifier, None));
        // Test permanent modifier
        tile.add_modifier (modifier_1);
        tile.dec_durations ();
        assert! (matches! (tile.modifier, Some { .. }));
        tile.dec_durations ();
        assert! (matches! (tile.modifier, Some { .. }));
    }
}
