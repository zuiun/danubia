use std::cell::Cell;
use std::fmt;
use std::rc::Rc;
use crate::Lists;
use crate::common::{Target, Timed, ID};
use crate::dynamic::{Adjustment, Appliable, Applier, Change, Changeable, Modifier, StatisticType, Status, Trigger};
use super::{COST_IMPASSABLE, COST_MINIMUM};

const CLIMB_MAX: u8 = 2;

#[derive (Debug)]
pub struct Tile {
    lists: Rc<Lists>,
    modifier: Cell<Option<Modifier>>,
    status: Cell<Option<Status>>,
    terrain_id: ID,
    height: u8,
    city_id: Option<ID>,
}

impl Tile {
    pub fn new (lists: Rc<Lists>, terrain_id: ID, height: u8, city_id: Option<ID>) -> Self {
        let modifier: Option<Modifier> = None;
        let modifier: Cell<Option<Modifier>> = Cell::new (modifier);
        let status: Option<Status> = None;
        let status: Cell<Option<Status>> = Cell::new (status);

        Self { lists, modifier, status, terrain_id, height, city_id }
    }

    pub fn get_cost (&self) -> u8 {
        let cost: u8 = self.lists.get_terrain (&self.terrain_id).get_cost ();

        match self.modifier.get () {
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
        self.modifier.get ()
    }

    pub fn get_height (&self) -> u8 {
        self.height
    }

    pub fn get_city_id (&self) -> Option<ID> {
        self.city_id
    }
}

impl Changeable for Tile {
    fn add_appliable (&self, appliable: Box<dyn Appliable>) -> bool {
        if let Change::Modifier ( .. ) = appliable.change () {
            let modifier: Modifier = appliable.modifier ();
            let adjustment: Adjustment = modifier.get_adjustments ()[0]
                    .expect (&format! ("Adjustment not found for modifier {:?}", modifier));

            if let StatisticType::Tile ( .. ) = adjustment.0 {
                self.modifier.replace (Some (modifier));

                true
            } else {
                false
            }
        } else {
            false
        }
    }

    fn add_status (&self, status: Status) -> bool {
        if let Change::Modifier ( .. ) = status.get_change () {
            let appliable: Box<dyn Appliable> = status.try_yield_appliable (Rc::clone (&self.lists))
                    .expect (&format! ("Appliable not found for status {:?}", status));

            if let Target::Map = status.get_target () {
                match status.get_trigger () {
                    Trigger::OnOccupy => { self.modifier.replace (None); }
                    Trigger::None => { self.add_appliable (appliable); }
                    _ => return false,
                }

                self.status.replace (Some (status));

                true
            } else {
                false
            }
        } else {
            false
        }
    }

    fn remove_modifier (&self, modifier_id: &ID) -> bool {
        let modifier: Option<Modifier> = self.modifier.get ();

        if let Some (m) = modifier {
            if m.get_id () == *modifier_id {
                self.modifier.replace (None);
    
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    fn remove_status (&self, status_id: &ID) -> bool {
        let status: Option<Status> = self.status.get ();

        if let Some (s) = status {
            if s.get_id () == *status_id {
                if let Change::Modifier (m, _) = s.get_change () {
                    self.remove_modifier (&m);
                }

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
        if let Some (mut m) = self.modifier.get () {
            let modifier: Option<Modifier> = if m.dec_duration () {
                None
            } else {
                Some (m)
            };

            self.modifier.replace (modifier);
        }

        if let Some (mut s) = self.status.get () {
            let status: Option<Status> = if s.dec_duration () {
                s.get_next_id ().and_then (|n: ID|
                    Some (self.lists.get_status (&n).clone ())
                )
            } else {
                Some (s)
            };

            self.status.replace (status);
        }
    }
}

impl Applier for Tile {
    fn try_yield_appliable (&self, lists: Rc<Lists>) -> Option<Box<dyn Appliable>> {
        self.status.get ().and_then (|s: Status| s.try_yield_appliable (lists))
    }

    fn get_target (&self) -> Target {
        Target::Map
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
    use crate::dynamic::ModifierBuilder;
    use crate::tests::generate_lists;

    fn generate_modifiers () -> (Box<Modifier>, Box<Modifier>, Box<Modifier>) {
        let lists = generate_lists ();
        let modifier_builder_0: &ModifierBuilder = lists.get_modifier_builder (&0);
        let modifier_0 = modifier_builder_0.build (false);
        let modifier_0 = Box::new (modifier_0);
        let modifier_builder_1: &ModifierBuilder = lists.get_modifier_builder (&1);
        let modifier_1 = modifier_builder_1.build (false);
        let modifier_1 = Box::new (modifier_1);
        let modifier_builder_2: &ModifierBuilder = lists.get_modifier_builder (&2);
        let modifier_2 = modifier_builder_2.build (false);
        let modifier_2 = Box::new (modifier_2);

        (modifier_0, modifier_1, modifier_2)
    }

    fn generate_statuses () -> (Status, Status, Status) {
        let lists = generate_lists ();
        let status_2 = lists.get_status (&2).clone ();
        let status_3 = lists.get_status (&3).clone ();
        let status_4 = lists.get_status (&4).clone ();

        (status_2, status_3, status_4)
    }

    #[test]
    fn tile_get_cost () {
        let lists = generate_lists ();
        let tile_0 = Tile::new (Rc::clone (&lists), 0, 0, None);
        let tile_1 = Tile::new (Rc::clone (&lists), 1, 0, None);
        let tile_2 = Tile::new (Rc::clone (&lists), 2, 0, None);

        assert_eq! (tile_0.get_cost (), 1);
        assert_eq! (tile_1.get_cost (), 2);
        assert_eq! (tile_2.get_cost (), 0);
    }

    #[test]
    fn tile_is_impassable () {
        let lists = generate_lists ();
        let tile_0 = Tile::new (Rc::clone (&lists), 0, 0, None);
        let tile_2 = Tile::new (Rc::clone (&lists), 2, 0, None);

        // Test passable tile
        assert! (!tile_0.is_impassable ());
        // Test impassable tile
        assert! (tile_2.is_impassable ());
    }

    #[test]
    fn tile_try_climb () {
        let lists = generate_lists ();
        let tile_0 = Tile::new (Rc::clone (&lists), 0, 0, None);
        let tile_1_0 = Tile::new (Rc::clone (&lists), 1, 0, None);
        let tile_1_1 = Tile::new (Rc::clone (&lists), 1, 1, None);
        let tile_1_2 = Tile::new (Rc::clone (&lists), 1, 2, None);

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
        let lists = generate_lists ();
        let tile_0 = Tile::new (Rc::clone (&lists), 0, 0, None);
        let tile_1_0 = Tile::new (Rc::clone (&lists), 1, 0, None);
        let tile_1_1 = Tile::new (Rc::clone (&lists), 1, 1, None);
        let tile_2 = Tile::new (Rc::clone (&lists), 2, 0, None);

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
        let lists = generate_lists ();
        let mut tile = Tile::new (Rc::clone (&lists), 0, 0, None);
        let (modifier_0, modifier_1, modifier_2) = generate_modifiers ();

        // Test additive modifier
        assert_eq! (tile.add_appliable (modifier_0), true);
        assert! (matches! (tile.modifier.get (), Some { .. }));
        assert_eq! (tile.get_cost (), 2);
        // Test subtractive modifier
        assert_eq! (tile.add_appliable (modifier_1), true);
        assert! (matches! (tile.modifier.get (), Some { .. }));
        assert_eq! (tile.get_cost (), 1);
        tile.terrain_id = 1;
        assert_eq! (tile.get_cost (), 1);
        // Test constant modifier
        assert_eq! (tile.add_appliable (modifier_2), true);
        assert! (matches! (tile.modifier.get (), Some { .. }));
        assert_eq! (tile.get_cost (), 1);
    }

    
    #[test]
    fn tile_add_status () {
        let lists = generate_lists ();
        let mut tile = Tile::new (Rc::clone (&lists), 0, 0, None);
        let (status_2, status_3, _) = generate_statuses ();

        // Test tile status
        assert_eq! (tile.add_status (status_3), true);
        assert! (matches! (tile.status.get (), Some { .. }));
        assert_eq! (tile.get_cost (), 1);
        tile.terrain_id = 1;
        assert_eq! (tile.get_cost (), 1);
        // Test applier status
        assert_eq! (tile.add_status (status_2), true);
        assert! (matches! (tile.status.get (), Some { .. }));
        assert_eq! (tile.get_cost (), 2);
        assert! (matches! (tile.try_yield_appliable (Rc::clone (&lists)), Some { .. }));
    }

    #[test]
    fn tile_remove_modifier () {
        let lists = generate_lists ();
        let tile = Tile::new (Rc::clone (&lists), 0, 0, None);
        let (modifier_0, _, _) = generate_modifiers ();

        // Test empty remove
        assert_eq! (tile.remove_modifier (&0), false);
        assert_eq! (tile.modifier.get (), None);
        // Test non-empty remove
        tile.add_appliable (modifier_0);
        assert_eq! (tile.remove_modifier (&0), true);
        assert_eq! (tile.modifier.get (), None);
    }

    #[test]
    fn tile_remove_status () {
        let lists = generate_lists ();
        let tile = Tile::new (Rc::clone (&lists), 0, 0, None);
        let (status_2, status_3, _) = generate_statuses ();

        // Test empty remove
        assert_eq! (tile.remove_status (&0), false);
        assert_eq! (tile.status.get (), None);
        assert_eq! (tile.modifier.get (), None);
        // Test non-empty remove
        tile.add_status (status_3);
        assert_eq! (tile.remove_status (&3), true);
        assert_eq! (tile.status.get (), None);
        assert_eq! (tile.modifier.get (), None);
        // Test applier remove
        tile.add_status (status_2);
        assert_eq! (tile.remove_status (&2), true);
        assert_eq! (tile.status.get (), None);
        assert_eq! (tile.modifier.get (), None);
    }

    #[test]
    fn tile_dec_durations () {
        let lists = generate_lists ();
        let tile = Tile::new (Rc::clone (&lists), 0, 0, None);
        let (modifier_0, modifier_1, _) = generate_modifiers ();
        let (status_2, status_3, status_4) = generate_statuses ();

        // Test empty modifier
        tile.dec_durations ();
        assert_eq! (tile.modifier.get (), None);
        // Test timed modifier
        tile.add_appliable (modifier_0);
        tile.dec_durations ();
        assert! (matches! (tile.modifier.get (), Some { .. }));
        tile.dec_durations ();
        assert! (matches! (tile.modifier.get (), Some { .. }));
        tile.dec_durations ();
        assert_eq! (tile.modifier.get (), None);
        // Test permanent modifier
        tile.add_appliable (modifier_1);
        tile.dec_durations ();
        assert! (matches! (tile.modifier.get (), Some { .. }));
        tile.dec_durations ();
        assert! (matches! (tile.modifier.get (), Some { .. }));

        // Test empty status
        tile.dec_durations ();
        assert! (matches! (tile.status.get (), None));
        // Test timed status
        tile.add_status (status_2);
        tile.dec_durations ();
        assert! (matches! (tile.status.get (), Some { .. }));
        tile.dec_durations ();
        assert! (matches! (tile.status.get (), Some { .. }));
        tile.dec_durations ();
        assert! (matches! (tile.status.get (), None));
        // Test permanent status
        tile.add_status (status_3);
        tile.dec_durations ();
        assert! (matches! (tile.status.get (), Some { .. }));
        tile.dec_durations ();
        assert! (matches! (tile.status.get (), Some { .. }));
        // Test linked status
        tile.add_status (status_4);
        tile.dec_durations ();
        assert! (matches! (tile.status.get (), Some { .. }));
        assert! (matches! (tile.status.get ().unwrap ().get_next_id ().unwrap (), 3));
        tile.dec_durations ();
        assert! (matches! (tile.status.get (), Some { .. }));
        assert! (matches! (tile.status.get ().unwrap ().get_next_id ().unwrap (), 3));
        tile.dec_durations ();
        assert! (matches! (tile.status.get (), Some { .. }));
        assert_eq! (tile.status.get ().unwrap ().get_next_id (), None);
    }
}
