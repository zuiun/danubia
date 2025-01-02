use super::*;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::{Rc, Weak};
use crate::{event::Handler, Lists};
use crate::common::{Capacity, ID, ID_UNINITIALISED, Target, Timed};
use crate::event::{Message, Observer, Response, Subject};
use crate::map::{Area, Direction, Location, Search};
use crate::dynamic::{Appliable, Applier, Change, Changeable, Effect, Modifier, ModifierBuilder, StatisticType, Status, Trigger};

const PERCENT_1: u16 = 1_0;
const PERCENT_100: u16 = 100_0;
const MRL_MAX: u16 = PERCENT_100; // 100.0%
const HLT_MAX: u16 = 1000; // 1000
const SPL_MAX: u16 = PERCENT_100; // 100.0%
const ATK_MAX: u16 = 200; // 200
const DEF_MAX: u16 = 200; // 200
const MAG_MAX: u16 = 200; // 200
const MOV_MAX: u16 = 100; // 100
const ORG_MAX: u16 = 2 * PERCENT_100; // 200.0%
const DRAIN_SPL: f32 = 50.0; // 5.0%
const RECOVER_MRL: u16 = 1_0; // 1.0%
const DRAIN_HLT: u16 = 4; // 4
const THRESHOLD_RETREAT_MRL: u16 = 40_0; // 40.0%
const THRESHOLD_ROUT_MRL: u16 = 20_0; // 20.0%
const THRESHOLD_SKILL_PASSIVE: usize = 1; // TODO: probably needs to be balanced
const WEAPON_0: usize = 0;
const WEAPON_1: usize = 1;
const WEAPONS_LENGTH: usize = 2;
const SKILL_0: usize = 0;
const SKILL_1: usize = 1;
const SKILL_2: usize = 2;
const SKILLS_LENGTH: usize = 3;
/*
 * Calculated from build.rs
 * Unit MOV is an index into the table
 * Attack (* 1.0): 21 delay at 0, 20 delay at 1, 2 delay at 77, and 1 delay at 100
 * Skill/Magic (* 1.4): 29 delay at 0, 28 delay at 1, 2 delay at 77, and 1 delay at 100
 * Wait (* 0.67): 14 delay at 0, 13 delay at 1, 2 delay at 54, and 1 delay at 77
 */
const DELAYS: [u8; 101] = [21, 20, 19, 19, 18, 18, 17, 17, 16, 16, 15, 15, 14, 14, 14, 13, 13, 13, 12, 12, 11, 11, 11, 11, 10, 10, 10, 9, 9, 9, 9, 8, 8, 8, 8, 7, 7, 7, 7, 7, 7, 6, 6, 6, 6, 6, 6, 5, 5, 5, 5, 5, 5, 5, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 1];
const DELAY_ATTACK: f32 = 1.0;
const DELAY_SKILL: f32 = 1.4;
const DELAY_MAGIC: f32 = 1.4;
const DELAY_WAIT: f32 = 0.67;

fn get_delay (mov: u16, delay_multiplier: f32) -> u8 {
    assert! ((mov as usize) < DELAYS.len ());

    let delay: f32 = DELAYS[mov as usize] as f32;

    (delay * delay_multiplier) as u8
}

#[derive (Debug)]
#[derive (Clone, Copy)]
pub enum UnitStatistic {
    MRL, // morale - willingness to fight (permillage)
    HLT, // manpower - number of soldiers
    SPL, // supply - proportion of soldiers equipped (permillage)
    ATK, // attack – physical damage
    DEF, // defence – physical resistance
    MAG, // magic – magical damage and resistance
    MOV, // manoeuvre – speed and movement
    ORG, // cohesion – modifier for physical damage and resistance (permillage)
    Length,
}

#[derive (Debug)]
#[derive (Clone, Copy)]
pub enum Rank {
    Leader,
    Follower (ID), // leader
}

#[derive (Debug)]
#[derive (Clone, Copy)]
pub struct UnitStatistics ([Capacity; UnitStatistic::Length as usize]);

impl UnitStatistics {
    pub const fn new (mrl: u16, hlt: u16, spl: u16, atk: u16, def: u16, mag: u16, mov: u16, org: u16) -> Self {
        assert! (mrl <= MRL_MAX);
        assert! (hlt <= HLT_MAX);
        assert! (spl <= SPL_MAX);
        assert! (atk <= ATK_MAX);
        assert! (def <= DEF_MAX);
        assert! (mag <= MAG_MAX);
        assert! (mov <= MOV_MAX);
        assert! (org <= ORG_MAX);

        let mrl: Capacity = Capacity::Quantity (mrl, MRL_MAX);
        let hlt: Capacity = Capacity::Quantity (hlt, HLT_MAX);
        let spl: Capacity = Capacity::Quantity (spl, SPL_MAX);
        let atk: Capacity = Capacity::Constant (atk, ATK_MAX, atk);
        let def: Capacity = Capacity::Constant (def, DEF_MAX, def);
        let mag: Capacity = Capacity::Constant (mag, MAG_MAX, mag);
        let mov: Capacity = Capacity::Constant (mov, MOV_MAX, mov);
        let org: Capacity = Capacity::Quantity (org, ORG_MAX);
        let statistics: [Capacity; UnitStatistic::Length as usize] = [mrl, hlt, spl, atk, def, mag, mov, org];

        Self (statistics)
    }

    fn validate_statistic (&self, statistic: UnitStatistic) -> bool {
        match statistic {
            UnitStatistic::MRL => if let Capacity::Quantity ( .. ) = self.0[UnitStatistic::MRL as usize] {
                true
            } else {
                false
            }
            UnitStatistic::HLT => if let Capacity::Quantity ( .. ) = self.0[UnitStatistic::HLT as usize] {
                true
            } else {
                false
            }
            UnitStatistic::SPL => if let Capacity::Quantity ( .. ) = self.0[UnitStatistic::SPL as usize] {
                true
            } else {
                false
            }
            UnitStatistic::ATK => if let Capacity::Constant ( .. ) = self.0[UnitStatistic::ATK as usize] {
                true
            } else {
                false
            }
            UnitStatistic::DEF => if let Capacity::Constant ( .. ) = self.0[UnitStatistic::DEF as usize] {
                true
            } else {
                false
            }
            UnitStatistic::MAG => if let Capacity::Constant ( .. ) = self.0[UnitStatistic::MAG as usize] {
                true
            } else {
                false
            }
            UnitStatistic::MOV => if let Capacity::Constant ( .. ) = self.0[UnitStatistic::MOV as usize] {
                true
            } else {
                false
            }
            UnitStatistic::ORG => if let Capacity::Quantity ( .. ) = self.0[UnitStatistic::ORG as usize] {
                true
            } else {
                false
            }
            _ => panic! ("Statistic not found"),
        }
    }

    fn get_statistic (&self, statistic: UnitStatistic) -> (u16, u16) {
        assert! (self.validate_statistic (statistic));

        match self.0[statistic as usize] {
            Capacity::Constant (c, _, b) => {
                (c, b)
            }
            Capacity::Quantity (c, m) => {
                (c, m)
            }
        }
    }

    fn set_statistic (&mut self, statistic: UnitStatistic, value: u16) -> () {
        assert! (self.validate_statistic (statistic));

        self.0[statistic as usize] = match self.0[statistic as usize] {
            Capacity::Constant (_, m, b) => {
                assert! (value <= m);

                Capacity::Constant (value, m, b)
            }
            Capacity::Quantity (_, m) => {
                assert! (value <= m);

                Capacity::Quantity (value, m)
            }
        };
    }

    fn change_statistic_flat (&mut self, statistic: UnitStatistic, change: u16, is_add: bool) -> () {
        assert! (self.validate_statistic (statistic));

        let (current, maximum): (u16, u16) = match self.0[statistic as usize] {
            Capacity::Constant (c, m, _) => {
                (c, m)
            }
            Capacity::Quantity (c, m) => {
                (c, m)
            }
        };
        let value: u16 = if is_add {
            u16::min (current + change, maximum)
        } else {
            current.checked_sub (change).unwrap_or (0)
        };

        self.set_statistic (statistic, value);
    }

    fn change_statistic_percentage (&mut self, statistic: UnitStatistic, change: u16, is_add: bool) -> () {
        assert! (self.validate_statistic (statistic));

        let base: f32 = match self.0[statistic as usize] {
            Capacity::Constant (_, _, b) => b,
            Capacity::Quantity (_, m) => m,
        } as f32;
        let change: f32 = (change as f32) / 100.0;
        let change: u16 = (base * change) as u16;

        self.change_statistic_flat (statistic, change, is_add);
    }
}

#[derive (Debug)]
pub struct Unit {
    id: ID,
    lists: Rc<Lists>,
    statistics: Cell<UnitStatistics>,
    modifier_terrain: Cell<Modifier>,
    // Safety guarantee: Only Unit can reference its own modifiers
    modifiers: RefCell<Vec<Modifier>>,
    // Safety guarantee: Only Unit can reference its own statuses
    statuses: RefCell<HashMap<Trigger, Vec<Status>>>,
    weapons: [Weapon; WEAPONS_LENGTH],
    skill_passive_id: Cell<ID>,
    skills: [Skill; SKILLS_LENGTH],
    magic_ids: Vec<ID>,
    weapon_active: usize,
    faction_id: ID,
    rank: Cell<Rank>,
    // Safety guarantee: Unit will never borrow_mut Handler
    handler: Weak<RefCell<Handler>>,
    observer_id: Cell<ID>,
}

impl Unit {
    pub fn new (id: ID, lists: Rc<Lists>, statistics: UnitStatistics, weapon_ids: [ID; WEAPONS_LENGTH], skill_passive_id: ID, skill_ids: [ID; SKILLS_LENGTH], magics_usable: [bool; 3], faction_id: ID, rank: Rank, handler: Weak<RefCell<Handler>>) -> Self {
        let statistics: Cell<UnitStatistics> = Cell::new (statistics);
        let modifier_terrain: Modifier = Modifier::default ();
        let modifier_terrain: Cell<Modifier> = Cell::new (modifier_terrain);
        let modifiers: Vec<Modifier> = Vec::new ();
        let modifiers: RefCell<Vec<Modifier>> = RefCell::new (modifiers);
        let statuses: HashMap<Trigger, Vec<Status>> = HashMap::new ();
        let statuses: RefCell<HashMap<Trigger, Vec<Status>>> = RefCell::new (statuses);
        let magic_ids: Vec<ID> = Vec::new ();
        let skill_passive_id: Cell<ID> = Cell::new (skill_passive_id);
        let skills: [Skill; SKILLS_LENGTH] = [
            lists.get_skill (&skill_ids[SKILL_0]).clone (),
            lists.get_skill (&skill_ids[SKILL_1]).clone (),
            lists.get_skill (&skill_ids[SKILL_2]).clone (),
        ];
        let weapons: [Weapon; WEAPONS_LENGTH] = [
            lists.get_weapon (&weapon_ids[WEAPON_0]).clone (),
            lists.get_weapon (&weapon_ids[WEAPON_1]).clone (),
        ];
        let weapon_active: usize = WEAPON_0;
        let rank: Cell<Rank> = Cell::new (rank);
        let observer_id: Cell<ID> = Cell::new (ID_UNINITIALISED);

        statuses.borrow_mut ().insert (Trigger::None, Vec::new ());

        Self { id, lists, statistics, modifier_terrain, modifiers, statuses, magic_ids, skill_passive_id, skills, weapons, weapon_active, faction_id, rank, handler, observer_id }
    }

    pub fn initialise (&self) -> () {
        // self.apply_inactive_skills ();
    }

    fn get_statistic (&self, statistic: UnitStatistic) -> (u16, u16) {
        self.statistics.get ().get_statistic (statistic)
    }

    fn change_statistic_flat (&self, statistic: UnitStatistic, change: u16, is_add: bool) -> () {
        let mut statistics: UnitStatistics = self.statistics.get ();

        statistics.change_statistic_flat (statistic, change, is_add);
        self.statistics.replace (statistics);

        if self.get_statistic (UnitStatistic::HLT).0 == 0 {
            self.die ();
        }
    }

    fn change_statistic_percentage (&self, statistic: UnitStatistic, change: u16, is_add: bool) -> () {
        let mut statistics: UnitStatistics = self.statistics.get ();

        statistics.change_statistic_percentage (statistic, change, is_add);
        self.statistics.replace (statistics);

        if self.get_statistic (UnitStatistic::HLT).0 == 0 {
            self.die ();
        }
    }

    fn die (&self) -> () {
        self.notify (Message::GameUnitDie (self.id));

        // TODO: deinitialise
    }

    fn filter_unit_allegiance (&self, unit_ids: &Vec<ID>, is_ally: bool) -> Vec<ID> {
        if is_ally {
            unit_ids.iter ().filter_map (|u: &ID| {
                let is_member: Vec<Response> = self.notify (Message::FactionIsMember (self.faction_id, *u));
                if let Response::FactionIsMember (m) = Handler::reduce_responses (&is_member) {
                    if *m {
                        Some (*u)
                    } else {
                        None
                    }
                } else {
                    panic! ("Invalid response")
                }
            }).collect::<Vec<ID>> ()
        } else {
            unit_ids.iter ().filter_map (|u: &ID| {
                let is_member: Vec<Response> = self.notify (Message::FactionIsMember (self.faction_id, *u));

                if let Response::FactionIsMember (m) = Handler::reduce_responses (&is_member) {
                    if !(*m) {
                        Some (*u)
                    } else {
                        None
                    }
                } else {
                    panic! ("Invalid response")
                }
            }).collect::<Vec<ID>> ()
        }
    }

    fn choose_targets_units (&self, potential_ids: &Vec<ID>, target: Target, area: Area, range: u8) -> Vec<ID> {
        assert! (potential_ids.len () > 0);

        let mut target_id: ID = potential_ids[0];
        let target_ids: Vec<ID> = match target {
            Target::This => {
                // TODO: Prompt user for confirmation
                if true {
                    vec![target_id]
                // TODO: User rejected choice
                } else {
                    Vec::new ()
                }
            }
            Target::Ally | Target::Enemy => {
                loop {
                    // TODO: Prompt user to choose ONE
                    // target_id = ???;

                    // TODO: Prompt user for confirmation
                    if true {
                        break vec![target_id]
                    // TODO: User rejected choice
                    } else if 1 > 0 {
                        break Vec::new ()
                    }
                    // TODO: else -> user made another choice
                }
            }
            Target::Allies | Target::Enemies => {
                match area {
                    Area::Single => potential_ids.clone (),
                    Area::Radial (r) => loop {
                        // TODO:: Prompt user to choose ONE centre
                        // target_id = ???;

                        let target_ids: Vec<Response> = self.notify (Message::GridFindUnits (target_id, Search::Radial (r)));
                        let target_ids: Vec<ID> = if let Response::GridFindUnits (t) = Handler::reduce_responses (&target_ids) {
                            if let Target::Allies = target {
                                self.filter_unit_allegiance (t, true)
                            } else if let Target::Enemies = target {
                                self.filter_unit_allegiance (t, false)
                            } else {
                                panic! ("Invalid target {:?}", target)
                            }
                        } else {
                            panic! ("Invalid response")
                        };

                        // TODO: Prompt user for confirmation
                        if true {
                            break target_ids.clone ()
                        // TODO: User rejected choice
                        } else if 1 > 0 {
                            break Vec::new ()
                        }
                        // TODO: else -> user made another choice
                    }
                    Area::Path (w) => loop {
                        // TODO:: Prompt user to choose ONE direction
                        // direction = ???
                        let direction = Direction::Right;
                        let target_ids: Vec<Response> = self.notify (Message::GridFindUnits (self.id, Search::Path (w, range, direction)));
                        let target_ids: Vec<ID> = if let Response::GridFindUnits (t) = Handler::reduce_responses (&target_ids) {
                            if let Target::Allies = target {
                                self.filter_unit_allegiance (t, true)
                            } else if let Target::Enemies = target {
                                self.filter_unit_allegiance (t, false)
                            } else {
                                panic! ("Invalid target {:?}", target)
                            }
                        } else {
                            panic! ("Invalid response")
                        };

                        // TODO: Prompt user for confirmation
                        if true {
                            break target_ids.clone ()
                        // TODO: User rejected choice
                        } else if 1 > 0 {
                            break Vec::new ()
                        }
                        // TODO: else -> user made another choice
                    }
                }
            }
            _ => panic! ("Invalid target {:?}", target),
        };

        // TODO: Prompt user to confirm
        target_ids
    }

    fn find_targets_units (&self, target: Target, area: Area, range: u8) -> Vec<ID> {
        let potential_ids: Vec<ID> = if let Target::Map = target {
            panic! ("Invalid target {:?}", target)
        } else if let Target::This = target {
            vec![self.id]
        } else {
            let neighbour_ids: Vec<ID> = if let Area::Path (w) = area {
                let neighbour_ids_up: Vec<Response> = self.notify (Message::GridFindUnits (self.id, Search::Path (w, range, Direction::Up)));
                let neighbour_ids_up: &Vec<ID> = if let Response::GridFindUnits (n) = Handler::reduce_responses (&neighbour_ids_up) {
                    n
                } else {
                    panic! ("Invalid response")
                };
                let neighbour_ids_right: Vec<Response> = self.notify (Message::GridFindUnits (self.id, Search::Path (w, range, Direction::Right)));
                let neighbour_ids_right: &Vec<ID> = if let Response::GridFindUnits (n) = Handler::reduce_responses (&neighbour_ids_right) {
                    n
                } else {
                    panic! ("Invalid response")
                };
                let neighbour_ids_left: Vec<Response> = self.notify (Message::GridFindUnits (self.id, Search::Path (w, range, Direction::Left)));
                let neighbour_ids_left: &Vec<ID> = if let Response::GridFindUnits (n) = Handler::reduce_responses (&neighbour_ids_left) {
                    n
                } else {
                    panic! ("Invalid response")
                };
                let neighbour_ids_down: Vec<Response> = self.notify (Message::GridFindUnits (self.id, Search::Path (w, range, Direction::Down)));
                let neighbour_ids_down: &Vec<ID> = if let Response::GridFindUnits (n) = Handler::reduce_responses (&neighbour_ids_down) {
                    n
                } else {
                    panic! ("Invalid response")
                };
                let mut neighbour_ids: Vec<ID> = Vec::new ();

                neighbour_ids.extend (neighbour_ids_up.iter ());
                neighbour_ids.extend (neighbour_ids_right.iter ());
                neighbour_ids.extend (neighbour_ids_left.iter ());
                neighbour_ids.extend (neighbour_ids_down.iter ());

                neighbour_ids
            } else {
                let neighbour_ids: Vec<Response> = self.notify (Message::GridFindUnits (self.id, Search::Radial (range)));

                if let Response::GridFindUnits (n) = Handler::reduce_responses (&neighbour_ids) {
                    n.clone ()
                } else {
                    panic! ("Invalid response")
                }
            };

            match target {
                Target::Ally | Target::Allies => self.filter_unit_allegiance (&neighbour_ids, true),
                Target::Enemy | Target::Enemies => self.filter_unit_allegiance (&neighbour_ids, false),
                _ => panic! ("Invalid target {:?}", target),
            }
        };

        if potential_ids.len () > 0 {
            self.choose_targets_units (&potential_ids, target, area, range)
        } else {
            // TODO: if there are no potential targets, then just give up but wait for the user to cancel
            Vec::new ()
        }
    }

    fn choose_targets_locations (&self, potential_locations: &Vec<Location>, area: Area, range: u8) -> Vec<Location> {
        assert! (potential_locations.len () > 0);

        let mut target_location: Location = potential_locations[0];

        match area {
            Area::Single => potential_locations.clone (),
            Area::Radial (r) => loop {
                // TODO:: Prompt user to choose ONE centre
                // target_location = ???;

                let target_locations: Vec<Response> = self.notify (Message::GridFindLocations (target_location, Search::Radial (r)));
                let target_locations: &Vec<Location> = if let Response::GridFindLocations (t) = Handler::reduce_responses (&target_locations) {
                    t
                } else {
                    panic! ("Invalid response")
                };

                // TODO: Prompt user for confirmation
                if true {
                    break target_locations.clone ()
                // TODO: User rejected choice
                } else if 1 > 0 {
                    break Vec::new ()
                }
                // TODO: else -> user made another choice
            }
            Area::Path (w) => loop {
                // TODO:: Prompt user to choose ONE direction
                // direction = ???;
                let location: Vec<Response> = self.notify (Message::GridGetUnitLocation (self.id));
                let location: Location = if let Response::GridGetUnitLocation (l) = Handler::reduce_responses (&location) {
                *l
                } else {
                    panic! ("Invalid response")
                };
                let direction = Direction::Right;
                let target_locations: Vec<Response> = self.notify (Message::GridFindLocations (location, Search::Path (w, range, direction)));
                let target_locations: &Vec<Location> = if let Response::GridFindLocations (t) = Handler::reduce_responses (&target_locations) {
                    t
                } else {
                    panic! ("Invalid response")
                };

                // TODO: Prompt user for confirmation
                if true {
                    break target_locations.clone ()
                // TODO: User rejected choice
                } else if 1 > 0 {
                    break Vec::new ()
                }
                // TODO: else -> user made another choice
            }
        }
    }

    fn find_targets_locations (&self, area: Area, range: u8) -> Vec<Location> {
        let location: Vec<Response> = self.notify (Message::GridGetUnitLocation (self.id));
        let location: Location = if let Response::GridGetUnitLocation (l) = Handler::reduce_responses (&location) {
            *l
        } else {
            panic! ("Invalid response")
        };
        let potential_locations: Vec<Location> = if let Area::Path (w) = area {
            let neighbour_locations_up: Vec<Response> = self.notify (Message::GridFindLocations (location, Search::Path (w, range, Direction::Up)));
            let neighbour_locations_up: &Vec<Location> = if let Response::GridFindLocations (n) = Handler::reduce_responses (&neighbour_locations_up) {
                n
            } else {
                panic! ("Invalid response")
            };
            let neighbour_locations_right: Vec<Response> = self.notify (Message::GridFindLocations (location, Search::Path (w, range, Direction::Right)));
            let neighbour_locations_right: &Vec<Location> = if let Response::GridFindLocations (n) = Handler::reduce_responses (&neighbour_locations_right) {
                n
            } else {
                panic! ("Invalid response")
            };
            let neighbour_locations_left: Vec<Response> = self.notify (Message::GridFindLocations (location, Search::Path (w, range, Direction::Left)));
            let neighbour_locations_left: &Vec<Location> = if let Response::GridFindLocations (n) = Handler::reduce_responses (&neighbour_locations_left) {
                n
            } else {
                panic! ("Invalid response")
            };
            let neighbour_locations_down: Vec<Response> = self.notify (Message::GridFindLocations (location, Search::Path (w, range, Direction::Down)));
            let neighbour_locations_down: &Vec<Location> = if let Response::GridFindLocations (n) = Handler::reduce_responses (&neighbour_locations_down) {
                n
            } else {
                panic! ("Invalid response")
            };
            let mut neighbour_locations: Vec<Location> = Vec::new ();

            neighbour_locations.extend (neighbour_locations_up.iter ());
            neighbour_locations.extend (neighbour_locations_right.iter ());
            neighbour_locations.extend (neighbour_locations_left.iter ());
            neighbour_locations.extend (neighbour_locations_down.iter ());

            neighbour_locations
        } else {
            let potential_locations: Vec<Response> = self.notify (Message::GridFindLocations (location, Search::Radial (range)));

            if let Response::GridFindLocations (n) = Handler::reduce_responses (&potential_locations) {
                n.clone ()
            } else {
                panic! ("Invalid response")
            }
        };

        if potential_locations.len () > 0 {
            self.choose_targets_locations (&potential_locations, area, range)
        } else {
            // TODO: if there are no potential targets, then just give up but wait for the user to cancel
            Vec::new ()
        }
    }

    fn calculate_damage (&self, statistics_other: &UnitStatistics) -> (u16, u16, u16) {
        let weapon: &Weapon = &self.weapons[self.weapon_active];
        let dmg_weapon: u16 = weapon.get_statistic (WeaponStatistic::DMG);
        let slh_weapon: u16 = weapon.get_statistic (WeaponStatistic::SLH);
        let prc_weapon: u16 = weapon.get_statistic (WeaponStatistic::PRC);
        let dcy_weapon: u16 = weapon.get_statistic (WeaponStatistic::DCY);

        let hlt_self: (u16, u16) = self.get_statistic (UnitStatistic::HLT);
        let spl_self: (u16, u16) = self.get_statistic (UnitStatistic::SPL);
        let atk_self: u16 = self.get_statistic (UnitStatistic::ATK).0;
        let mag_self: u16 = self.get_statistic (UnitStatistic::MAG).0;
        let org_self: u16 = self.get_statistic (UnitStatistic::ORG).0;

        let mrl_other: (u16, u16) = statistics_other.get_statistic (UnitStatistic::MRL);
        let spl_other: (u16, u16) = statistics_other.get_statistic (UnitStatistic::SPL);
        let def_other: u16 = statistics_other.get_statistic (UnitStatistic::DEF).0;
        let mag_other: u16 = statistics_other.get_statistic (UnitStatistic::MAG).0;
        let org_other: u16 = statistics_other.get_statistic (UnitStatistic::ORG).0;

        let damage_weapon: f32 = {
            let damage: f32 = (atk_self + dmg_weapon) as f32;
            let multiplier: f32 = (spl_self.0 as f32) / (spl_self.1 as f32);

            damage * multiplier
        };
        let damage_bonus: u16 = {
            let damage: u16 = u16::max (mag_self.checked_sub (mag_other).unwrap_or (1), 1);
            let multiplier: u16 = (dcy_weapon * 2) + 1;

            damage * multiplier
        };
        let multiplier: f32 = {
            let multiplier_mrl: f32 = 1.0 - (mrl_other.0 as f32) / (mrl_other.1 as f32);
            let multiplier_hlt: f32 = (hlt_self.0 as f32) / (hlt_self.1 as f32);
            let multiplier_org: f32 = (org_self as f32) / (PERCENT_100 as f32);

            multiplier_mrl + multiplier_hlt + multiplier_org
        };
        let reducer: u16 = {
            let multiplier_spl: f32 = (spl_other.0 as f32) / (spl_other.1 as f32);
            let multiplier_org: f32 = (org_other as f32) / (PERCENT_100 as f32);
            let multiplier: f32 = (multiplier_spl + multiplier_org) / 2.0;

            ((def_other as f32) * multiplier) as u16
        };
        let damage_base: u16 = u16::max ((damage_weapon * multiplier) as u16 - reducer, 1);
        let damage_mrl: u16 = (damage_base * (slh_weapon + 1)) + damage_bonus;
        let damage_hlt: u16 = damage_base + (damage_bonus * (prc_weapon + 1));
        let damage_spl: u16 = damage_base + damage_bonus + slh_weapon + prc_weapon + dcy_weapon;

        (damage_mrl, damage_hlt, damage_spl)
    }

    fn take_damage (&self, damage_mrl: u16, damage_hlt: u16, damage_spl: u16) -> () {
        self.change_statistic_flat (UnitStatistic::MRL, damage_mrl, false);
        self.change_statistic_flat (UnitStatistic::HLT, damage_hlt, false);
        self.change_statistic_flat (UnitStatistic::SPL, damage_spl, false);
    }

    // Should only be called during initialisation
    fn apply_inactive_skills (&self) -> () {
        let skill_passive_id: ID = self.skill_passive_id.get ();
        let skill_passive: &Skill = self.lists.get_skill (&skill_passive_id);
        let status_passive_id: ID = skill_passive.get_status_id ();

        if status_passive_id < ID_UNINITIALISED {
            assert! (skill_passive.is_passive ());

            let status_passive: Status = self.lists.get_status (&status_passive_id).clone ();

            self.add_status (status_passive);
        }


        for skill in self.skills.iter () {
            if skill.is_toggle () {
                let status_id: ID = skill.get_status_id ();
                let status: Status = self.lists.get_status (&status_id).clone ();

                self.add_status (status);
            }
        }
    }

    fn change_modifier_terrain (&self, modifier_id: ID) -> () {
        let modifier: Modifier = if modifier_id < ID_UNINITIALISED {
            let modifier_builder: &ModifierBuilder = self.lists.get_modifier_builder (&modifier_id);
            let modifier: Modifier = modifier_builder.build (false);
            let appliable: Box<dyn Appliable> = Box::new (modifier);

            self.add_appliable (appliable);

            modifier
        } else {
            Modifier::default ()
        };

        self.modifier_terrain.replace (modifier);
    }

    fn set_leader (&self, leader_id: ID) -> () {
        let rank: Rank = self.rank.get ();

        if let Rank::Follower ( .. ) = rank {
            self.rank.replace (Rank::Follower (leader_id));
        }
    }

    fn try_add_passive (&self, status_id: &ID) -> bool {
        if let Rank::Follower (l) = self.rank.get () {
            let distance: Vec<Response> = self.notify (Message::GridFindDistanceBetween (self.id, l));
            let distance: usize = if let Response::GridFindDistanceBetween (d) = Handler::reduce_responses (&distance) {
                *d
            } else {
                panic! ("Invalid response")
            };
            let org: u16 = self.get_statistic (UnitStatistic::ORG).0;
            let multiplier: f32 = ((org / PERCENT_100) as f32) * 2.0;
            let threshold: usize = ((THRESHOLD_SKILL_PASSIVE as f32) * multiplier) as usize;

            if distance > threshold {
                self.remove_status (status_id);

                false
            } else if self.skill_passive_id.get () < ID_UNINITIALISED {
                false
            } else {
                let status: Status = self.lists.get_status (status_id).clone ();

                self.skill_passive_id.replace (*status_id);

                self.add_status (status)
            }
        } else {
            false
        }
    }

    fn send_passive (&self) -> () {
        let follower_ids: Vec<Response> = self.notify (Message::FactionGetFollowers (self.faction_id, self.id));
        let follower_ids: &Vec<ID> = if let Response::FactionGetFollowers (c) = Handler::reduce_responses (&follower_ids) {
            c
        } else {
            panic! ("Invalid response")
        };
        let skill_passive_id: ID = self.skill_passive_id.get ();

        for follower_id in follower_ids {
            if *follower_id != self.id {
                self.notify (Message::UnitTryAddPassive (*follower_id, skill_passive_id));
            }
        }
    }

    pub fn is_retreat (&self) -> bool {
        let mrl: u16 = self.get_statistic (UnitStatistic::MRL).0;

        mrl < THRESHOLD_RETREAT_MRL
    }

    pub fn is_rout (&self) -> bool {
        let mrl: u16 = self.get_statistic (UnitStatistic::MRL).0;

        mrl < THRESHOLD_ROUT_MRL
    }

    pub fn start_turn (&mut self) -> () {
        let is_on_impassable: Vec<Response> = self.notify (Message::GridIsUnitOnImpassable (self.id));
        let is_on_impassable: bool = if let Response::GridIsUnitOnImpassable (i) = Handler::reduce_responses (&is_on_impassable) {
            *i
        } else {
            panic! ("Invalid response")
        };

        if is_on_impassable {
            self.die ();
        }
    }

    pub fn act_switch_weapon (&mut self) -> ID {
        self.weapon_active = if self.weapon_active == WEAPON_0 {
            WEAPON_1
        } else {
            WEAPON_0
        };

        self.weapon_active
    }

    pub fn act_attack (&self) -> u8 {
        let mov: u16 = self.get_statistic (UnitStatistic::MOV).0;
        let mut drain_spl: u16 = (DRAIN_SPL * DELAY_ATTACK) as u16;
        let weapon: &Weapon = &self.weapons[self.weapon_active];
        let appliable_on_attack: Option<Box<dyn Appliable>> = weapon.try_yield_appliable (Rc::clone (&self.lists));
        let target_ids: Vec<ID> = self.find_targets_units (weapon.get_target (), weapon.get_area (), weapon.get_range ());

        for target_id in target_ids {
            let statistics_other: Vec<Response> = self.notify (Message::UnitGetStatistics (target_id));
            let statistics_other: &UnitStatistics = if let Response::UnitGetStatistics (s) = Handler::reduce_responses (&statistics_other) {
                s
            } else {
                panic! ("Invalid response")
            };
            let (damage_mrl, damage_hlt, damage_spl): (u16, u16, u16) = self.calculate_damage (statistics_other);
            let appliable_on_hit: Vec<Response> = self.notify (Message::UnitTakeDamage (target_id, damage_mrl, damage_hlt, damage_spl));
            let appliable_on_hit: Option<Change> = if let Response::UnitTakeDamage (a) = Handler::reduce_responses (&appliable_on_hit) {
                *a
            } else {
                panic! ("Invalid response")
            };

            if let Some (ref a) = appliable_on_attack {
                let _ = self.notify (Message::UnitAddAppliable (target_id, a.change ()));
            }

            if let Some (c) = appliable_on_hit {
                self.add_appliable (c.appliable (Rc::clone (&self.lists)));
            }
        }

        self.change_statistic_flat (UnitStatistic::SPL, drain_spl, false);

        get_delay (mov, DELAY_ATTACK)
    }

    pub fn act_skill (&self, skill_id: &ID) -> u8 {
        let mov: u16 = self.get_statistic (UnitStatistic::MOV).0;
        let drain_spl: u16 = (DRAIN_SPL * DELAY_SKILL) as u16;
        let index: usize = self.skills.iter ().position (|s: &Skill| s.get_id () == *skill_id)
                .expect (&format! ("Skill {:?} not found", skill_id));
        let skill: &Skill = &self.skills[index];
        let target: Target = skill.get_target ();
        let area: Area = skill.get_area ();
        let range: u8 = skill.get_range ();

        let status_id: ID = if skill.is_active () {
            skill.get_status_id ()
        } else if skill.is_toggle () {
            let (status_id_old, status_id_new): (ID, ID) = skill.switch_status ();

            self.remove_status (&status_id_old);

            status_id_new
        } else {
            panic! ("Invalid skill {:?}", skill)
        };

        if let Target::Map = target {
            let target_locations: Vec<Location> = self.find_targets_locations (area, range);

            for location in target_locations {
                self.notify (Message::GridAddStatus (location, status_id));
            }
        } else {
            let target_ids: Vec<ID> = self.find_targets_units (target, area, range);

            for target_id in target_ids {
                self.notify (Message::UnitAddStatus (target_id, status_id));
            }
        }

        self.change_statistic_flat (UnitStatistic::SPL, drain_spl, false);

        get_delay (mov, DELAY_SKILL)
    }

    pub fn act_magic (&self, magic_id: &ID) -> u8 {
        let mov: u16 = self.get_statistic (UnitStatistic::MOV).0;
        let drain_spl: u16 = (DRAIN_SPL * DELAY_MAGIC) as u16;
        let magic: &Magic = self.lists.get_magic (magic_id);
        let target: Target = magic.get_target ();
        let area: Area = magic.get_area ();
        let range: u8 = magic.get_range ();
        let status_id: ID = magic.get_status_id ();
        let cost: u16 = magic.get_cost ();
        let drain_hlt: u16 = cost * DRAIN_HLT;
        let drain_org: u16 = cost * PERCENT_1;

        if let Target::Map = target {
                let target_locations: Vec<Location> = self.find_targets_locations (area, range);

                for location in target_locations {
                    self.notify (Message::GridAddStatus (location, status_id));
                }
        } else {
            let target_ids: Vec<ID> = self.find_targets_units (target, area, range);

            for target_id in target_ids {
                self.notify (Message::UnitAddStatus (target_id, status_id));
            }
        }

        self.change_statistic_flat (UnitStatistic::HLT, drain_hlt, false);
        self.change_statistic_flat (UnitStatistic::ORG, drain_org, false);
        self.change_statistic_flat (UnitStatistic::SPL, drain_spl, false);

        get_delay (mov, DELAY_MAGIC)
    }

    pub fn act_wait (&self) -> u8 {
        let mov: u16 = self.get_statistic (UnitStatistic::MOV).0;
        let drain_spl: u16 = (DRAIN_SPL * DELAY_WAIT) as u16;

        self.change_statistic_flat (UnitStatistic::SPL, drain_spl, false);

        get_delay (mov, DELAY_WAIT)
    }

    pub fn end_turn (&mut self) -> () {
        if !self.is_retreat () {
            let city_ids: Vec<Response> = self.notify (Message::GridFindUnitCities (self.id, self.faction_id));
            let city_ids: &Vec<ID> = if let Response::GridFindUnitCities (c) = Handler::reduce_responses (&city_ids) {
                c
            } else {
                panic! ("Invalid response")
            };

            if city_ids.len () > 0 {
                let mut recover_hlt: u16 = 0;
                let mut recover_spl: u16 = 0;

                for city_id in city_ids {
                    let change_hlt: u16 = self.lists.get_city (&city_id).get_manpower ();
                    let change_spl: u16 = self.lists.get_city (&city_id).get_equipment ();

                    recover_hlt += change_hlt;
                    recover_spl += change_spl;
                }

                self.change_statistic_flat (UnitStatistic::HLT, recover_hlt, true);
                self.change_statistic_flat (UnitStatistic::SPL, recover_spl, false);
            }
        }

        self.change_statistic_flat (UnitStatistic::MRL, RECOVER_MRL, true);
        self.dec_durations ();

        let appliable_on_occupy: Vec<Response> = self.notify (Message::GridTryYieldAppliable (self.id));
        let appliable_on_occupy: &Option<Change> = if let Response::GridTryYieldAppliable (c) = Handler::reduce_responses (&appliable_on_occupy) {
            c
        } else {
            panic! ("Invalid response")
        };

        if let Some (c) = appliable_on_occupy {
            let appliable_on_occupy: Box<dyn Appliable> = match c {
                Change::Modifier ( .. ) => {
                    let modifier: Modifier = c.modifier (Rc::clone (&self.lists));
                    let appliable: Box<dyn Appliable> = Box::new (modifier);

                    appliable
                }
                Change::Effect ( .. ) => {
                    let effect: Effect = c.effect (Rc::clone (&self.lists));
                    let appliable: Box<dyn Appliable> = Box::new (effect);

                    appliable
                }
            };

            self.add_appliable (appliable_on_occupy);
        }
    }

    pub fn get_statistics (&self) -> UnitStatistics {
        self.statistics.get ()
    }

    pub fn get_faction_id (&self) -> ID {
        self.faction_id
    }
}

impl Observer for Unit {
    fn respond (&self, message: Message) -> Option<Response> {
        match message {
            Message::UnitTakeDamage (u, m, h, s) => if u == self.id {
                let appliable: Option<Change> = self.try_yield_appliable (Rc::clone (&self.lists))
                        .map (|a: Box<dyn Appliable>| a.change ());

                self.take_damage (m, h, s);

                Some (Response::UnitTakeDamage (appliable))
            } else {
                None
            }
            Message::UnitAddStatus (u, s) => if u == self.id {
                let status: Status = self.lists.get_status (&s).clone ();

                self.add_status (status);

                Some (Response::UnitAddStatus)
            } else {
                None
            }
            Message::UnitAddAppliable (u, c) => if u == self.id {
                let appliable = c.appliable (Rc::clone (&self.lists));

                self.add_appliable (appliable);

                Some (Response::UnitAddStatus)
            } else {
                None
            }
            Message::UnitGetStatistics  (u) => if u == self.id {
                Some (Response::UnitGetStatistics (self.get_statistics ()))
            } else {
                None
            }
            Message::UnitGetFactionId (u) => if u == self.id {
                Some (Response::UnitGetFactionId (self.faction_id))
            } else {
                None
            }
            Message::UnitChangeModifierTerrain (u, m) => if u == self.id {
                self.change_modifier_terrain (m);

                Some (Response::UnitChangeModifierTerrain)
            } else {
                None
            }
            Message::UnitTryAddPassive (u, s) => if u == self.id {
                self.try_add_passive (&s);

                Some (Response::UnitTryAddPassive)
            } else {
                None
            }
            Message::UnitSetLeader (u, l) => if u == self.id {
                self.set_leader (l);

                Some (Response::UnitTryAddPassive)
            } else {
                None
            }
            Message::UnitSendPassive (u) => if u == self.id {
                self.send_passive ();

                Some (Response::UnitSendPassive)
            } else {
                None
            }
            _ => None,
        }
    }

    fn set_observer_id (&self, observer_id: ID) -> bool {
        if self.observer_id.get () < ID_UNINITIALISED {
            false
        } else {
            self.observer_id.replace (observer_id);

            true
        }
    }
}

impl Subject for Unit {
    fn notify (&self, message: Message) -> Vec<Response> {
        self.handler.upgrade ()
                .expect (&format! ("Pointer upgrade failed for {:?}", self.handler))
                .borrow ()
                .notify (message)
    }
}

impl Applier for Unit {
    fn try_yield_appliable (&self, _lists: Rc<Lists>) -> Option<Box<dyn Appliable>> {
        self.statuses.borrow ().get (&Trigger::OnHit).and_then (|c: &Vec<Status>|
            c.get (0).and_then (|s: &Status| s.try_yield_appliable (Rc::clone (&self.lists)) )
        )
    }

    fn get_target (&self) -> Target {
        Target::Enemy
    }
}

impl Changeable for Unit {
    fn add_appliable (&self, appliable: Box<dyn Appliable>) -> bool {
        let change: Change = appliable.change ();

        match change {
            Change::Modifier ( .. ) => {
                let modifier: Modifier = appliable.modifier ();

                if modifier.can_stack () || !self.modifiers.borrow ().contains (&modifier){
                    for adjustment in modifier.get_adjustments () {
                        if let Some (a) = adjustment {
                            if let StatisticType::Unit (s) = a.0 {
                                self.change_statistic_percentage (s, a.1, a.2);
                            }
                        }
                    }

                    self.modifiers.borrow_mut ().push (modifier);

                    true
                } else {
                    false
                }
            }
            Change::Effect ( .. ) => {
                let effect: Effect = appliable.effect ();

                for adjustment in effect.get_adjustments () {
                    if let Some (a) = adjustment {
                        if let StatisticType::Unit (s) = a.0 {
                            if effect.can_stack_or_is_flat () {
                                self.change_statistic_flat (s, a.1, a.2);
                            } else {
                                self.change_statistic_percentage (s, a.1, a.2);
                            }
                        }
                    }
                }

                true
            }
        }
    }

    fn add_status (&self, status: Status) -> bool {
        let trigger: Trigger = status.get_trigger ();

        if let Trigger::OnOccupy = trigger {
            false
        } else {
            let appliable: Box<dyn Appliable> = status.try_yield_appliable (Rc::clone (&self.lists))
                    .expect (&format! ("Appliable not found for status {:?}", status));

            match trigger {
                Trigger::OnAttack => {
                    let weapon: &Weapon = &self.weapons[self.weapon_active];

                    weapon.add_status (status);

                    true
                }
                Trigger::OnHit => {
                    let mut collection: Vec<Status> = Vec::new ();

                    collection.push (status);
                    self.statuses.borrow_mut ().insert (trigger, collection);

                    true
                }
                Trigger::None => {
                    if let Target::This = status.get_target () {
                        {let mut collection = self.statuses.borrow_mut ();
                        let collection: &mut Vec<Status> = collection.get_mut (&trigger)
                                .expect (&format! ("Collection not found for trigger {:?}", trigger));

                        collection.push (status);
                        self.add_appliable (appliable);}

                        true
                    } else {
                        false
                    }
                }
                _ => false,
            }
        }
    }

    fn remove_modifier (&self, modifier_id: &ID) -> bool {
        let index: Option<usize> = self.modifiers.borrow ().iter ().position (|m: &Modifier| m.get_id () == *modifier_id);

        if let Some (i) = index {
            self.modifiers.borrow_mut ().swap_remove (i);

            true
        } else {
            false
        }
    }

    fn remove_status (&self, status_id: &ID) -> bool {
        for (_, collection) in self.statuses.borrow_mut ().iter_mut () {
            let index: Option<usize> = collection.iter ().position (|m: &Status| m.get_id () == *status_id);

            if let Some (i) = index {
                let status: Status = collection.swap_remove (i);

                if let Change::Modifier (m, _) = status.get_change () {
                    self.remove_modifier (&m);
                }

                return true
            }
        }

        false
    }

    fn dec_durations (&self) -> () {
        self.modifiers.borrow_mut ().retain_mut (|m: &mut Modifier| !m.dec_duration ());

        for (_, collection) in self.statuses.borrow_mut ().iter_mut () {
            for status in collection.iter_mut () {
                status.dec_duration ();
            }

            let nexts: Vec<ID> = collection.iter ().filter_map (|s: &Status|
                if s.get_duration () == 0 {
                    if let Change::Modifier (m, _) = s.get_change () {
                        self.remove_modifier (&m);
                    }

                    s.get_next_id ()
                } else {
                    None
                }
            ).collect ();

            collection.retain (|s: &Status| s.get_duration () > 0);
            collection.extend (nexts.iter ().map (|n: &ID| self.lists.get_status (&n).clone ()));
        }

    }
}

#[derive (Debug)]
pub struct UnitBuilder {
    id: ID,
    statistics: UnitStatistics,
    weapon_ids: [ID; WEAPONS_LENGTH],
    skill_passive_id: ID,
    skill_ids: [ID; SKILLS_LENGTH],
    magics_usable: [bool; 3],
    faction_id: ID,
    rank: Rank,
}

impl UnitBuilder {
    pub const fn new (id: ID, statistics: UnitStatistics, weapon_ids: [ID; WEAPONS_LENGTH], skill_passive_id: ID, skill_ids: [ID; SKILLS_LENGTH], magics_usable: [bool; 3], faction_id: ID, rank: Rank) -> Self {
        Self { id, statistics, weapon_ids, skill_passive_id, skill_ids, magics_usable, faction_id, rank }
    }

    pub fn build (&self, lists: Rc<Lists>, handler: Weak<RefCell<Handler>>) -> Unit {
        Unit::new (self.id, lists, self.statistics, self.weapon_ids, self.skill_passive_id, self.skill_ids, self.magics_usable, self.faction_id, self.rank, handler)
    }
}

#[cfg (test)]
pub mod tests {
    use super::*;
    use crate::event::{EVENT_FACTION_ADD_MEMBER, EVENT_FACTION_GET_FOLLOWERS, EVENT_FACTION_GET_LEADER, EVENT_FACTION_IS_MEMBER, EVENT_GRID_ADD_STATUS, EVENT_GRID_FIND_DISTANCE_BETWEEN, EVENT_GRID_FIND_LOCATIONS, EVENT_GRID_FIND_UNITS, EVENT_GRID_FIND_UNIT_CITIES, EVENT_GRID_GET_UNIT_LOCATION, EVENT_GRID_TRY_YIELD_APPLIABLE, EVENT_UNIT_ADD_APPLIABLE, EVENT_UNIT_ADD_STATUS, EVENT_UNIT_CHANGE_MODIFIER_TERRAIN, EVENT_UNIT_GET_FACTION_ID, EVENT_UNIT_GET_STATISTICS, EVENT_UNIT_SEND_PASSIVE, EVENT_UNIT_TAKE_DAMAGE, EVENT_UNIT_TRY_ADD_PASSIVE};
    use crate::map::grid::tests::generate_grid;
    use crate::tests::generate_lists;
    use crate::event::handler::tests::generate_handler;

    pub fn generate_units (handler: Rc<RefCell<Handler>>) -> (Rc<RefCell<Unit>>, Rc<RefCell<Unit>>, Rc<RefCell<Unit>>) {
        let lists = generate_lists ();
        let unit_builder_0 = lists.get_unit_builder (&0);
        let unit_0 = unit_builder_0.build (Rc::clone (&lists), Rc::downgrade (&handler));
        let unit_0 = RefCell::new (unit_0);
        let unit_0 = Rc::new (unit_0);
        let unit_builder_1 = lists.get_unit_builder (&1);
        let unit_1 = unit_builder_1.build (Rc::clone (&lists), Rc::downgrade (&handler));
        let unit_1 = RefCell::new (unit_1);
        let unit_1 = Rc::new (unit_1);
        let unit_builder_2 = lists.get_unit_builder (&2);
        let unit_2 = unit_builder_2.build (Rc::clone (&lists), Rc::downgrade (&handler));
        let unit_2 = RefCell::new (unit_2);
        let unit_2 = Rc::new (unit_2);

        (unit_0, unit_1, unit_2)
    }

    fn generate_modifiers () -> (Box<Modifier>, Box<Modifier>) {
        let lists = generate_lists ();
        let modifier_builder_3 = lists.get_modifier_builder (&3);
        let modifier_3 = modifier_builder_3.build (true);
        let modifier_3 = Box::new (modifier_3);
        let modifier_builder_4 = lists.get_modifier_builder (&4);
        let modifier_4 = modifier_builder_4.build (false);
        let modifier_4 = Box::new (modifier_4);

        (modifier_3, modifier_4)
    }

    fn generate_effects () -> (Box<Effect>, Box<Effect>) {
        let lists = generate_lists ();
        let effect_0 = lists.get_effect (&0).clone ();
        let effect_0 = Box::new (effect_0);
        let effect_1 = lists.get_effect (&1).clone ();
        let effect_1 = Box::new (effect_1);

        (effect_0, effect_1)
    }

    fn generate_statuses () -> (Status, Status, Status) {
        let lists = generate_lists ();
        let status_0 = lists.get_status (&0).clone ();
        let status_1 = lists.get_status (&1).clone ();
        let status_5 = lists.get_status (&5).clone ();

        (status_0, status_1, status_5)
    }

    pub fn generate_factions (handler: Rc<RefCell<Handler>>) -> (Rc<RefCell<Faction>>, Rc<RefCell<Faction>>) {
        let lists = generate_lists ();
        let faction_builder_0 = lists.get_faction_builder (&0);
        let faction_0 = faction_builder_0.build (Rc::downgrade (&handler));
        let faction_0 = RefCell::new (faction_0);
        let faction_0 = Rc::new (faction_0);
        let faction_builder_1 = lists.get_faction_builder (&1);
        let faction_1 = faction_builder_1.build (Rc::downgrade (&handler));
        let faction_1 = RefCell::new (faction_1);
        let faction_1 = Rc::new (faction_1);

        (faction_0, faction_1)
    }

    #[test]
    fn unit_change_statistic_flat () {
        let handler = generate_handler ();
        let (unit_0, _, _) = generate_units (Rc::clone (&handler));

        // Test constant
        unit_0.borrow_mut ().change_statistic_flat (UnitStatistic::ATK, 5, true); // Test additive change
        assert_eq! (unit_0.borrow ().get_statistic (UnitStatistic::ATK).0, 25);
        unit_0.borrow_mut ().change_statistic_flat (UnitStatistic::ATK, 5, false); // Test subtractive change
        assert_eq! (unit_0.borrow ().get_statistic (UnitStatistic::ATK).0, 20);
        unit_0.borrow_mut ().change_statistic_flat (UnitStatistic::ATK, ATK_MAX, true); // Test maximum change
        assert_eq! (unit_0.borrow ().get_statistic (UnitStatistic::ATK).0, ATK_MAX);
        unit_0.borrow_mut ().change_statistic_flat (UnitStatistic::ATK, ATK_MAX, false); // Test minimum change
        assert_eq! (unit_0.borrow ().get_statistic (UnitStatistic::ATK).0, 0);
        // Test quantity
        unit_0.borrow_mut ().change_statistic_flat (UnitStatistic::HLT, 10, false); // Test subtractive change
        assert_eq! (unit_0.borrow ().get_statistic (UnitStatistic::HLT).0, 990);
        unit_0.borrow_mut ().change_statistic_flat (UnitStatistic::HLT, 5, true); // Test additive change
        assert_eq! (unit_0.borrow ().get_statistic (UnitStatistic::HLT).0, 995);
        unit_0.borrow_mut ().change_statistic_flat (UnitStatistic::HLT, HLT_MAX, true); // Test maximum change
        assert_eq! (unit_0.borrow ().get_statistic (UnitStatistic::HLT).0, HLT_MAX);
        unit_0.borrow_mut ().change_statistic_flat (UnitStatistic::HLT, HLT_MAX, false); // Test minimum change
        assert_eq! (unit_0.borrow ().get_statistic (UnitStatistic::HLT).0, 0);
    }

    #[test]
    fn unit_change_statistic_percentage () {
        let handler = generate_handler ();
        let (unit_0, _, _) = generate_units (Rc::clone (&handler));

        // Test constant
        unit_0.borrow_mut ().change_statistic_percentage (UnitStatistic::ATK, 10, true); // Test additive change
        assert_eq! (unit_0.borrow ().get_statistic (UnitStatistic::ATK).0, 22);
        unit_0.borrow_mut ().change_statistic_percentage (UnitStatistic::ATK, 10, false); // Test subtractive change
        assert_eq! (unit_0.borrow ().get_statistic (UnitStatistic::ATK).0, 20);
        unit_0.borrow_mut ().change_statistic_percentage (UnitStatistic::ATK, 1000, true); // Test maximum change
        assert_eq! (unit_0.borrow ().get_statistic (UnitStatistic::ATK).0, ATK_MAX);
        unit_0.borrow_mut ().change_statistic_percentage (UnitStatistic::ATK, 1000, false); // Test minimum change
        assert_eq! (unit_0.borrow ().get_statistic (UnitStatistic::ATK).0, 0);
        // Test quantity
        unit_0.borrow_mut ().change_statistic_percentage (UnitStatistic::HLT, 10, false); // Test subtractive change
        assert_eq! (unit_0.borrow ().get_statistic (UnitStatistic::HLT).0, 900);
        unit_0.borrow_mut ().change_statistic_percentage (UnitStatistic::HLT, 5, true); // Test additive change
        assert_eq! (unit_0.borrow ().get_statistic (UnitStatistic::HLT).0, 950);
        unit_0.borrow_mut ().change_statistic_percentage (UnitStatistic::HLT, 1000, true); // Test maximum change
        assert_eq! (unit_0.borrow ().get_statistic (UnitStatistic::HLT).0, HLT_MAX);
        unit_0.borrow_mut ().change_statistic_percentage (UnitStatistic::HLT, 1000, false); // Test minimum change
        assert_eq! (unit_0.borrow ().get_statistic (UnitStatistic::HLT).0, 0);
    }

    #[test]
    fn unit_filter_unit_allegiance () {
        let handler = generate_handler ();
        let (unit_0, unit_1, unit_2) = generate_units (Rc::clone (&handler));
        let (faction_0, faction_1) = generate_factions (Rc::clone (&handler));
        let unit_0_id = handler.borrow_mut ().register (Rc::clone (&unit_0) as Rc<RefCell<dyn Observer>>);
        let unit_1_id = handler.borrow_mut ().register (Rc::clone (&unit_1) as Rc<RefCell<dyn Observer>>);
        let unit_2_id = handler.borrow_mut ().register (Rc::clone (&unit_2) as Rc<RefCell<dyn Observer>>);
        let faction_0_id = handler.borrow_mut ().register (Rc::clone (&faction_0) as Rc<RefCell<dyn Observer>>);
        let faction_1_id = handler.borrow_mut ().register (Rc::clone (&faction_1) as Rc<RefCell<dyn Observer>>);

        handler.borrow_mut ().subscribe (unit_0_id, EVENT_UNIT_GET_FACTION_ID);
        handler.borrow_mut ().subscribe (unit_1_id, EVENT_UNIT_GET_FACTION_ID);
        handler.borrow_mut ().subscribe (unit_2_id, EVENT_UNIT_GET_FACTION_ID);
        handler.borrow_mut ().subscribe (faction_0_id, EVENT_FACTION_IS_MEMBER);
        handler.borrow_mut ().subscribe (faction_1_id, EVENT_FACTION_IS_MEMBER);
        handler.borrow_mut ().subscribe (faction_0_id, EVENT_FACTION_ADD_MEMBER);
        handler.borrow_mut ().subscribe (faction_1_id, EVENT_FACTION_ADD_MEMBER);
        handler.borrow ().notify (Message::FactionAddMember (0, 0));
        handler.borrow ().notify (Message::FactionAddMember (0, 1));
        handler.borrow ().notify (Message::FactionAddMember (1, 2));

        let response = unit_0.borrow ().filter_unit_allegiance (&vec![0, 1, 2], true);
        assert_eq! (response.len (), 2);
        assert_eq! (response.contains (&0), true);
        assert_eq! (response.contains (&1), true);
        let response = unit_0.borrow ().filter_unit_allegiance (&vec![0, 1, 2], false);
        assert_eq! (response.len (), 1);
        assert_eq! (response.contains (&2), true);
    }

    #[test]
    fn unit_choose_targets_units () {
        let handler = generate_handler ();
        let (unit_0, unit_1, unit_2) = generate_units (Rc::clone (&handler));
        let grid = generate_grid (Rc::downgrade (&handler));
        let (faction_0, faction_1) = generate_factions (Rc::clone (&handler));
        let grid_id = handler.borrow_mut ().register (Rc::clone (&grid) as Rc<RefCell<dyn Observer>>);
        let unit_0_id = handler.borrow_mut ().register (Rc::clone (&unit_0) as Rc<RefCell<dyn Observer>>);
        let unit_1_id = handler.borrow_mut ().register (Rc::clone (&unit_1) as Rc<RefCell<dyn Observer>>);
        let unit_2_id = handler.borrow_mut ().register (Rc::clone (&unit_2) as Rc<RefCell<dyn Observer>>);
        let faction_0_id = handler.borrow_mut ().register (Rc::clone (&faction_0) as Rc<RefCell<dyn Observer>>);
        let faction_1_id = handler.borrow_mut ().register (Rc::clone (&faction_1) as Rc<RefCell<dyn Observer>>);

        handler.borrow_mut ().subscribe (grid_id, EVENT_GRID_FIND_UNITS);
        handler.borrow_mut ().subscribe (unit_0_id, EVENT_UNIT_GET_FACTION_ID);
        handler.borrow_mut ().subscribe (unit_1_id, EVENT_UNIT_GET_FACTION_ID);
        handler.borrow_mut ().subscribe (unit_2_id, EVENT_UNIT_GET_FACTION_ID);
        handler.borrow_mut ().subscribe (faction_0_id, EVENT_FACTION_IS_MEMBER);
        handler.borrow_mut ().subscribe (faction_1_id, EVENT_FACTION_IS_MEMBER);
        handler.borrow_mut ().subscribe (faction_0_id, EVENT_FACTION_ADD_MEMBER);
        handler.borrow_mut ().subscribe (faction_1_id, EVENT_FACTION_ADD_MEMBER);
        grid.borrow_mut ().place_unit (0, (0, 0));
        grid.borrow_mut ().place_unit (1, (1, 1));
        grid.borrow_mut ().place_unit (2, (1, 0));
        handler.borrow ().notify (Message::FactionAddMember (0, 0));
        handler.borrow ().notify (Message::FactionAddMember (0, 1));
        handler.borrow ().notify (Message::FactionAddMember (1, 2));

        let results: Vec<ID> = unit_0.borrow ().choose_targets_units (&vec![0], Target::This, Area::Single, 1);
        assert_eq! (results.len (), 1);
        assert_eq! (results.contains (&0), true);
        let results: Vec<ID> = unit_0.borrow ().choose_targets_units (&vec![1], Target::Ally, Area::Path (0), 1);
        assert_eq! (results.len (), 1);
        assert_eq! (results.contains (&1), true);
        let results: Vec<ID> = unit_0.borrow ().choose_targets_units (&vec![0, 1], Target::Allies, Area::Radial (1), 0);
        assert_eq! (results.len (), 1);
        assert_eq! (results.contains (&0), true);
        let results: Vec<ID> = unit_0.borrow ().choose_targets_units (&vec![0, 1], Target::Allies, Area::Radial (2), 0);
        assert_eq! (results.len (), 2);
        assert_eq! (results.contains (&0), true);
        assert_eq! (results.contains (&1), true);
        let results: Vec<ID> = unit_0.borrow ().choose_targets_units (&vec![2], Target::Enemy, Area::Radial (1), 0);
        assert_eq! (results.len (), 1);
        assert_eq! (results.contains (&2), true);
    }

    #[test]
    fn unit_find_targets_units () {
        let handler = generate_handler ();
        let (unit_0, unit_1, unit_2) = generate_units (Rc::clone (&handler));
        let grid = generate_grid (Rc::downgrade (&handler));
        let (faction_0, faction_1) = generate_factions (Rc::clone (&handler));
        let grid_id = handler.borrow_mut ().register (Rc::clone (&grid) as Rc<RefCell<dyn Observer>>);
        let unit_0_id = handler.borrow_mut ().register (Rc::clone (&unit_0) as Rc<RefCell<dyn Observer>>);
        let unit_1_id = handler.borrow_mut ().register (Rc::clone (&unit_1) as Rc<RefCell<dyn Observer>>);
        let unit_2_id = handler.borrow_mut ().register (Rc::clone (&unit_2) as Rc<RefCell<dyn Observer>>);
        let faction_0_id = handler.borrow_mut ().register (Rc::clone (&faction_0) as Rc<RefCell<dyn Observer>>);
        let faction_1_id = handler.borrow_mut ().register (Rc::clone (&faction_1) as Rc<RefCell<dyn Observer>>);

        handler.borrow_mut ().subscribe (grid_id, EVENT_GRID_FIND_UNITS);
        handler.borrow_mut ().subscribe (unit_0_id, EVENT_UNIT_GET_FACTION_ID);
        handler.borrow_mut ().subscribe (unit_1_id, EVENT_UNIT_GET_FACTION_ID);
        handler.borrow_mut ().subscribe (unit_2_id, EVENT_UNIT_GET_FACTION_ID);
        handler.borrow_mut ().subscribe (faction_0_id, EVENT_FACTION_IS_MEMBER);
        handler.borrow_mut ().subscribe (faction_1_id, EVENT_FACTION_IS_MEMBER);
        handler.borrow_mut ().subscribe (faction_0_id, EVENT_FACTION_ADD_MEMBER);
        handler.borrow_mut ().subscribe (faction_1_id, EVENT_FACTION_ADD_MEMBER);
        handler.borrow ().notify (Message::FactionAddMember (0, 0));
        handler.borrow ().notify (Message::FactionAddMember (0, 1));
        handler.borrow ().notify (Message::FactionAddMember (1, 2));
        grid.borrow_mut ().place_unit (0, (0, 0));
        grid.borrow_mut ().place_unit (1, (1, 1));
        grid.borrow_mut ().place_unit (2, (1, 0));

        let results: Vec<ID> = unit_0.borrow ().find_targets_units (Target::This, Area::Single, 1);
        assert_eq! (results.len (), 1);
        assert_eq! (results.contains (&0), true);
        let results: Vec<ID> = unit_0.borrow ().find_targets_units (Target::Ally, Area::Single, 1);
        assert_eq! (results.len (), 1);
        assert_eq! (results.contains (&0), true);
        let results: Vec<ID> = unit_0.borrow ().find_targets_units (Target::Allies, Area::Radial (1), 0);
        assert_eq! (results.len (), 1);
        assert_eq! (results.contains (&0), true);
        let results: Vec<ID> = unit_0.borrow ().find_targets_units (Target::Allies, Area::Radial (2), 0);
        assert_eq! (results.len (), 2);
        assert_eq! (results.contains (&0), true);
        assert_eq! (results.contains (&1), true);
        let results: Vec<ID> = unit_0.borrow ().find_targets_units (Target::Enemy, Area::Radial (0), 1);
        assert_eq! (results.len (), 1);
        assert_eq! (results.contains (&2), true);
        let results: Vec<ID> = unit_2.borrow ().find_targets_units (Target::Enemies, Area::Path (0), 1);
        assert_eq! (results.len (), 1);
        assert_eq! (results.contains (&1), true);
        let results: Vec<ID> = unit_0.borrow ().find_targets_units (Target::Enemies, Area::Path (0), 1); // Test empty find
        assert_eq! (results.len (), 0);
    }

    #[test]
    fn unit_choose_targets_locations () {
        let handler = generate_handler ();
        let (unit_0, _, _) = generate_units (Rc::clone (&handler));
        let grid = generate_grid (Rc::downgrade (&handler));
        let grid_id = handler.borrow_mut ().register (Rc::clone (&grid) as Rc<RefCell<dyn Observer>>);
        let unit_0_id = handler.borrow_mut ().register (Rc::clone (&unit_0) as Rc<RefCell<dyn Observer>>);

        handler.borrow_mut ().subscribe (grid_id, EVENT_GRID_GET_UNIT_LOCATION);
        handler.borrow_mut ().subscribe (grid_id, EVENT_GRID_FIND_LOCATIONS);
        handler.borrow_mut ().subscribe (unit_0_id, EVENT_UNIT_GET_FACTION_ID);
        grid.borrow_mut ().place_unit (0, (0, 0));

        let results: Vec<Location> = unit_0.borrow ().choose_targets_locations (&vec![(0, 0)], Area::Single, 1);
        assert_eq! (results.len (), 1);
        assert_eq! (results.contains (&(0, 0)), true);
        let results: Vec<Location> = unit_0.borrow ().choose_targets_locations (&vec![(0, 0), (0, 1)], Area::Path (0), 1);
        assert_eq! (results.len (), 1);
        assert_eq! (results.contains (&(0, 1)), true);
        let results: Vec<Location> = unit_0.borrow ().choose_targets_locations (&vec![(0, 0), (0, 1), (1, 0)], Area::Radial (1), 0);
        assert_eq! (results.len (), 3);
        assert_eq! (results.contains (&(0, 0)), true);
        assert_eq! (results.contains (&(0, 1)), true);
        assert_eq! (results.contains (&(1, 0)), true);
        let results: Vec<Location> = unit_0.borrow ().choose_targets_locations (&vec![(1, 0), (0, 1), (0, 0)], Area::Radial (1), 0);
        assert_eq! (results.len (), 3);
        assert_eq! (results.contains (&(1, 0)), true);
        assert_eq! (results.contains (&(0, 0)), true);
        assert_eq! (results.contains (&(1, 1)), true);
    }

    #[test]
    fn unit_find_targets_locations () {
        let handler = generate_handler ();
        let (unit_0, _, _) = generate_units (Rc::clone (&handler));
        let grid = generate_grid (Rc::downgrade (&handler));
        let grid_id = handler.borrow_mut ().register (Rc::clone (&grid) as Rc<RefCell<dyn Observer>>);
        let unit_0_id = handler.borrow_mut ().register (Rc::clone (&unit_0) as Rc<RefCell<dyn Observer>>);

        handler.borrow_mut ().subscribe (grid_id, EVENT_GRID_GET_UNIT_LOCATION);
        handler.borrow_mut ().subscribe (grid_id, EVENT_GRID_FIND_LOCATIONS);
        handler.borrow_mut ().subscribe (unit_0_id, EVENT_UNIT_GET_FACTION_ID);
        grid.borrow_mut ().place_unit (0, (0, 0));
 
        let results: Vec<Location> = unit_0.borrow ().find_targets_locations (Area::Single, 1);
        assert_eq! (results.len (), 3);
        assert_eq! (results.contains (&(0, 0)), true);
        assert_eq! (results.contains (&(0, 1)), true);
        assert_eq! (results.contains (&(1, 0)), true);
        let results: Vec<Location> = unit_0.borrow ().find_targets_locations (Area::Path (0), 1);
        assert_eq! (results.len (), 1);
        assert_eq! (results.contains (&(0, 1)), true);
        let results: Vec<Location> = unit_0.borrow ().find_targets_locations (Area::Path (0), 2);
        assert_eq! (results.len (), 2);
        assert_eq! (results.contains (&(0, 1)), true);
        assert_eq! (results.contains (&(0, 2)), true);
        let results: Vec<Location> = unit_0.borrow ().find_targets_locations (Area::Path (1), 1);
        assert_eq! (results.len (), 2);
        assert_eq! (results.contains (&(0, 1)), true);
        assert_eq! (results.contains (&(1, 1)), true);
        let results: Vec<Location> = unit_0.borrow ().find_targets_locations (Area::Path (2), 2);
        assert_eq! (results.len (), 4);
        assert_eq! (results.contains (&(0, 1)), true);
        assert_eq! (results.contains (&(0, 2)), true);
        assert_eq! (results.contains (&(1, 1)), true);
        assert_eq! (results.contains (&(1, 2)), true);

        // Radial is non-deterministic (any target could be picked to search)
        for _ in 0 .. 100 {
            let results: Vec<Location> = unit_0.borrow ().find_targets_locations (Area::Radial (1), 1);  

            if results.contains (&(0, 2)) {
                assert_eq! (results.len (), 4);    
                assert_eq! (results.contains (&(0, 1)), true);
                assert_eq! (results.contains (&(1, 1)), true);
            } else {
                assert_eq! (results.len (), 3);
                assert_eq! (results.contains (&(1, 0)), true);
                assert! (results.contains (&(0, 1)) || results.contains (&(1, 1)));
            }

            assert_eq! (results.contains (&(0, 0)), true);
        }
    }

    #[test]
    fn unit_apply_inactive_skills () {
        let handler = generate_handler ();
        let (unit_0, _, _) = generate_units (Rc::clone (&handler));

        unit_0.borrow ().apply_inactive_skills ();
        assert_eq! (unit_0.borrow ().statuses.borrow ().get (&Trigger::None).unwrap ().len (), 2);
        assert_eq! (unit_0.borrow ().modifiers.borrow ().len (), 2);
    }

    #[test]
    fn unit_change_modifier_terrain () {
        let handler = generate_handler ();
        let (unit_0, _, _) = generate_units (Rc::clone (&handler));
        let grid = generate_grid (Rc::downgrade (&handler));
        let (faction_0, _) = generate_factions (Rc::clone (&handler));
        let grid_id = handler.borrow_mut ().register (Rc::clone (&grid) as Rc<RefCell<dyn Observer>>);
        let unit_0_id = handler.borrow_mut ().register (Rc::clone (&unit_0) as Rc<RefCell<dyn Observer>>);
        let faction_0_id = handler.borrow_mut ().register (Rc::clone (&faction_0) as Rc<RefCell<dyn Observer>>);

        handler.borrow_mut ().subscribe (grid_id, EVENT_GRID_FIND_UNITS);
        handler.borrow_mut ().subscribe (unit_0_id, EVENT_UNIT_GET_FACTION_ID);
        handler.borrow_mut ().subscribe (unit_0_id, EVENT_UNIT_CHANGE_MODIFIER_TERRAIN);
        handler.borrow_mut ().subscribe (faction_0_id, EVENT_FACTION_IS_MEMBER);
        handler.borrow_mut ().subscribe (faction_0_id, EVENT_FACTION_ADD_MEMBER);
        handler.borrow_mut ().subscribe (faction_0_id, EVENT_FACTION_GET_LEADER);
        handler.borrow ().notify (Message::FactionAddMember (0, 0));
        faction_0.borrow ().add_follower (0, 0);
        
        // Test empty modifier
        grid.borrow_mut ().place_unit (0, (0, 0));
        assert_eq! (unit_0.borrow ().modifier_terrain.get ().get_id (), ID_UNINITIALISED);
        assert_eq! (unit_0.borrow ().modifiers.borrow ().len (), 0);
        // Test non-empty modifier
        grid.borrow_mut ().move_unit (0, vec![Direction::Right, Direction::Down]);
        assert_eq! (unit_0.borrow ().modifier_terrain.get ().get_id (), 3);
        assert_eq! (unit_0.borrow ().modifiers.borrow ().len (), 1);
    }

    #[test]
    fn unit_set_leader () {
        let handler = generate_handler ();
        let (unit_0, unit_1, _) = generate_units (Rc::clone (&handler));

        // Test leader set
        unit_0.borrow ().set_leader (1);
        assert! (matches! (unit_0.borrow ().rank.get (), Rank::Leader));
        // Test follower set
        unit_1.borrow ().set_leader (0);
        assert! (matches! (unit_1.borrow ().rank.get (), Rank::Follower (0)));
    }

    #[test]
    fn unit_try_add_passive () {
        let lists = generate_lists ();
        let handler = generate_handler ();
        let (unit_0, unit_1, _) = generate_units (Rc::clone (&handler));
        let grid = generate_grid (Rc::downgrade (&handler));
        let grid_id = handler.borrow_mut ().register (Rc::clone (&grid) as Rc<RefCell<dyn Observer>>);
        let unit_0_id = handler.borrow_mut ().register (Rc::clone (&unit_0) as Rc<RefCell<dyn Observer>>);
        let unit_1_id = handler.borrow_mut ().register (Rc::clone (&unit_1) as Rc<RefCell<dyn Observer>>);
        let skill_passive_id = unit_0.borrow ().skill_passive_id.get ();
        let skill_passive = lists.get_skill (&skill_passive_id);
        let status_passive_id = skill_passive.get_status_id ();

        handler.borrow_mut ().subscribe (grid_id, EVENT_GRID_FIND_DISTANCE_BETWEEN);
        handler.borrow_mut ().subscribe (unit_0_id, EVENT_UNIT_GET_FACTION_ID);
        handler.borrow_mut ().subscribe (unit_1_id, EVENT_UNIT_GET_FACTION_ID);
        grid.borrow_mut ().place_unit (0, (0, 0));
        grid.borrow_mut ().place_unit (1, (1, 1));
        unit_1.borrow ().set_leader (0);

        // Test leader add
        assert_eq! (unit_0.borrow ().try_add_passive (&status_passive_id), false);
        // Test follower add
        assert_eq! (unit_1.borrow ().try_add_passive (&status_passive_id), true);
        assert_eq! (unit_1.borrow ().skill_passive_id.get (), status_passive_id);
        assert_eq! (unit_1.borrow ().statuses.borrow ().get (&Trigger::None).unwrap ().len (), 1);
        assert_eq! (unit_1.borrow ().modifiers.borrow ().len (), 1);
    }

    #[test]
    fn unit_send_passive () {
        let lists = generate_lists ();
        let handler = generate_handler ();
        let (unit_0, unit_1, _) = generate_units (Rc::clone (&handler));
        let grid = generate_grid (Rc::downgrade (&handler));
        let (faction_0, _) = generate_factions (Rc::clone (&handler));
        let grid_id = handler.borrow_mut ().register (Rc::clone (&grid) as Rc<RefCell<dyn Observer>>);
        let unit_0_id = handler.borrow_mut ().register (Rc::clone (&unit_0) as Rc<RefCell<dyn Observer>>);
        let unit_1_id = handler.borrow_mut ().register (Rc::clone (&unit_1) as Rc<RefCell<dyn Observer>>);
        let faction_0_id = handler.borrow_mut ().register (Rc::clone (&faction_0) as Rc<RefCell<dyn Observer>>);
        let skill_passive_id = unit_0.borrow ().skill_passive_id.get ();
        let skill_passive = lists.get_skill (&skill_passive_id);
        let status_passive_id = skill_passive.get_status_id ();

        handler.borrow_mut ().subscribe (grid_id, EVENT_GRID_FIND_DISTANCE_BETWEEN);
        handler.borrow_mut ().subscribe (unit_0_id, EVENT_UNIT_GET_FACTION_ID);
        handler.borrow_mut ().subscribe (unit_0_id, EVENT_UNIT_SEND_PASSIVE);
        handler.borrow_mut ().subscribe (unit_1_id, EVENT_UNIT_GET_FACTION_ID);
        handler.borrow_mut ().subscribe (unit_1_id, EVENT_UNIT_TRY_ADD_PASSIVE);
        handler.borrow_mut ().subscribe (faction_0_id, EVENT_FACTION_IS_MEMBER);
        handler.borrow_mut ().subscribe (faction_0_id, EVENT_FACTION_GET_LEADER);
        handler.borrow_mut ().subscribe (faction_0_id, EVENT_FACTION_GET_FOLLOWERS);
        grid.borrow_mut ().place_unit (0, (0, 0));
        grid.borrow_mut ().place_unit (1, (0, 2));
        unit_1.borrow ().set_leader (0);
        faction_0.borrow ().add_follower (0, 0);
        faction_0.borrow ().add_follower (1, 0);

        unit_0.borrow ().send_passive ();
        assert_eq! (unit_1.borrow ().skill_passive_id.get (), status_passive_id);
        assert_eq! (unit_1.borrow ().statuses.borrow ().get (&Trigger::None).unwrap ().len (), 1);
        assert_eq! (unit_1.borrow ().modifiers.borrow ().len (), 1);
        grid.borrow_mut ().move_unit (0, vec![Direction::Right, Direction::Down, Direction::Left]);
        unit_0.borrow ().send_passive ();
        assert_eq! (unit_1.borrow ().skill_passive_id.get (), status_passive_id);
        assert_eq! (unit_1.borrow ().statuses.borrow ().get (&Trigger::None).unwrap ().len (), 0);
        assert_eq! (unit_1.borrow ().modifiers.borrow ().len (), 0);
    }

    #[test]
    fn unit_act_attack () {
        let handler = generate_handler ();
        let (unit_0, _, unit_2) = generate_units (Rc::clone (&handler));
        let grid = generate_grid (Rc::downgrade (&handler));
        let (faction_0, faction_1) = generate_factions (Rc::clone (&handler));
        let grid_id = handler.borrow_mut ().register (Rc::clone (&grid) as Rc<RefCell<dyn Observer>>);
        let unit_0_id = handler.borrow_mut ().register (Rc::clone (&unit_0) as Rc<RefCell<dyn Observer>>);
        let unit_2_id = handler.borrow_mut ().register (Rc::clone (&unit_2) as Rc<RefCell<dyn Observer>>);
        let faction_0_id = handler.borrow_mut ().register (Rc::clone (&faction_0) as Rc<RefCell<dyn Observer>>);
        let faction_1_id = handler.borrow_mut ().register (Rc::clone (&faction_1) as Rc<RefCell<dyn Observer>>);

        handler.borrow_mut ().subscribe (grid_id, EVENT_GRID_FIND_UNITS);
        handler.borrow_mut ().subscribe (unit_0_id, EVENT_UNIT_GET_FACTION_ID);
        handler.borrow_mut ().subscribe (unit_2_id, EVENT_UNIT_GET_FACTION_ID);
        handler.borrow_mut ().subscribe (unit_0_id, EVENT_UNIT_TAKE_DAMAGE);
        handler.borrow_mut ().subscribe (unit_2_id, EVENT_UNIT_TAKE_DAMAGE);
        handler.borrow_mut ().subscribe (unit_0_id, EVENT_UNIT_ADD_STATUS);
        handler.borrow_mut ().subscribe (unit_2_id, EVENT_UNIT_ADD_STATUS);
        handler.borrow_mut ().subscribe (unit_0_id, EVENT_UNIT_ADD_APPLIABLE);
        handler.borrow_mut ().subscribe (unit_2_id, EVENT_UNIT_ADD_APPLIABLE);
        handler.borrow_mut ().subscribe (unit_0_id, EVENT_UNIT_GET_STATISTICS);
        handler.borrow_mut ().subscribe (unit_2_id, EVENT_UNIT_GET_STATISTICS);
        handler.borrow_mut ().subscribe (faction_0_id, EVENT_FACTION_IS_MEMBER);
        handler.borrow_mut ().subscribe (faction_1_id, EVENT_FACTION_IS_MEMBER);
        handler.borrow_mut ().subscribe (faction_0_id, EVENT_FACTION_ADD_MEMBER);
        handler.borrow_mut ().subscribe (faction_1_id, EVENT_FACTION_ADD_MEMBER);
        handler.borrow ().notify (Message::FactionAddMember (0, 0));
        handler.borrow ().notify (Message::FactionAddMember (1, 2));
        grid.borrow_mut ().place_unit (0, (0, 1));
        grid.borrow_mut ().place_unit (2, (0, 0));

        let mrl_unit_2_0 = unit_2.borrow ().get_statistic (UnitStatistic::MRL).0;
        let hlt_unit_2_0 = unit_2.borrow ().get_statistic (UnitStatistic::HLT).0;
        let spl_unit_2_0 = unit_2.borrow ().get_statistic (UnitStatistic::SPL).0;
        assert_eq! (unit_0.borrow ().act_attack (), 15);
        let mrl_unit_2_1 = unit_2.borrow ().get_statistic (UnitStatistic::MRL).0;
        let hlt_unit_2_1 = unit_2.borrow ().get_statistic (UnitStatistic::HLT).0;
        let spl_unit_2_1 = unit_2.borrow ().get_statistic (UnitStatistic::SPL).0;
        assert! (mrl_unit_2_0 > mrl_unit_2_1);
        assert! (hlt_unit_2_0 > hlt_unit_2_1);
        assert! (spl_unit_2_0 > spl_unit_2_1);

        let mrl_unit_0_0 = unit_0.borrow ().get_statistic (UnitStatistic::MRL).0;
        let hlt_unit_0_0 = unit_0.borrow ().get_statistic (UnitStatistic::HLT).0;
        let spl_unit_0_0 = unit_0.borrow ().get_statistic (UnitStatistic::SPL).0;
        assert_eq! (unit_2.borrow ().act_attack (), 15);
        let mrl_unit_0_1 = unit_0.borrow ().get_statistic (UnitStatistic::MRL).0;
        let hlt_unit_0_1 = unit_0.borrow ().get_statistic (UnitStatistic::HLT).0;
        let spl_unit_0_1 = unit_0.borrow ().get_statistic (UnitStatistic::SPL).0;
        assert! (mrl_unit_0_0 > mrl_unit_0_1);
        assert! (hlt_unit_0_0 > hlt_unit_0_1);
        assert! (spl_unit_0_0 > spl_unit_0_1);

        assert_eq! (unit_2.borrow_mut ().act_switch_weapon (), 1);
        assert_eq! (unit_2.borrow ().act_attack (), 15);
        let mrl_unit_0_2 = unit_0.borrow ().get_statistic (UnitStatistic::MRL).0;
        let hlt_unit_0_2 = unit_0.borrow ().get_statistic (UnitStatistic::HLT).0;
        let spl_unit_0_2 = unit_0.borrow ().get_statistic (UnitStatistic::SPL).0;
        assert! (mrl_unit_0_1 > mrl_unit_0_2);
        assert! (hlt_unit_0_1 > hlt_unit_0_2);
        assert! (spl_unit_0_1 > spl_unit_0_2);
    }

    #[test]
    fn unit_act_skill () {
        let handler = generate_handler ();
        let (unit_0, _, unit_2) = generate_units (Rc::clone (&handler));
        let grid = generate_grid (Rc::downgrade (&handler));
        let (faction_0, faction_1) = generate_factions (Rc::clone (&handler));
        let grid_id = handler.borrow_mut ().register (Rc::clone (&grid) as Rc<RefCell<dyn Observer>>);
        let unit_0_id = handler.borrow_mut ().register (Rc::clone (&unit_0) as Rc<RefCell<dyn Observer>>);
        let unit_2_id = handler.borrow_mut ().register (Rc::clone (&unit_2) as Rc<RefCell<dyn Observer>>);
        let faction_0_id = handler.borrow_mut ().register (Rc::clone (&faction_0) as Rc<RefCell<dyn Observer>>);
        let faction_1_id = handler.borrow_mut ().register (Rc::clone (&faction_1) as Rc<RefCell<dyn Observer>>);

        handler.borrow_mut ().subscribe (grid_id, EVENT_GRID_FIND_UNITS);
        handler.borrow_mut ().subscribe (grid_id, EVENT_GRID_FIND_LOCATIONS);
        handler.borrow_mut ().subscribe (grid_id, EVENT_GRID_GET_UNIT_LOCATION);
        handler.borrow_mut ().subscribe (grid_id, EVENT_GRID_ADD_STATUS);
        handler.borrow_mut ().subscribe (grid_id, EVENT_GRID_TRY_YIELD_APPLIABLE);
        handler.borrow_mut ().subscribe (grid_id, EVENT_GRID_FIND_UNIT_CITIES);
        handler.borrow_mut ().subscribe (unit_0_id, EVENT_UNIT_GET_FACTION_ID);
        handler.borrow_mut ().subscribe (unit_2_id, EVENT_UNIT_GET_FACTION_ID);
        handler.borrow_mut ().subscribe (unit_0_id, EVENT_UNIT_TAKE_DAMAGE);
        handler.borrow_mut ().subscribe (unit_2_id, EVENT_UNIT_TAKE_DAMAGE);
        handler.borrow_mut ().subscribe (unit_0_id, EVENT_UNIT_ADD_STATUS);
        handler.borrow_mut ().subscribe (unit_2_id, EVENT_UNIT_ADD_STATUS);
        handler.borrow_mut ().subscribe (unit_0_id, EVENT_UNIT_ADD_APPLIABLE);
        handler.borrow_mut ().subscribe (unit_2_id, EVENT_UNIT_ADD_APPLIABLE);
        handler.borrow_mut ().subscribe (unit_0_id, EVENT_UNIT_GET_STATISTICS);
        handler.borrow_mut ().subscribe (unit_2_id, EVENT_UNIT_GET_STATISTICS);
        handler.borrow_mut ().subscribe (faction_0_id, EVENT_FACTION_IS_MEMBER);
        handler.borrow_mut ().subscribe (faction_1_id, EVENT_FACTION_IS_MEMBER);
        handler.borrow_mut ().subscribe (faction_0_id, EVENT_FACTION_ADD_MEMBER);
        handler.borrow_mut ().subscribe (faction_1_id, EVENT_FACTION_ADD_MEMBER);
        handler.borrow ().notify (Message::FactionAddMember (0, 0));
        handler.borrow ().notify (Message::FactionAddMember (0, 1));
        handler.borrow ().notify (Message::FactionAddMember (1, 2));
        grid.borrow_mut ().place_unit (0, (0, 1));
        grid.borrow_mut ().place_unit (2, (0, 0));

        // Test OnHit skill
        assert_eq! (unit_2.borrow ().act_skill (&0), 21);
        assert_eq! (unit_2.borrow ().statuses.borrow ().get (&Trigger::OnHit).unwrap ().len (), 1);
        assert_eq! (unit_2.borrow ().statuses.borrow ().get (&Trigger::OnHit).unwrap ()[0].get_id (), 5);
        unit_0.borrow ().act_attack ();
        assert_eq! (unit_0.borrow ().modifiers.borrow ().len (), 1);
        assert_eq! (unit_0.borrow ().modifiers.borrow ()[0].get_id (), 6);
        // Test OnOccupy skill
        assert_eq! (unit_0.borrow ().act_skill (&3), 21);
        let responses = handler.borrow ().notify (Message::GridTryYieldAppliable (0));
        let responses = Handler::reduce_responses (&responses);
        assert! (matches! (responses, Response::GridTryYieldAppliable (Some { .. })));
        let responses = handler.borrow ().notify (Message::GridTryYieldAppliable (2));
        let responses = Handler::reduce_responses (&responses);
        assert! (matches! (responses, Response::GridTryYieldAppliable (Some { .. })));
        grid.borrow_mut ().move_unit (2, vec![Direction::Down, Direction::Right]);
        unit_2.borrow_mut ().end_turn ();
        assert_eq! (unit_2.borrow ().modifiers.borrow ().len (), 1);
        assert_eq! (unit_0.borrow ().modifiers.borrow ()[0].get_id (), 6);
        // Test toggle skill
        assert_eq! (unit_0.borrow ().act_skill (&2), 21);
        assert_eq! (unit_0.borrow ().statuses.borrow ().get (&Trigger::None).unwrap ().len (), 1);
        assert_eq! (unit_0.borrow ().statuses.borrow ().get (&Trigger::None).unwrap ()[0].get_id (), 1);
        assert_eq! (unit_0.borrow ().modifiers.borrow ().len (), 2);
        assert_eq! (unit_0.borrow ().modifiers.borrow ()[1].get_id (), 5);
        assert_eq! (unit_0.borrow ().act_skill (&2), 21);
        assert_eq! (unit_0.borrow ().statuses.borrow ().get (&Trigger::None).unwrap ().len (), 1);
        assert_eq! (unit_0.borrow ().statuses.borrow ().get (&Trigger::None).unwrap ()[0].get_id (), 0);
        assert_eq! (unit_0.borrow ().modifiers.borrow ().len (), 2);
        assert_eq! (unit_0.borrow ().modifiers.borrow ()[1].get_id (), 3);
    }

    #[test]
    fn unit_act_magic () {
        let lists = generate_lists ();
        let handler = generate_handler ();
        let (unit_0, unit_1, unit_2) = generate_units (Rc::clone (&handler));
        let grid = generate_grid (Rc::downgrade (&handler));
        let (faction_0, faction_1) = generate_factions (Rc::clone (&handler));
        let grid_id = handler.borrow_mut ().register (Rc::clone (&grid) as Rc<RefCell<dyn Observer>>);
        let unit_0_id = handler.borrow_mut ().register (Rc::clone (&unit_0) as Rc<RefCell<dyn Observer>>);
        let unit_1_id = handler.borrow_mut ().register (Rc::clone (&unit_1) as Rc<RefCell<dyn Observer>>);
        let unit_2_id = handler.borrow_mut ().register (Rc::clone (&unit_2) as Rc<RefCell<dyn Observer>>);
        let faction_0_id = handler.borrow_mut ().register (Rc::clone (&faction_0) as Rc<RefCell<dyn Observer>>);
        let faction_1_id = handler.borrow_mut ().register (Rc::clone (&faction_1) as Rc<RefCell<dyn Observer>>);

        handler.borrow_mut ().subscribe (grid_id, EVENT_GRID_FIND_UNITS);
        handler.borrow_mut ().subscribe (unit_0_id, EVENT_UNIT_GET_FACTION_ID);
        handler.borrow_mut ().subscribe (unit_1_id, EVENT_UNIT_GET_FACTION_ID);
        handler.borrow_mut ().subscribe (unit_2_id, EVENT_UNIT_GET_FACTION_ID);
        handler.borrow_mut ().subscribe (unit_0_id, EVENT_UNIT_TAKE_DAMAGE);
        handler.borrow_mut ().subscribe (unit_1_id, EVENT_UNIT_TAKE_DAMAGE);
        handler.borrow_mut ().subscribe (unit_2_id, EVENT_UNIT_TAKE_DAMAGE);
        handler.borrow_mut ().subscribe (unit_0_id, EVENT_UNIT_ADD_STATUS);
        handler.borrow_mut ().subscribe (unit_1_id, EVENT_UNIT_ADD_STATUS);
        handler.borrow_mut ().subscribe (unit_2_id, EVENT_UNIT_ADD_STATUS);
        handler.borrow_mut ().subscribe (unit_0_id, EVENT_UNIT_ADD_APPLIABLE);
        handler.borrow_mut ().subscribe (unit_1_id, EVENT_UNIT_ADD_APPLIABLE);
        handler.borrow_mut ().subscribe (unit_2_id, EVENT_UNIT_ADD_APPLIABLE);
        handler.borrow_mut ().subscribe (unit_0_id, EVENT_UNIT_GET_STATISTICS);
        handler.borrow_mut ().subscribe (unit_1_id, EVENT_UNIT_GET_STATISTICS);
        handler.borrow_mut ().subscribe (unit_2_id, EVENT_UNIT_GET_STATISTICS);
        handler.borrow_mut ().subscribe (faction_0_id, EVENT_FACTION_IS_MEMBER);
        handler.borrow_mut ().subscribe (faction_1_id, EVENT_FACTION_IS_MEMBER);
        handler.borrow_mut ().subscribe (faction_0_id, EVENT_FACTION_ADD_MEMBER);
        handler.borrow_mut ().subscribe (faction_1_id, EVENT_FACTION_ADD_MEMBER);
        handler.borrow ().notify (Message::FactionAddMember (0, 0));
        handler.borrow ().notify (Message::FactionAddMember (0, 1));
        handler.borrow ().notify (Message::FactionAddMember (1, 2));
        grid.borrow_mut ().place_unit (0, (0, 1));
        grid.borrow_mut ().place_unit (2, (0, 0));

        // Test normal magic
        let atk_unit_0_0 = unit_0.borrow ().get_statistic (UnitStatistic::ATK).0;
        let hlt_unit_0_0 = unit_0.borrow ().get_statistic (UnitStatistic::HLT).0;
        let org_unit_0_0 = unit_0.borrow ().get_statistic (UnitStatistic::ORG).0;
        assert_eq! (unit_0.borrow ().act_magic (&1), 21);
        let atk_unit_0_1 = unit_0.borrow ().get_statistic (UnitStatistic::ATK).0;
        let hlt_unit_0_1 = unit_0.borrow ().get_statistic (UnitStatistic::HLT).0;
        let org_unit_0_1 = unit_0.borrow ().get_statistic (UnitStatistic::ORG).0;
        assert! (atk_unit_0_0 < atk_unit_0_1);
        assert! (hlt_unit_0_0 > hlt_unit_0_1);
        assert! (org_unit_0_0 > org_unit_0_1);
        assert_eq! (unit_0.borrow ().statuses.borrow ().get (&Trigger::None).unwrap ().len (), 1);
        assert_eq! (unit_0.borrow ().modifiers.borrow ().len (), 1);
        // Test OnAttack magic
        assert_eq! (unit_0.borrow ().act_magic (&2), 21);
        let hlt_unit_0_2 = unit_0.borrow ().get_statistic (UnitStatistic::HLT).0;
        let org_unit_0_2 = unit_0.borrow ().get_statistic (UnitStatistic::ORG).0;
        assert! (hlt_unit_0_1 > hlt_unit_0_2);
        assert! (org_unit_0_1 > org_unit_0_2);
        assert! (matches! (unit_0.borrow ().weapons[unit_0.borrow ().weapon_active].try_yield_appliable (Rc::clone (&lists)), Some { .. }));
        unit_0.borrow ().act_attack ();
        assert_eq! (unit_2.borrow ().modifiers.borrow ().len (), 1);
        assert_eq! (unit_2.borrow ().modifiers.borrow ()[0].get_id (), 6);
    }

    #[test]
    fn unit_add_appliable () {
        let handler = generate_handler ();
        let (unit_0, unit_1, _) = generate_units (Rc::clone (&handler));
        let (modifier_3, modifier_4) = generate_modifiers ();
        let (effect_0, effect_1) = generate_effects ();

        // Test additive modifier
        assert_eq! (unit_0.borrow ().add_appliable (modifier_3.clone ()), true);
        assert_eq! (unit_0.borrow ().modifiers.borrow ().len (), 1);
        assert_eq! (unit_0.borrow ().get_statistic (UnitStatistic::ATK).0, 22);
        // Test subtractive modifier
        assert_eq! (unit_0.borrow ().add_appliable (modifier_4.clone ()), true); // Test multiple adjustments
        assert_eq! (unit_0.borrow ().modifiers.borrow ().len (), 2);
        assert_eq! (unit_0.borrow ().get_statistic (UnitStatistic::ATK).0, 20);
        assert_eq! (unit_0.borrow ().get_statistic (UnitStatistic::DEF).0, 18);
        // Test stacking modifier
        assert_eq! (unit_0.borrow ().add_appliable (modifier_3.clone ()), true);
        assert_eq! (unit_0.borrow ().modifiers.borrow ().len (), 3);
        assert_eq! (unit_0.borrow ().get_statistic (UnitStatistic::ATK).0, 22);
        assert_eq! (unit_0.borrow ().add_appliable (modifier_3), true);
        assert_eq! (unit_0.borrow ().modifiers.borrow ().len (), 4);
        assert_eq! (unit_0.borrow ().get_statistic (UnitStatistic::ATK).0, 24);
        // Test non-stacking modifier
        assert_eq! (unit_0.borrow ().add_appliable (modifier_4), false);
        assert_eq! (unit_0.borrow ().modifiers.borrow ().len (), 4);
        assert_eq! (unit_0.borrow ().get_statistic (UnitStatistic::ATK).0, 24);

        // Test flat effect
        assert_eq! (unit_1.borrow ().add_appliable (effect_0), true);
        assert_eq! (unit_1.borrow ().get_statistic (UnitStatistic::HLT).0, 998);
        // Test percentage effect
        assert_eq! (unit_1.borrow ().add_appliable (effect_1), true); // Test multiple adjustments
        assert_eq! (unit_1.borrow ().get_statistic (UnitStatistic::ATK).0, 21);
        assert_eq! (unit_1.borrow ().get_statistic (UnitStatistic::DEF).0, 19);
    }

    #[test]
    fn unit_add_status () {
        let handler = generate_handler ();
        let lists = generate_lists ();
        let (unit_0, _, _) = generate_units (Rc::clone (&handler));
        let (status_0, _, status_5) = generate_statuses ();
        let status_6 = lists.get_status (&6).clone ();

        // Test unit status
        assert_eq! (unit_0.borrow ().add_status (status_0), true);
        assert_eq! (unit_0.borrow ().get_statistic (UnitStatistic::ATK).0, 22);
        // Test applier status
        assert_eq! (unit_0.borrow ().add_status (status_5), true);
        assert_eq! (unit_0.borrow ().statuses.borrow ().get (&Trigger::OnHit).unwrap ().len (), 1);
        assert! (matches! (unit_0.borrow ().try_yield_appliable (Rc::clone (&unit_0.borrow ().lists)), Some { .. }));
        // Test weapon status
        assert_eq! (unit_0.borrow ().add_status (status_6), true);
        assert! (matches! (unit_0.borrow ().weapons[unit_0.borrow ().weapon_active].try_yield_appliable (lists), Some { .. }));
    }

    #[test]
    fn unit_remove_modifier () {
        let handler = generate_handler ();
        let (unit_0, _, _) = generate_units (Rc::clone (&handler));
        let (modifier_3, _) = generate_modifiers ();

        // Test empty remove
        assert_eq! (unit_0.borrow ().remove_modifier (&3), false);
        assert_eq! (unit_0.borrow ().modifiers.borrow ().len (), 0);
        // Test non-empty remove
        unit_0.borrow ().add_appliable (modifier_3);
        assert_eq! (unit_0.borrow ().remove_modifier (&3), true);
        assert_eq! (unit_0.borrow ().modifiers.borrow ().len (), 0);
    }

    #[test]
    fn unit_remove_status () {
        let handler = generate_handler ();
        let (unit_0, _, _) = generate_units (Rc::clone (&handler));
        let (status_0, _, status_5) = generate_statuses ();

        // Test empty remove
        assert_eq! (unit_0.borrow ().remove_status (&0), false);
        assert_eq! (unit_0.borrow ().statuses.borrow ().get (&Trigger::None).unwrap ().len (), 0);
        assert_eq! (unit_0.borrow ().modifiers.borrow ().len (), 0);
        // Test non-empty remove
        unit_0.borrow ().add_status (status_0);
        assert_eq! (unit_0.borrow ().remove_status (&0), true);
        assert_eq! (unit_0.borrow ().statuses.borrow ().get (&Trigger::None).unwrap ().len (), 0);
        assert_eq! (unit_0.borrow ().modifiers.borrow ().len (), 0);
        // Test applier remove
        unit_0.borrow ().add_status (status_5);
        assert_eq! (unit_0.borrow ().remove_status (&5), true);
        assert_eq! (unit_0.borrow ().statuses.borrow ().get (&Trigger::OnHit).unwrap ().len (), 0);
        assert_eq! (unit_0.borrow ().modifiers.borrow ().len (), 0);
    }

    #[test]
    fn unit_dec_durations () {
        let handler = generate_handler ();
        let (unit_0, unit_1, _) = generate_units (Rc::clone (&handler));
        let (modifier_3, modifier_4) = generate_modifiers ();
        let (status_0, status_1, status_5) = generate_statuses ();

        // Test empty modifier
        unit_0.borrow_mut ().dec_durations ();
        assert_eq! (unit_0.borrow ().modifiers.borrow ().len (), 0);
        // Test timed modifier
        unit_0.borrow ().add_appliable (modifier_3.clone ());
        unit_0.borrow_mut ().dec_durations ();
        assert_eq! (unit_0.borrow ().modifiers.borrow ().len (), 1);
        unit_0.borrow_mut ().dec_durations ();
        assert_eq! (unit_0.borrow ().modifiers.borrow ().len (), 1);
        unit_0.borrow_mut ().dec_durations ();
        assert_eq! (unit_0.borrow ().modifiers.borrow ().len (), 0);
        // Test permanent modifier
        unit_0.borrow ().add_appliable (modifier_4);
        unit_0.borrow_mut ().dec_durations ();
        assert_eq! (unit_0.borrow ().modifiers.borrow ().len (), 1);
        unit_0.borrow_mut ().dec_durations ();
        assert_eq! (unit_0.borrow ().modifiers.borrow ().len (), 1);

        // Test empty status
        unit_1.borrow_mut ().dec_durations ();
        assert_eq! (unit_1.borrow ().statuses.borrow ().get (&Trigger::None).unwrap ().len (), 0);
        // Test timed status
        unit_1.borrow ().add_status (status_1);
        unit_1.borrow_mut ().dec_durations ();
        assert_eq! (unit_1.borrow ().statuses.borrow ().get (&Trigger::None).unwrap ().len (), 1);
        unit_1.borrow_mut ().dec_durations ();
        assert_eq! (unit_1.borrow ().statuses.borrow ().get (&Trigger::None).unwrap ().len (), 0);
        // Test permanent status
        unit_1.borrow ().add_status (status_0);
        unit_1.borrow_mut ().dec_durations ();
        assert_eq! (unit_1.borrow ().statuses.borrow ().get (&Trigger::None).unwrap ().len (), 1);
        unit_1.borrow_mut ().dec_durations ();
        assert_eq! (unit_1.borrow ().statuses.borrow ().get (&Trigger::None).unwrap ().len (), 1);
        // Test linked status
        unit_1.borrow ().add_status (status_5);
        unit_1.borrow_mut ().dec_durations ();
        assert_eq! (unit_1.borrow ().statuses.borrow ().get (&Trigger::OnHit).unwrap ().len (), 1);
        assert_eq! (unit_1.borrow ().statuses.borrow ().get (&Trigger::OnHit).unwrap ()[0].get_next_id ().unwrap (), 0);
        unit_1.borrow_mut ().dec_durations ();
        assert_eq! (unit_1.borrow ().statuses.borrow ().get (&Trigger::OnHit).unwrap ().len (), 1);
        assert_eq! (unit_1.borrow ().statuses.borrow ().get (&Trigger::OnHit).unwrap ()[0].get_next_id (), None);
    }
}
