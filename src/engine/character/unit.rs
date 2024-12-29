use super::*;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::{Rc, Weak};
use crate::engine::{event::Handler, Lists};
use crate::engine::common::{Area, Capacity, ID, ID_UNINITIALISED, Target, Timed};
use crate::engine::event::{Message, Observer, Response, Subject};
use crate::engine::map::{Direction, Location};
use crate::engine::dynamic::{Appliable, Applier, Change, Changeable, Effect, Modifier, StatisticType, Status, Trigger};

const MRL_MAX: u16 = 1000; // 100.0%
const HLT_MAX: u16 = 1000; // 100.0%
const SPL_MAX: u16 = 1000; // 1000
const ATK_MAX: u16 = 200; // 200
const DEF_MAX: u16 = 200; // 200
const MAG_MAX: u16 = 200; // 200
const MOV_MAX: u16 = 100; // 100
const ORG_MAX: u16 = 2000; // 200.0%
const DRAIN_SPL: u16 = 50; // 5.0%
const RECOVER_MRL: u16 = 10; // 1.0%
const WEAPON_0: usize = 0;
const WEAPON_1: usize = 1;
/*
 * Calculated from build.rs
 * Unit MOV is an index into the table
 * Attack (* 1.0): 21 delay at 0, 20 delay at 1, 2 delay at 77, and 1 delay at 100
 * Magic/skill (* 1.4): 29 delay at 0, 28 delay at 1, 2 delay at 77, and 1 delay at 100
 * Wait (* 0.67): 14 delay at 0, 13 delay at 1, 2 delay at 54, and 1 delay at 77
 */
const DELAYS: [u8; 101] = [21, 20, 19, 19, 18, 18, 17, 17, 16, 16, 15, 15, 14, 14, 14, 13, 13, 13, 12, 12, 11, 11, 11, 11, 10, 10, 10, 9, 9, 9, 9, 8, 8, 8, 8, 7, 7, 7, 7, 7, 7, 6, 6, 6, 6, 6, 6, 5, 5, 5, 5, 5, 5, 5, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 1];
const DELAY_APPLIER: f32 = 1.4;
const DELAY_WAIT: f32 = 0.67;

fn get_delay (mov: u16, action: Action) -> u8 {
    assert! ((mov as usize) < DELAYS.len ());

    let delay: u8 = DELAYS[mov as usize];

    match action {
        Action::Attack => delay,
        Action::Wait => ((delay as f32) * DELAY_WAIT) as u8,
        _ => ((delay as f32) * DELAY_APPLIER) as u8,
    }
}

#[derive (Debug)]
#[derive (Clone, Copy)]
pub enum UnitStatistic {
    MRL, // morale - willingness to fight (percentage)
    HLT, // manpower - number of soldiers
    SPL, // supply - proportion of soldiers equipped (percentage)
    ATK, // attack – physical damage
    DEF, // defence – physical resistance
    MAG, // magic – magical damage and resistance
    MOV, // manoeuvre – speed and movement
    ORG, // cohesion – modifier for formation effects and subordinate units (percentage)
    Length,
}

#[derive (Debug)]
#[derive (Clone, Copy)]
pub enum Action {
    Attack,
    Skill (ID),
    Magic (ID),
    Wait,
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
            UnitStatistic::MRL => if let Capacity::Quantity (c, m) = self.0[UnitStatistic::MRL as usize] {
                true
            } else {
                false
            }
            UnitStatistic::HLT => if let Capacity::Quantity (c, m) = self.0[UnitStatistic::HLT as usize] {
                true
            } else {
                false
            }
            UnitStatistic::SPL => if let Capacity::Quantity (c, m) = self.0[UnitStatistic::SPL as usize] {
                true
            } else {
                false
            }
            UnitStatistic::ATK => if let Capacity::Constant (c, _, b) = self.0[UnitStatistic::ATK as usize] {
                true
            } else {
                false
            }
            UnitStatistic::DEF => if let Capacity::Constant (c, _, b) = self.0[UnitStatistic::DEF as usize] {
                true
            } else {
                false
            }
            UnitStatistic::MAG => if let Capacity::Constant (c, _, b) = self.0[UnitStatistic::MAG as usize] {
                true
            } else {
                false
            }
            UnitStatistic::MOV => if let Capacity::Constant (c, _, b) = self.0[UnitStatistic::MOV as usize] {
                true
            } else {
                false
            }
            UnitStatistic::ORG => if let Capacity::Quantity (c, m) = self.0[UnitStatistic::ORG as usize] {
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

impl Default for UnitStatistics {
    fn default () -> Self {
        let mrl: Capacity = Capacity::Quantity (0, MRL_MAX);
        let hlt: Capacity = Capacity::Quantity (0, HLT_MAX);
        let spl: Capacity = Capacity::Quantity (0, SPL_MAX);
        let atk: Capacity = Capacity::Constant (0, ATK_MAX, 0);
        let def: Capacity = Capacity::Constant (0, DEF_MAX, 0);
        let mag: Capacity = Capacity::Constant (0, MAG_MAX, 0);
        let mov: Capacity = Capacity::Constant (0, MOV_MAX, 0);
        let org: Capacity = Capacity::Quantity (0, ORG_MAX);
        let statistics: [Capacity; UnitStatistic::Length as usize] = [mrl, hlt, spl, atk, def, mag, mov, org];

        Self (statistics)
    }
}

#[derive (Debug)]
pub struct Unit {
    id: ID,
    lists: Rc<Lists>,
    // Safety guarante: Only Unit can reference its own statistics
    statistics: Cell<UnitStatistics>,
    modifiers: Vec<Modifier>,
    statuses: HashMap<Trigger, Vec<Status>>,
    magic_ids: Vec<ID>,
    weapons: [Weapon; 2],
    weapon_active: usize,
    faction_id: ID,
    // Safety guarantee: Unit will never borrow_mut Handler
    handler: Weak<RefCell<Handler>>,
    observer_id: Cell<ID>,
}

impl Unit {
    pub fn new (id: ID, lists: Rc<Lists>, statistics: UnitStatistics, magics_usable: [bool; 3], weapon_ids: [ID; 2], faction_id: ID, handler: Weak<RefCell<Handler>>) -> Self {
        let statistics: Cell<UnitStatistics> = Cell::new (statistics);
        let modifiers: Vec<Modifier> = Vec::new ();
        let magic_ids: Vec<ID> = Vec::new ();
        let mut statuses: HashMap<Trigger, Vec<Status>> = HashMap::new ();
        let weapons: [Weapon; 2] = [
            lists.get_weapon (&weapon_ids[WEAPON_0]).clone (),
            lists.get_weapon (&weapon_ids[WEAPON_1]).clone (),
        ];
        let weapon_active: usize = WEAPON_0;
        let observer_id: Cell<ID> = Cell::new (ID_UNINITIALISED);

        statuses.insert (Trigger::None, Vec::new ());

        Self { id, lists, statistics, modifiers, statuses, magic_ids, weapons, weapon_active, faction_id, handler, observer_id }
    }

    pub fn initialise (&self) -> () {
        self.notify (Message::FactionAddMember (self.faction_id, self.id));
    }

    fn get_statistic (&self, statistic: UnitStatistic) -> (u16, u16) {
        self.statistics.get ().get_statistic (statistic)
    }

    fn change_statistic_flat (&self, statistic: UnitStatistic, change: u16, is_add: bool) -> () {
        let mut statistics: UnitStatistics = self.statistics.take ();

        statistics.change_statistic_flat (statistic, change, is_add);
        self.statistics.replace (statistics);

        if self.get_statistic (UnitStatistic::HLT).0 == 0 {
            self.die ();
        }
    }

    fn change_statistic_percentage (&self, statistic: UnitStatistic, change: u16, is_add: bool) -> () {
        let mut statistics: UnitStatistics = self.statistics.take ();

        statistics.change_statistic_percentage (statistic, change, is_add);
        self.statistics.replace (statistics);

        if self.get_statistic (UnitStatistic::HLT).0 == 0 {
            self.die ();
        }
    }

    pub fn calculate_damage (&self, statistics_other: &UnitStatistics) -> (u16, u16) {
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

        let spl_other: (u16, u16) = statistics_other.get_statistic (UnitStatistic::SPL);
        let def_other: u16 = statistics_other.get_statistic (UnitStatistic::DEF).0;
        let mag_other: u16 = statistics_other.get_statistic (UnitStatistic::MAG).0;

        let damage_weapon: u16 = {
            let atk: u16 = atk_self + dmg_weapon;
            let multiplier_def: f32 = (spl_other.0 as f32) / (spl_other.1 as f32);
            let def: u16 = ((def_other as f32) * multiplier_def) as u16;
            let damage: u16 = u16::max (atk.checked_sub (def).unwrap_or (1), 1);
            let multiplier_damage: f32 = (spl_self.0 as f32) / (spl_self.1 as f32);

            ((damage as f32) * multiplier_damage) as u16
        };
        let damage_bonus: u16 = {
            let damage: u16 = u16::max (mag_self.checked_sub (mag_other).unwrap_or (1), 1);
            let multiplier: u16 = (dcy_weapon * 2) + 1;

            damage * multiplier
        };
        let multiplier: f32 = {
            let multiplier_hlt: f32 = (hlt_self.0 as f32) / (hlt_self.1 as f32);
            let multiplier_org: f32 = (org_self as f32) / 100.0;

            multiplier_hlt * multiplier_org
        };
        let damage_base: u16 = (((damage_weapon + damage_bonus) as f32) * multiplier) as u16;
        let damage_mrl: u16 = damage_base + slh_weapon;
        let damage_hlt: u16 = damage_base * prc_weapon;

        (damage_mrl, damage_hlt)
    }

    fn take_damage (&self, damage_mrl: u16, damage_hlt: u16) -> () {
        let damage_spl: u16 = (damage_mrl + damage_hlt) / 2;

        self.change_statistic_flat (UnitStatistic::MRL, damage_mrl, false);
        self.change_statistic_flat (UnitStatistic::HLT, damage_hlt, false);
        self.change_statistic_flat (UnitStatistic::SPL, damage_spl, false);
    }

    fn die (&self) -> () {
        self.notify (Message::GameUnitDie (self.id));

        // TODO: ???
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
                if let Area::Radial (_) = area {
                    loop {
                        // TODO:: Prompt user to choose ONE centre
                        // target_id = ???;

                        let target_ids: Vec<Response> = self.notify (Message::GridFindNearbyUnits (target_id, None, area, range));
                        let target_ids: Vec<ID> = if let Response::GridFindNearbyUnits (t) = Handler::reduce_responses (&target_ids) {
                            if let Target::Allies = target {
                                t.iter ().filter_map (|u: &ID| {
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
                            } else if let Target::Enemies = target {
                                t.iter ().filter_map (|u: &ID| {
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
                    
                } else {
                    potential_ids.clone ()
                }
            }
            _ => panic! ("Invalid target {:?}", target),
        };

        // TODO: Prompt user to confirm
        target_ids
    }

    fn find_targets_units (&self, direction: Option<Direction>, target: Target, area: Area, range: u8) -> Vec<ID> {
        let potential_ids: Vec<ID> = if let Target::Map = target {
            panic! ("Invalid target {:?}", target)
        } else if let Target::This = target {
            vec![self.id]
        } else {
            let neighbour_ids: Vec<Response> = self.notify (Message::GridFindNearbyUnits (self.id, direction, area, range));
            let neighbour_ids: &Vec<ID> = if let Response::GridFindNearbyUnits (n) = Handler::reduce_responses (&neighbour_ids) {
                n
            } else {
                panic! ("Invalid response")
            };

            match target {
                Target::Ally | Target::Allies => {
                    neighbour_ids.iter ().filter_map (|u: &ID| {
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
                }
                Target::Enemy | Target::Enemies => {
                    neighbour_ids.iter ().filter_map (|u: &ID| {
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
                _ => panic! ("Invalid target {:?}", target),
            }
        };

        self.choose_targets_units (&potential_ids, target, area, range)
    }

    fn choose_targets_locations (&self, potential_locations: &Vec<Location>, area: Area, range: u8) -> Vec<Location> {
        assert! (potential_locations.len () > 0);

        let mut target_location: Location = potential_locations[0];

        if let Area::Radial (_) = area {
            loop {
                // TODO:: Prompt user to choose ONE centre
                // target_id = ???;

                let target_locations: Vec<Response> = self.notify (Message::GridFindNearbyLocations (target_location, None, area, range));
                let target_locations: &Vec<Location> = if let Response::GridFindNearbyLocations (t) = Handler::reduce_responses (&target_locations) {
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
            
        } else {
            loop {
                // TODO: Prompt user to choose ONE
                // target_id = ???;

                // TODO: Prompt user for confirmation
                if true {
                    break vec![target_location]
                // TODO: User rejected choice
                } else if 1 > 0 {
                    break Vec::new ()
                }
                // TODO: else -> user made another choice
            }
        }
    }

    fn find_targets_locations (&self, direction: Option<Direction>, target: Target, area: Area, range: u8) -> Vec<Location> {
        if let Target::Map = target {
            let location: Vec<Response> = self.notify (Message::GridGetUnitLocation (self.id));
            let location: Location = if let Response::GridGetUnitLocation (l) = Handler::reduce_responses (&location) {
                *l
            } else {
                panic! ("Invalid response")
            };
            let neighbour_locations: Vec<Response> = self.notify (Message::GridFindNearbyLocations (location, direction, area, range));
            let neighbour_locations: &Vec<Location> = if let Response::GridFindNearbyLocations (n) = Handler::reduce_responses (&neighbour_locations) {
                n
            } else {
                panic! ("Invalid response")
            };

            self.choose_targets_locations (neighbour_locations, area, range)
        } else {
            panic! ("Invalid target {:?}", target)
        }
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

    pub fn switch_weapon (&mut self) -> ID {
        self.weapon_active = if self.weapon_active == WEAPON_0 {
            WEAPON_1
        } else {
            WEAPON_0
        };

        self.weapon_active
    }

    pub fn act (&mut self, action: Action) -> u8 {
        let mov: u16 = self.get_statistic (UnitStatistic::MOV).0;

        match action {
            Action::Attack => {
                let weapon: &Weapon = &self.weapons[self.weapon_active];
                
                // let targets: Vec<ID> = self.find_targets (location, direction, target, area, range);
                // let (damage_mrl, damage_hlt): (u16, u16) = self.calculate_damage (other);

                // other.take_damage (damage_mrl, damage_hlt)
                // let damage_self = other.calculate_damage (self);
                // let damage_spl = (damage_self.0 + damage_self.1) / 2;
                // self.change_statistic_flat (UnitStatistic::SPL, damage_spl, false);
            }
            Action::Magic (m) => {
                let magic: &Magic = self.lists.get_magic (&m);
            }
            Action::Skill (s) => {
                let skill: &Skill = self.lists.get_skill (&s);
            }
            Action::Wait => (),
        }

        get_delay (mov, action)
    }

    pub fn act_magic (&mut self, magic_id: ID) -> () {
        let magic: &Magic = self.lists.get_magic (&magic_id);

        todo! ()
    }

    pub fn act_skill (&mut self, skill_id: ID) -> () {
        let skill: &Skill = self.lists.get_skill (&skill_id);

        // Skills always target allies
        // if let Target::Ally (s) = target {
        //     if s {
        //         self.add_appliable (modifier.clone ());
        //     } else {
                
        //     }
        // } else if let Target::All (true) = target {
        //     let unit_ids: Vec<ID> = self.find_targets (target, skill.get_area (), skill.get_range ());
            
        //     for unit_id in unit_ids {
        //         // TODO: Get units from somewhere???
        //     }
        // } else {
        //     panic! ("Invalid target {:?}", target)
        // }

        todo! ()
    }

    pub fn end_turn (&mut self) -> () {
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

            self.change_statistic_flat (UnitStatistic::MRL, RECOVER_MRL, true);
            self.change_statistic_flat (UnitStatistic::HLT, recover_hlt, true);
            self.change_statistic_flat (UnitStatistic::SPL, recover_spl, false);
        } else {
            self.change_statistic_flat (UnitStatistic::SPL, DRAIN_SPL, false);
        }

        self.dec_durations ();
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
            Message::UnitTakeDamage (u, m, h) => if u == self.id {
                let appliable: Option<Box<dyn Appliable>> = self.try_yield_appliable (Rc::clone (&self.lists));
                let appliable: Option<Change> = appliable.map (|a: Box<dyn Appliable>| a.get_change ());

                self.take_damage (m, h);

                Some (Response::UnitTakeDamage (appliable))
            } else {
                None
            }
            Message::UnitAddStatus (u, s) => if u == self.id {
                // let status: Status = self.lists.get_status (&s).clone ();
                // self.add_status (status);
                todo!()
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
    fn try_yield_appliable (&self, lists: Rc<Lists>) -> Option<Box<dyn Appliable>> {
        self.statuses.get (&Trigger::OnHit).and_then (|c: &Vec<Status>| {
            c.get (0).and_then (|s: &Status| s.try_yield_appliable (Rc::clone (&self.lists)) )
        })
    }

    fn get_target (&self) -> Option<Target> {
        self.statuses.get (&Trigger::OnHit).and_then (|c: &Vec<Status>| {
            c.get (0).and_then (|s: &Status| s.get_target () )
        })
    }
}

impl Changeable for Unit {
    fn add_appliable (&mut self, appliable: Box<dyn Appliable>) -> bool {
        let change: Change = appliable.get_change ();

        match change {
            Change::Modifier (_, _) => {
                let modifier: Modifier = appliable.modifier ();

                if modifier.can_stack () || !self.modifiers.contains (&modifier){
                    for adjustment in modifier.get_adjustments () {
                        if let Some (a) = adjustment {
                            if let StatisticType::Unit (s) = a.0 {
                                self.change_statistic_percentage (s, a.1, a.2);
                            }
                        }
                    }

                    self.modifiers.push (modifier);

                    true
                } else {
                    false
                }
            }
            Change::Effect (_) => {
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

    fn add_status (&mut self, status: Status) -> bool {
        let trigger: Trigger = status.get_trigger ();

        if let Trigger::OnOccupy = trigger {
            false
        } else {
            let appliable: Box<dyn Appliable> = status.try_yield_appliable (Rc::clone (&self.lists))
                    .expect (&format! ("Appliable not found for status {:?}", status));
            let target: Target = status.get_target ()
                    .expect (&format! ("Target not found for status {:?}", status));

            match trigger {
                Trigger::OnAttack => {
                    let weapon: &mut Weapon = &mut self.weapons[self.weapon_active];

                    weapon.add_status (status);

                    true
                }
                Trigger::OnHit => {
                    let mut collection: Vec<Status> = Vec::new ();

                    collection.push (status);
                    self.statuses.insert (trigger, collection);

                    true
                }
                Trigger::None => {
                    if let Target::This = target {
                        let collection: &mut Vec<Status> = self.statuses.get_mut (&trigger)
                                .expect (&format! ("Collection not found for trigger {:?}", trigger));

                        collection.push (status);
                        self.add_appliable (appliable);

                        true
                    } else {
                        false
                    }
                }
                _ => false,
            }
        }
    }

    fn dec_durations (&mut self) -> () {
        self.modifiers.retain_mut (|m: &mut Modifier| !m.dec_duration ());


        for (trigger, collection) in self.statuses.iter_mut () {
            for status in collection.iter_mut () {
                status.dec_duration ();
            }

            let nexts: Vec<ID> = collection.iter ().filter_map (|s: &Status|
                if s.get_duration () == 0 {
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
    magics_usable: [bool; 3],
    weapon_ids: [ID; 2],
    faction_id: ID,
}

impl UnitBuilder {
    pub const fn new (id: ID, statistics: UnitStatistics, magics_usable: [bool; 3], weapon_ids: [ID; 2], faction_id: ID) -> Self {
        Self { id, statistics, magics_usable, weapon_ids, faction_id  }
    }

    pub fn build (&self, lists: Rc<Lists>, handler: Weak<RefCell<Handler>>) -> Unit {
        Unit::new (self.id, lists, self.statistics, self.magics_usable, self.weapon_ids, self.faction_id, handler)
    }
}

#[cfg (test)]
pub mod tests {
    use super::*;
    use crate::engine::common::DURATION_PERMANENT;
    use crate::engine::event::{EVENT_FACTION_ADD_MEMBER, EVENT_FACTION_IS_MEMBER, EVENT_GRID_FIND_NEARBY_UNITS, EVENT_UNIT_GET_FACTION_ID};
    use crate::engine::map::grid::tests::generate_grid;
    use crate::engine::tests::generate_lists;
    use crate::engine::event::handler::tests::generate_handler;

    pub fn generate_units (handler: Rc<RefCell<Handler>>) -> (Rc<RefCell<Unit>>, Rc<RefCell<Unit>>, Rc<RefCell<Unit>>) {
        let lists: Rc<Lists> = generate_lists ();
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
        let lists: Rc<Lists> = generate_lists ();
        let modifier_builder_3 = lists.get_modifier_builder (&3);
        let modifier_3 = modifier_builder_3.build (2, true);
        let modifier_3 = Box::new (modifier_3);
        let modifier_builder_4 = lists.get_modifier_builder (&4);
        let modifier_4 = modifier_builder_4.build (DURATION_PERMANENT, false);
        let modifier_4 = Box::new (modifier_4);

        (modifier_3, modifier_4)
    }

    fn generate_effects () -> (Box<Effect>, Box<Effect>) {
        let lists: Rc<Lists> = generate_lists ();
        let effect_0 = lists.get_effect (&0).clone ();
        let effect_0 = Box::new (effect_0);
        let effect_1 = lists.get_effect (&1).clone ();
        let effect_1 = Box::new (effect_1);

        (effect_0, effect_1)
    }

    fn generate_statuses () -> (Status, Status, Status) {
        let lists: Rc<Lists> = generate_lists ();
        let status_0 = lists.get_status (&0).clone ();
        let status_1 = lists.get_status (&1).clone ();
        let status_5 = lists.get_status (&5).clone ();

        (status_0, status_1, status_5)
    }

    fn generate_factions (handler: Rc<RefCell<Handler>>) -> (Rc<RefCell<Faction>>, Rc<RefCell<Faction>>) {
        let lists: Rc<Lists> = generate_lists ();
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

        handler.borrow_mut ().subscribe (grid_id, EVENT_GRID_FIND_NEARBY_UNITS);
        handler.borrow_mut ().subscribe (unit_0_id, EVENT_UNIT_GET_FACTION_ID);
        handler.borrow_mut ().subscribe (unit_1_id, EVENT_UNIT_GET_FACTION_ID);
        handler.borrow_mut ().subscribe (unit_2_id, EVENT_UNIT_GET_FACTION_ID);
        handler.borrow_mut ().subscribe (faction_0_id, EVENT_FACTION_IS_MEMBER);
        handler.borrow_mut ().subscribe (faction_1_id, EVENT_FACTION_IS_MEMBER);
        handler.borrow_mut ().subscribe (faction_0_id, EVENT_FACTION_ADD_MEMBER);
        handler.borrow_mut ().subscribe (faction_1_id, EVENT_FACTION_ADD_MEMBER);
        grid.borrow_mut ().place_unit (0, (0, 0));
        grid.borrow_mut ().place_unit (1, (0, 1));
        grid.borrow_mut ().place_unit (2, (1, 0));
        handler.borrow ().notify (Message::FactionAddMember (0, 0));
        handler.borrow ().notify (Message::FactionAddMember (0, 1));
        handler.borrow ().notify (Message::FactionAddMember (1, 2));

        let results: Vec<ID> = unit_0.borrow ().choose_targets_units (&vec![0], Target::This, Area::Single, 1);
        assert_eq! (results.len (), 1);
        assert_eq! (results[0], 0);
        let results: Vec<ID> = unit_0.borrow ().choose_targets_units (&vec![1], Target::Ally, Area::Single, 1);
        assert_eq! (results.len (), 1);
        assert_eq! (results[0], 1);
        let results: Vec<ID> = unit_0.borrow ().choose_targets_units (&vec![0, 1], Target::Allies, Area::Radial (1), 0);
        assert_eq! (results.len (), 2);
        assert_eq! (results.contains (&0), true);
        assert_eq! (results.contains (&1), true);
        let results: Vec<ID> = unit_0.borrow ().choose_targets_units (&vec![2], Target::Enemy, Area::Single, 1);
        assert_eq! (results.len (), 1);
        assert_eq! (results[0], 2);
        let results: Vec<ID> = unit_0.borrow ().choose_targets_units (&vec![2], Target::Enemies, Area::Path (0), 1);
        assert_eq! (results.len (), 1);
        assert_eq! (results[0], 2);
    }

    #[test]
    fn unit_find_targets_units () {
        let handler = generate_handler ();
        let (unit_0, unit_1, unit_2) = generate_units (Rc::clone (&handler));
        let grid = generate_grid (Rc::downgrade (&handler));
        let potential_ids: Vec<ID> = vec![1, 2];

        // unit_0.find_targets_units (potential_ids, target, area, range);
        todo! ();
    }

    #[test]
    fn unit_choose_targets_locations () {
        let handler = generate_handler ();
        let (unit_0, unit_1, unit_2) = generate_units (Rc::clone (&handler));
        let grid = generate_grid (Rc::downgrade (&handler));

        // unit_0.choose_targets_locations (potential_ids, target, area, range);
        todo! ();
    }

    #[test]
    fn unit_find_targets_locations () {
        let handler = generate_handler ();
        let (unit_0, unit_1, unit_2) = generate_units (Rc::clone (&handler));
        let grid = generate_grid (Rc::downgrade (&handler));

        // unit_0.find_targets_locations (potential_ids, target, area, range);
        todo! ();
    }

    #[test]
    fn unit_act_attack () {
        let handler = generate_handler ();
        let (unit_0, unit_1, unit_2) = generate_units (Rc::clone (&handler));

        unit_0.borrow ().calculate_damage (&unit_1.borrow ().statistics.get ());
        unit_1.borrow ().calculate_damage (&unit_2.borrow ().statistics.get ());
        unit_2.borrow ().calculate_damage (&unit_0.borrow ().statistics.get ());
        todo! ();

        // println!("{:?}",weapon);
        // println!("{},{}",damage_mrl,damage_hlt);
        // println!("{},{},{}",
        // self.get_statistic (UnitStatistic::MRL).0,
        // self.get_statistic (UnitStatistic::HLT).0,
        // self.get_statistic (UnitStatistic::SPL).0);
        // assert! (false);
    }

    #[test]
    fn unit_act_magic () {
        todo! ();
    }

    #[test]
    fn unit_act_skill () {
        todo! ();
    }

    #[test]
    fn unit_add_appliable () {
        let handler = generate_handler ();
        let (unit_0, unit_1, _) = generate_units (Rc::clone (&handler));
        let (modifier_3, modifier_4) = generate_modifiers ();
        let (effect_0, effect_1) = generate_effects ();

        // Test additive modifier
        assert_eq! (unit_0.borrow_mut ().add_appliable (modifier_3.clone ()), true);
        assert_eq! (unit_0.borrow ().modifiers.len (), 1);
        assert_eq! (unit_0.borrow ().get_statistic (UnitStatistic::ATK).0, 22);
        // Test subtractive modifier
        assert_eq! (unit_0.borrow_mut ().add_appliable (modifier_4.clone ()), true); // Test multiple adjustments
        assert_eq! (unit_0.borrow ().modifiers.len (), 2);
        assert_eq! (unit_0.borrow ().get_statistic (UnitStatistic::ATK).0, 20);
        assert_eq! (unit_0.borrow ().get_statistic (UnitStatistic::DEF).0, 18);
        // Test stacking modifier
        assert_eq! (unit_0.borrow_mut ().add_appliable (modifier_3.clone ()), true);
        assert_eq! (unit_0.borrow ().modifiers.len (), 3);
        assert_eq! (unit_0.borrow ().get_statistic (UnitStatistic::ATK).0, 22);
        assert_eq! (unit_0.borrow_mut ().add_appliable (modifier_3), true);
        assert_eq! (unit_0.borrow ().modifiers.len (), 4);
        assert_eq! (unit_0.borrow ().get_statistic (UnitStatistic::ATK).0, 24);
        // Test non-stacking modifier
        assert_eq! (unit_0.borrow_mut ().add_appliable (modifier_4), false);
        assert_eq! (unit_0.borrow ().modifiers.len (), 4);
        assert_eq! (unit_0.borrow ().get_statistic (UnitStatistic::ATK).0, 24);

        // Test flat effect
        assert_eq! (unit_1.borrow_mut ().add_appliable (effect_0), true);
        assert_eq! (unit_1.borrow ().get_statistic (UnitStatistic::HLT).0, 998);
        // Test percentage effect
        assert_eq! (unit_1.borrow_mut ().add_appliable (effect_1), true); // Test multiple adjustments
        assert_eq! (unit_1.borrow ().get_statistic (UnitStatistic::ATK).0, 21);
        assert_eq! (unit_1.borrow ().get_statistic (UnitStatistic::DEF).0, 19);
    }

    #[test]
    fn unit_add_status () {
        let handler = generate_handler ();
        let lists: Rc<Lists> = generate_lists ();
        let (unit_0, _, _) = generate_units (Rc::clone (&handler));
        let (status_0, _, status_5) = generate_statuses ();
        let status_6 = lists.get_status (&6).clone ();

        // Test unit status
        assert_eq! (unit_0.borrow_mut ().add_status (status_0), true);
        assert_eq! (unit_0.borrow ().get_statistic (UnitStatistic::ATK).0, 22);
        // Test applier status
        assert_eq! (unit_0.borrow_mut ().add_status (status_5), true);
        assert_eq! (unit_0.borrow ().statuses.get (&Trigger::OnHit).unwrap ().len (), 1);
        assert! (matches! (unit_0.borrow ().try_yield_appliable (Rc::clone (&unit_0.borrow ().lists)), Some { .. }));
        // Test weapon status
        assert_eq! (unit_0.borrow_mut ().add_status (status_6), true);
        assert! (matches! (unit_0.borrow ().weapons[unit_0.borrow ().weapon_active].try_yield_appliable (lists), Some { .. }));
    }

    #[test]
    fn unit_dec_durations () {
        let handler = generate_handler ();
        let (unit_0, unit_1, _) = generate_units (Rc::clone (&handler));
        let (modifier_3, modifier_4) = generate_modifiers ();
        let (status_0, status_1, status_5) = generate_statuses ();

        // Test empty modifier
        unit_0.borrow_mut ().dec_durations ();
        assert_eq! (unit_0.borrow ().modifiers.len (), 0);
        // Test timed modifier
        unit_0.borrow_mut ().add_appliable (modifier_3.clone ());
        unit_0.borrow_mut ().dec_durations ();
        assert_eq! (unit_0.borrow ().modifiers.len (), 1);
        unit_0.borrow_mut ().dec_durations ();
        assert_eq! (unit_0.borrow ().modifiers.len (), 1);
        unit_0.borrow_mut ().dec_durations ();
        assert_eq! (unit_0.borrow ().modifiers.len (), 0);
        // Test permanent modifier
        unit_0.borrow_mut ().add_appliable (modifier_4);
        unit_0.borrow_mut ().dec_durations ();
        assert_eq! (unit_0.borrow ().modifiers.len (), 1);
        unit_0.borrow_mut ().dec_durations ();
        assert_eq! (unit_0.borrow ().modifiers.len (), 1);

        // Test empty status
        unit_1.borrow_mut ().dec_durations ();
        assert_eq! (unit_1.borrow ().statuses.get (&Trigger::None).unwrap ().len (), 0);
        // Test timed status
        unit_1.borrow_mut ().add_status (status_1);
        unit_1.borrow_mut ().dec_durations ();
        assert_eq! (unit_1.borrow ().statuses.get (&Trigger::None).unwrap ().len (), 1);
        unit_1.borrow_mut ().dec_durations ();
        assert_eq! (unit_1.borrow ().statuses.get (&Trigger::None).unwrap ().len (), 0);
        // Test permanent status
        unit_1.borrow_mut ().add_status (status_0);
        unit_1.borrow_mut ().dec_durations ();
        assert_eq! (unit_1.borrow ().statuses.get (&Trigger::None).unwrap ().len (), 1);
        unit_1.borrow_mut ().dec_durations ();
        assert_eq! (unit_1.borrow ().statuses.get (&Trigger::None).unwrap ().len (), 1);
        // Test linked status
        unit_1.borrow_mut ().add_status (status_5);
        unit_1.borrow_mut ().dec_durations ();
        assert_eq! (unit_1.borrow ().statuses.get (&Trigger::OnHit).unwrap ().len (), 1);
        assert_eq! (unit_1.borrow ().statuses.get (&Trigger::OnHit).unwrap ()[0].get_next_id ().unwrap (), 0);
        unit_1.borrow_mut ().dec_durations ();
        assert_eq! (unit_1.borrow ().statuses.get (&Trigger::OnHit).unwrap ().len (), 1);
        assert_eq! (unit_1.borrow ().statuses.get (&Trigger::OnHit).unwrap ()[0].get_next_id (), None);
    }
}
