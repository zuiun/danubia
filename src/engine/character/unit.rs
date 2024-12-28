use std::{collections::HashMap, rc::Rc};
use crate::engine::Lists;
use crate::engine::common::{Area, Capacity, ID, ID_UNINITIALISED, Target, Timed};
use crate::engine::event::{Event, Observer, Response, Subject, EVENT_CITY_DRAW_SUPPLY, EVENT_UNIT_DIE, FLAG_NOTIFICATION, FLAG_PART_NOTIFICATION, RESPONSE_NOTIFICATION};
use crate::engine::map::{Direction, Grid, Location};
use crate::engine::dynamic::{Appliable, Applier, Change, Changeable, Effect, Modifier, Statistic, Status, Trigger};
use super::*;

type UnitStatistics = [Capacity; UnitStatistic::Length as usize];

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
        Action::Attack (_) => delay,
        Action::Wait => ((delay as f32) * DELAY_WAIT) as u8,
        _ => ((delay as f32) * DELAY_APPLIER) as u8
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
    Attack (Location),
    Skill (ID, Location),
    Magic (ID, Location),
    Wait,
}

#[derive (Debug)]
pub struct UnitStatisticsBuilder {
    mrl: Capacity,
    hlt: Capacity,
    spl: Capacity,
    atk: Capacity,
    def: Capacity,
    mag: Capacity,
    mov: Capacity,
    org: Capacity,
}

impl UnitStatisticsBuilder {
    pub fn new (mrl: u16, hlt: u16, spl: u16, atk: u16, def: u16, mag: u16, mov: u16, org: u16) -> Self {
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

        Self { mrl, hlt, spl, atk, def, mag, mov, org }
    }

    pub fn build (self) -> UnitStatistics {
        [self.mrl, self.hlt, self.spl, self.atk, self.def, self.mag, self.mov, self.org]
    }
}

#[derive (Debug)]
pub struct Unit {
    id: ID,
    lists: Rc<Lists>,
    grid: Rc<Grid>,
    statistics: UnitStatistics,
    modifiers: Vec<Modifier>,
    statuses: HashMap<Trigger, Vec<Status>>,
    magic_ids: Vec<ID>,
    weapons: [Weapon; 2],
    weapon_active: usize,
    faction_id: ID,
    supply_city_ids: Vec<ID>,
    observer_id: ID,
}

impl Unit {
    pub fn new (id: ID, lists: Rc<Lists>, grid: Rc<Grid>, statistics_builder: UnitStatisticsBuilder, magic_ids: Vec<ID>, weapon_ids: [ID; 3], faction_id: ID) -> Self {
        let lists: Rc<Lists> = Rc::clone (&lists);
        let grid: Rc<Grid> = Rc::clone (&grid);
        let statistics: UnitStatistics = statistics_builder.build ();
        let modifiers: Vec<Modifier> = Vec::new ();
        let mut statuses: HashMap<Trigger, Vec<Status>> = HashMap::new ();
        let weapons: [Weapon; 2] = [
            lists.get_weapon (&weapon_ids[WEAPON_0]).clone (),
            lists.get_weapon (&weapon_ids[WEAPON_1]).clone (),
        ];
        let weapon_active: usize = WEAPON_0;
        let supply_city_ids: Vec<ID> = Vec::new ();
        let observer_id: ID = ID_UNINITIALISED;

        statuses.insert (Trigger::None, Vec::new ());

        Self { id, lists, grid, statistics, modifiers, statuses, magic_ids, weapons, weapon_active, faction_id, supply_city_ids, observer_id }
    }

    fn get_statistic (&self, statistic: UnitStatistic) -> (u16, u16) {
        match statistic {
            UnitStatistic::MRL => if let Capacity::Quantity (c, m) = self.statistics[UnitStatistic::MRL as usize] {
                (c, m)
            } else {
                panic! ("MRL should be a quantity");
            }
            UnitStatistic::HLT => if let Capacity::Quantity (c, m) = self.statistics[UnitStatistic::HLT as usize] {
                (c, m)
            } else {
                panic! ("HLT should be a quantity");
            }
            UnitStatistic::SPL => if let Capacity::Quantity (c, m) = self.statistics[UnitStatistic::SPL as usize] {
                (c, m)
            } else {
                panic! ("SPL should be a quantity");
            }
            UnitStatistic::ATK => if let Capacity::Constant (c, _, b) = self.statistics[UnitStatistic::ATK as usize] {
                (c, b)
            } else {
                panic! ("ATK should be a constant");
            }
            UnitStatistic::DEF => if let Capacity::Constant (c, _, b) = self.statistics[UnitStatistic::DEF as usize] {
                (c, b)
            } else {
                panic! ("DEF should be a constant");
            }
            UnitStatistic::MAG => if let Capacity::Constant (c, _, b) = self.statistics[UnitStatistic::MAG as usize] {
                (c, b)
            } else {
                panic! ("MAG should be a constant");
            }
            UnitStatistic::MOV => if let Capacity::Constant (c, _, b) = self.statistics[UnitStatistic::MOV as usize] {
                (c, b)
            } else {
                panic! ("MOV should be a constant");
            }
            UnitStatistic::ORG => if let Capacity::Quantity (c, m) = self.statistics[UnitStatistic::ORG as usize] {
                (c, m)
            } else {
                panic! ("ORG should be a quantity");
            }
            _ => panic! ("Statistic not found")
        }
    }

    fn set_statistic (&mut self, statistic: UnitStatistic, value: u16) -> () {
        self.statistics[statistic as usize] = match self.statistics[statistic as usize] {
            Capacity::Constant (_, m, b) => {
                assert! (value <= m);

                Capacity::Constant (value, m, b)
            }
            Capacity::Quantity (_, m) => {
                assert! (value <= m);

                Capacity::Quantity (value, m)
            }
        };

        if self.get_statistic (UnitStatistic::HLT).0 == 0 {
            self.die ();
        }
    }

    fn change_statistic_flat (&mut self, statistic: UnitStatistic, change: u16, is_add: bool) -> () {
        let (current, maximum): (u16, u16) = match self.statistics[statistic as usize] {
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
        let base: f32 = match self.statistics[statistic as usize] {
            Capacity::Constant (_, _, b) => { b }
            Capacity::Quantity (_, m) => { m }
        } as f32;
        let change: f32 = (change as f32) / 100.0;
        let change: u16 = (base * change) as u16;

        self.change_statistic_flat (statistic, change, is_add);
    }

    fn calculate_damage_weapon (&self, other: &Unit) -> u16 {
        let weapon: &Weapon = &self.weapons[self.weapon_active];
        let dmg_weapon: u16 = weapon.get_statistic (WeaponStatistic::DMG);
        let atk_self: u16 = self.get_statistic (UnitStatistic::ATK).0;
        let def_other: u16 = other.get_statistic (UnitStatistic::DEF).0;
        let spl_self: (u16, u16) = self.get_statistic (UnitStatistic::SPL);
        let spl_other: (u16, u16) = other.get_statistic (UnitStatistic::SPL);
        let atk_final: u16 = atk_self + dmg_weapon;
        let multiplier_other: f32 = (spl_other.0 as f32) / (spl_other.1 as f32);
        let def_final: u16 = ((def_other as f32) * multiplier_other) as u16;
        let damage: u16 = u16::max (atk_final.checked_sub (def_final).unwrap_or (1), 1);
        let multiplier_self: f32 = (spl_self.0 as f32) / (spl_self.1 as f32);

        ((damage as f32) * multiplier_self) as u16
    }

    fn calculate_damage_bonus (&self, other: &Unit) -> u16 {
        let weapon: &Weapon = &self.weapons[self.weapon_active];
        let mag_self: u16 = self.get_statistic (UnitStatistic::MAG).0;
        let mag_other: u16 = other.get_statistic (UnitStatistic::MAG).0;
        let damage: u16 = u16::max (mag_self.checked_sub (mag_other).unwrap_or (1), 1);
        let multiplier: u16 = ((weapon.get_statistic (WeaponStatistic::DCY) + 1) as u16) * 2;

        damage * multiplier
    }

    fn calculate_damage_multiplier (&self) -> f32 {
        let hlt: (u16, u16) = self.get_statistic (UnitStatistic::HLT);
        let org: u16 = self.get_statistic (UnitStatistic::ORG).0;
        let multiplier_hlt: f32 = (hlt.0 as f32) / (hlt.1 as f32);
        let multiplier_org: f32 = (org as f32) / 100.0;

        multiplier_hlt * multiplier_org
    }

    pub fn calculate_damage (&self, other: &Unit) -> (u16, u16) {
        let weapon: &Weapon = &self.weapons[self.weapon_active];
        let damage_weapon: u16 = self.calculate_damage_weapon (other);
        let damage_bonus: u16 = self.calculate_damage_bonus (other);
        let multiplier: f32 = self.calculate_damage_multiplier ();
        let damage_base: u16 = (((damage_weapon + damage_bonus) as f32) * multiplier) as u16;
        let damage_mrl: u16 = damage_base + (weapon.get_statistic (WeaponStatistic::SLH) as u16);
        let damage_hlt: u16 = damage_base * ((weapon.get_statistic (WeaponStatistic::PRC) + 1) as u16);

        (damage_mrl, damage_hlt)
    }

    fn take_damage (&mut self, damage_mrl: u16, damage_hlt: u16) -> bool {
        let damage_spl: u16 = (damage_mrl + damage_hlt) / 2;

        self.change_statistic_flat (UnitStatistic::MRL, damage_mrl, false);
        self.change_statistic_flat (UnitStatistic::HLT, damage_hlt, false);
        self.change_statistic_flat (UnitStatistic::SPL, damage_spl, false);

        if self.get_statistic (UnitStatistic::HLT).0 == 0 {
            true
        } else {
            false
        }
    }

    fn die (&mut self) -> () {
        self.notify ((EVENT_UNIT_DIE, (self.id, FLAG_PART_NOTIFICATION, FLAG_PART_NOTIFICATION))); // Likely don't need to await
        // TODO: ???
    }

    fn find_targets (&self, location: Location, direction: Option<Direction>, target: Target, area: Area, range: u8) -> Vec<ID> {
        let mut unit_ids: Vec<ID> = Vec::new ();

        // match target {
        //     Target::Ally (s) => {
        //         if s {
        //             unit_ids.push (self.id);
        //         } else {
        //             let other_id: &ID = self.grid.get_location_unit (&location)
        //                     .expect (panic! ("Unit not found for location {:?}", location));

        //             unit_ids.push (*other_id);
        //         }
        //     }
        //     Target::Enemy => {
        //         let other_id: &ID = self.grid.get_location_unit (&location)
        //                 .expect (panic! ("Unit not found for location {:?}", location));

        //         unit_ids.push (*other_id);
        //     }
        //     Target::All (s) => {
        //         if range > 0 {
        //             unit_ids = self.grid.find_nearby_units (location, area, range);
        //         } else {
        //             let faction: &Faction = self.lists.get_faction (&self.faction_id);
        //             // TODO: Get allies from faction
        //             todo! ()
        //         };
        //     }
        //     Target::Map => {

        //     }
        // }

        unit_ids
    }

    pub fn start_turn (&mut self) -> () {
        let location: &Location = self.grid.get_unit_location (&self.id)
                .expect (&format! ("Location not found for unit {}", self.id));

        if self.grid.is_impassable (location) {
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
            Action::Attack (l) => {
                let weapon: &Weapon = &self.weapons[self.weapon_active];

                // other.take_damage (damage_mrl, damage_hlt)
                // let damage_self = other.calculate_damage (self);
                // let damage_spl = (damage_self.0 + damage_self.1) / 2;
                // self.change_statistic_flat (UnitStatistic::SPL, damage_spl, false);
            }
            Action::Magic (m, l) => {
                let magic: &Magic = self.lists.get_magic (&m);
            }
            Action::Skill (s, l) => {
                let skill: &Skill = self.lists.get_skill (&s);
            }
            Action::Wait => ()
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
        //         // TODO: Get mut units from somewhere???
        //     }
        // } else {
        //     panic! ("Invalid target {:?}", target)
        // }

        todo! ()
    }

    pub async fn end_turn (&mut self) -> () {
        self.dec_durations ();
        self.supply_city_ids = self.grid.find_unit_cities (&self.id);

        if self.supply_city_ids.len () > 0 {
            let mut recover_hlt: u16 = 0;
            let mut recover_spl: u16 = 0;

            for supply_city_id in &self.supply_city_ids {
                let change_hlt: u16 = self.lists.get_city (supply_city_id).get_manpower ();
                let change_spl: u16 = self.lists.get_city (supply_city_id).get_equipment ();

                recover_hlt += change_hlt;
                recover_spl += change_spl;
            }

            self.change_statistic_flat (UnitStatistic::MRL, RECOVER_MRL, true);
            self.change_statistic_flat (UnitStatistic::HLT, recover_hlt, true);
            self.change_statistic_flat (UnitStatistic::SPL, recover_spl, false);
        } else {
            self.change_statistic_flat (UnitStatistic::SPL, DRAIN_SPL, false);
        }
    }

    pub fn get_faction_id (&self) -> ID {
        self.faction_id
    }
}

impl Observer for Unit {
    async fn update (&mut self, event: Event) -> Response {
        todo! ()
    }

    fn get_observer_id (&self) -> Option<ID> {
        if self.observer_id == ID_UNINITIALISED {
            None
        } else {
            Some (self.observer_id)
        }
    }

    fn set_observer_id (&mut self, observer_id: ID) -> () {
        self.observer_id = observer_id;
    }
}

impl Subject for Unit {
    async fn notify (&self, event: Event) -> Response {
        RESPONSE_NOTIFICATION
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
                            if let Statistic::Unit (s) = a.0 {
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
                        if let Statistic::Unit (s) = a.0 {
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
            let status: Status = status.clone ();
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
                _ => false
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

pub struct UnitBuilder {
    // TODO
}

impl UnitBuilder {
    pub fn new () -> Self {
        todo! ()
    }

    pub fn build (self) -> Unit {
        todo! ()
    }
}

#[cfg (test)]
mod tests {
    use super::*;
    use crate::engine::common::{DURATION_PERMANENT, FACTION_NONE};
    use crate::engine::dynamic::ModifierBuilder;
    use crate::engine::tests::generate_lists;
    use crate::engine::map::grid::tests::generate_grid;

    fn generate_units () -> (Unit, Unit, Unit) {
        let lists: Rc<Lists> = generate_lists ();
        let grid: Grid = generate_grid ();
        let grid: Rc<Grid> = Rc::new (grid);

        let statistics_unit_0: UnitStatisticsBuilder = UnitStatisticsBuilder::new (100, 1000, 100, 20, 20, 20, 10, 100);
        let unit_0: Unit = Unit::new (0, Rc::clone (&lists), Rc::clone (&grid), statistics_unit_0, vec![], [0, 0, 0], 0);
        let statistics_unit_1: UnitStatisticsBuilder = UnitStatisticsBuilder::new (100, 1000, 100, 20, 20, 20, 10, 100);
        let unit_1: Unit = Unit::new (1, Rc::clone (&lists), Rc::clone (&grid), statistics_unit_1, vec![], [1, 0, 1], 0);
        let statistics_unit_2: UnitStatisticsBuilder = UnitStatisticsBuilder::new (100, 1000, 100, 20, 20, 20, 10, 100);
        let unit_2: Unit = Unit::new (2, Rc::clone (&lists), Rc::clone (&grid), statistics_unit_2, vec![], [2, 0, 2], 0);

        (unit_0, unit_1, unit_2)
    }

    fn generate_modifiers () -> (Box<Modifier>, Box<Modifier>) {
        let lists: Rc<Lists> = generate_lists ();
        let modifier_builder_3: &ModifierBuilder = lists.get_modifier_builder (&3);
        let modifier_3: Modifier = modifier_builder_3.build (2, true);
        let modifier_3: Box<Modifier> = Box::new (modifier_3);
        let modifier_builder_4: &ModifierBuilder = lists.get_modifier_builder (&4);
        let modifier_4: Modifier = modifier_builder_4.build (DURATION_PERMANENT, false);
        let modifier_4: Box<Modifier> = Box::new (modifier_4);

        (modifier_3, modifier_4)
    }

    fn generate_effects () -> (Box<Effect>, Box<Effect>) {
        let lists: Rc<Lists> = generate_lists ();
        let effect_0: Effect = lists.get_effect (&0).clone ();
        let effect_0: Box<Effect> = Box::new (effect_0);
        let effect_1: Effect = lists.get_effect (&1).clone ();
        let effect_1: Box<Effect> = Box::new (effect_1);

        (effect_0, effect_1)
    }

    fn generate_statuses () -> (Status, Status, Status) {
        let lists: Rc<Lists> = generate_lists ();
        let status_0: Status = lists.get_status (&0).clone ();
        let status_1: Status = lists.get_status (&1).clone ();
        let status_5: Status = lists.get_status (&5).clone ();

        (status_0, status_1, status_5)
    }

    #[test]
    fn unit_change_statistic_flat () {
        let (mut unit_0, _, _): (Unit, _, _) = generate_units ();

        // Test constant
        unit_0.change_statistic_flat (UnitStatistic::ATK, 5, true); // Test additive change
        assert_eq! (unit_0.get_statistic (UnitStatistic::ATK).0, 25);
        unit_0.change_statistic_flat (UnitStatistic::ATK, 5, false); // Test subtractive change
        assert_eq! (unit_0.get_statistic (UnitStatistic::ATK).0, 20);
        unit_0.change_statistic_flat (UnitStatistic::ATK, ATK_MAX, true); // Test maximum change
        assert_eq! (unit_0.get_statistic (UnitStatistic::ATK).0, ATK_MAX);
        unit_0.change_statistic_flat (UnitStatistic::ATK, ATK_MAX, false); // Test minimum change
        assert_eq! (unit_0.get_statistic (UnitStatistic::ATK).0, 0);
        // Test quantity
        unit_0.change_statistic_flat (UnitStatistic::HLT, 10, false); // Test subtractive change
        assert_eq! (unit_0.get_statistic (UnitStatistic::HLT).0, 990);
        unit_0.change_statistic_flat (UnitStatistic::HLT, 5, true); // Test additive change
        assert_eq! (unit_0.get_statistic (UnitStatistic::HLT).0, 995);
        unit_0.change_statistic_flat (UnitStatistic::HLT, HLT_MAX, true); // Test maximum change
        assert_eq! (unit_0.get_statistic (UnitStatistic::HLT).0, HLT_MAX);
        unit_0.change_statistic_flat (UnitStatistic::HLT, HLT_MAX, false); // Test minimum change
        assert_eq! (unit_0.get_statistic (UnitStatistic::HLT).0, 0);
    }

    #[test]
    fn unit_change_statistic_percentage () {
        let (mut unit_0, _, _): (Unit, _, _) = generate_units ();

        // Test constant
        unit_0.change_statistic_percentage (UnitStatistic::ATK, 10, true); // Test additive change
        assert_eq! (unit_0.get_statistic (UnitStatistic::ATK).0, 22);
        unit_0.change_statistic_percentage (UnitStatistic::ATK, 10, false); // Test subtractive change
        assert_eq! (unit_0.get_statistic (UnitStatistic::ATK).0, 20);
        unit_0.change_statistic_percentage (UnitStatistic::ATK, 1000, true); // Test maximum change
        assert_eq! (unit_0.get_statistic (UnitStatistic::ATK).0, ATK_MAX);
        unit_0.change_statistic_percentage (UnitStatistic::ATK, 1000, false); // Test minimum change
        assert_eq! (unit_0.get_statistic (UnitStatistic::ATK).0, 0);
        // Test quantity
        unit_0.change_statistic_percentage (UnitStatistic::HLT, 10, false); // Test subtractive change
        assert_eq! (unit_0.get_statistic (UnitStatistic::HLT).0, 900);
        unit_0.change_statistic_percentage (UnitStatistic::HLT, 5, true); // Test additive change
        assert_eq! (unit_0.get_statistic (UnitStatistic::HLT).0, 950);
        unit_0.change_statistic_percentage (UnitStatistic::HLT, 1000, true); // Test maximum change
        assert_eq! (unit_0.get_statistic (UnitStatistic::HLT).0, HLT_MAX);
        unit_0.change_statistic_percentage (UnitStatistic::HLT, 1000, false); // Test minimum change
        assert_eq! (unit_0.get_statistic (UnitStatistic::HLT).0, 0);
    }

    #[test]
    fn unit_act_attack () {
        let (mut unit_0, mut unit_1, mut unit_2): (Unit, Unit, Unit) = generate_units ();

        unit_0.calculate_damage (&mut unit_1);
        unit_1.calculate_damage (&mut unit_2);
        unit_2.calculate_damage (&mut unit_0);
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
        let (mut unit_0, mut unit_1, _): (Unit, Unit, _) = generate_units ();
        let (modifier_3, modifier_4): (Box<Modifier>, Box<Modifier>) = generate_modifiers ();
        let (effect_0, effect_1): (Box<Effect>, Box<Effect>) = generate_effects ();

        // Test additive modifier
        assert_eq! (unit_0.add_appliable (modifier_3.clone ()), true);
        assert_eq! (unit_0.modifiers.len (), 1);
        assert_eq! (unit_0.get_statistic (UnitStatistic::ATK).0, 22);
        // Test subtractive modifier
        assert_eq! (unit_0.add_appliable (modifier_4.clone ()), true); // Test multiple adjustments
        assert_eq! (unit_0.modifiers.len (), 2);
        assert_eq! (unit_0.get_statistic (UnitStatistic::ATK).0, 20);
        assert_eq! (unit_0.get_statistic (UnitStatistic::DEF).0, 18);
        // Test stacking modifier
        assert_eq! (unit_0.add_appliable (modifier_3.clone ()), true);
        assert_eq! (unit_0.modifiers.len (), 3);
        assert_eq! (unit_0.get_statistic (UnitStatistic::ATK).0, 22);
        assert_eq! (unit_0.add_appliable (modifier_3), true);
        assert_eq! (unit_0.modifiers.len (), 4);
        assert_eq! (unit_0.get_statistic (UnitStatistic::ATK).0, 24);
        // Test non-stacking modifier
        assert_eq! (unit_0.add_appliable (modifier_4), false);
        assert_eq! (unit_0.modifiers.len (), 4);
        assert_eq! (unit_0.get_statistic (UnitStatistic::ATK).0, 24);

        // Test flat effect
        assert_eq! (unit_1.add_appliable (effect_0), true);
        assert_eq! (unit_1.get_statistic (UnitStatistic::HLT).0, 998);
        // Test percentage effect
        assert_eq! (unit_1.add_appliable (effect_1), true); // Test multiple adjustments
        assert_eq! (unit_1.get_statistic (UnitStatistic::ATK).0, 21);
        assert_eq! (unit_1.get_statistic (UnitStatistic::DEF).0, 19);
    }

    #[test]
    fn unit_add_status () {
        let lists: Rc<Lists> = generate_lists ();
        let (mut unit_0, _, _): (Unit, _, _) = generate_units ();
        let (status_0, _, status_5): (Status, _, Status) = generate_statuses ();
        let status_6: Status = lists.get_status (&6).clone ();

        // Test unit status
        assert_eq! (unit_0.add_status (status_0), true);
        assert_eq! (unit_0.get_statistic (UnitStatistic::ATK).0, 22);
        // Test applier status
        assert_eq! (unit_0.add_status (status_5), true);
        assert_eq! (unit_0.statuses.get (&Trigger::OnHit).unwrap ().len (), 1);
        assert! (matches! (unit_0.try_yield_appliable (Rc::clone (&unit_0.lists)), Some { .. }));
        // Test weapon status
        assert_eq! (unit_0.add_status (status_6), true);
        assert! (matches! (unit_0.weapons[unit_0.weapon_active].try_yield_appliable (lists), Some { .. }));
    }

    #[test]
    fn unit_dec_durations () {
        let (mut unit_0, mut unit_1, _): (Unit, Unit, _) = generate_units ();
        let (modifier_3, modifier_4): (Box<Modifier>, Box<Modifier>) = generate_modifiers ();
        let (status_0, status_1, status_5): (Status, Status, Status) = generate_statuses ();

        // Test empty modifier
        unit_0.dec_durations ();
        assert_eq! (unit_0.modifiers.len (), 0);
        // Test timed modifier
        unit_0.add_appliable (modifier_3.clone ());
        unit_0.dec_durations ();
        assert_eq! (unit_0.modifiers.len (), 1);
        unit_0.dec_durations ();
        assert_eq! (unit_0.modifiers.len (), 0);
        // Test permanent modifier
        unit_0.add_appliable (modifier_4);
        unit_0.dec_durations ();
        assert_eq! (unit_0.modifiers.len (), 1);
        unit_0.dec_durations ();
        assert_eq! (unit_0.modifiers.len (), 1);
        // Test multiple modifiers
        unit_0.add_appliable (modifier_3);
        unit_0.dec_durations ();
        assert_eq! (unit_0.modifiers.len (), 2);
        unit_0.dec_durations ();
        assert_eq! (unit_0.modifiers.len (), 1);

        
        // Test empty status
        unit_1.dec_durations ();
        assert_eq! (unit_1.statuses.get (&Trigger::None).unwrap ().len (), 0);
        // Test timed status
        unit_1.add_status (status_1);
        unit_1.dec_durations ();
        assert_eq! (unit_1.statuses.get (&Trigger::None).unwrap ().len (), 1);
        unit_1.dec_durations ();
        assert_eq! (unit_1.statuses.get (&Trigger::None).unwrap ().len (), 0);
        // Test permanent status
        unit_1.add_status (status_0);
        unit_1.dec_durations ();
        assert_eq! (unit_1.statuses.get (&Trigger::None).unwrap ().len (), 1);
        unit_1.dec_durations ();
        assert_eq! (unit_1.statuses.get (&Trigger::None).unwrap ().len (), 1);
        // Test linked status
        unit_1.add_status (status_5);
        unit_1.dec_durations ();
        assert_eq! (unit_1.statuses.get (&Trigger::OnHit).unwrap ().len (), 1);
        assert_eq! (unit_1.statuses.get (&Trigger::OnHit).unwrap ()[0].get_next_id ().unwrap (), 0);
        unit_1.dec_durations ();
        assert_eq! (unit_1.statuses.get (&Trigger::OnHit).unwrap ().len (), 1);
        assert_eq! (unit_1.statuses.get (&Trigger::OnHit).unwrap ()[0].get_next_id (), None);
    }
}
