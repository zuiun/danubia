use std::{cell::RefCell, cmp, rc::Rc, sync::atomic::Ordering};
use crate::engine::Lists;
use crate::engine::common::{Area, Capacity, ID, IDS, Modifiable, Modifier, Statistic, Target, Timed, TYPE_UNIT, UnitStatistic, Unique, WeaponStatistic};
use crate::engine::event::{Event, Observer, Subject, ACTION_CITY_DRAW_SUPPLY, ACTION_UNIT_DIE, OBSERVER_NOTIFICATION, Result, RESULT_NOTIFICATION, VALUE_NOTIFICATION};
use crate::engine::map::{Grid, Location};
use super::*;

type UnitStatistics = [Capacity; UnitStatistic::Length as usize];

const MRL_MAX: u16 = 100;
const HLT_MAX: u16 = 1000;
const SPL_MAX: u16 = 100;
const ATK_MAX: u16 = 200;
const DEF_MAX: u16 = 200;
const MAG_MAX: u16 = 200;
const MOV_MAX: u16 = 100;
const ORG_MAX: u16 = 200;
const DAMAGE_DIVIDEND_SPL: u16 = 3;
const DRAIN_SPL: u16 = 5;
const RECOVER_MRL: u16 = 1;
const WEAPON_1: usize = 0;
const WEAPON_2: usize = 1;
const WEAPON_ACTIVE: usize = 2;

#[derive (Debug)]
pub struct UnitStatisticsBuilder {
    mrl: Capacity,
    hlt: Capacity,
    spl: Capacity,
    atk: Capacity,
    def: Capacity,
    mag: Capacity,
    mov: Capacity,
    org: Capacity
}

impl UnitStatisticsBuilder {
    pub fn new (mrl: u16, hlt: u16, spl: u16, atk: u16, def: u16, mag: u16, mov: u16, org: u16) -> Self {
        assert! (mrl <= MRL_MAX);
        assert! (hlt <= HLT_MAX);
        assert! (spl <= SPL_MAX);
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
    magic_ids: Vec<ID>,
    weapon_ids: [ID; 3],
    faction_id: ID,
    supply_city_ids: Vec<ID>,
    observers: Vec<Rc<RefCell<dyn Observer>>>
}

impl Unit {
    pub fn new (lists: Rc<Lists>, statistics_builder: UnitStatisticsBuilder, magic_ids: Vec<ID>, weapon_ids: [ID; 3], faction_id: ID, grid: Rc<Grid>) -> Self {
        let id: ID = Unit::assign_id ();
        let lists: Rc<Lists> = Rc::clone (&lists);
        let statistics: UnitStatistics = statistics_builder.build ();
        let modifiers: Vec<Modifier> = Vec::new ();
        let supply_city_ids: Vec<ID> = Vec::new ();
        let observers: Vec<Rc<RefCell<dyn Observer>>> = Vec::new ();

        Self { id, lists, grid: grid, statistics, modifiers, magic_ids, weapon_ids, faction_id, supply_city_ids, observers }
    }

    fn get_statistic (&self, statistic: UnitStatistic) -> (u16, u16) {
        match statistic {
            UnitStatistic::MRL => if let Capacity::Quantity (_, _) = self.statistics[UnitStatistic::MRL as usize] {
                ()
            } else {
                panic! ("MRL should be a quantity");
            }
            UnitStatistic::HLT => if let Capacity::Quantity (_, _) = self.statistics[UnitStatistic::HLT as usize] {
                ()
            } else {
                panic! ("HLT should be a quantity");
            }
            UnitStatistic::SPL => if let Capacity::Quantity (_, _) = self.statistics[UnitStatistic::SPL as usize] {
                ()
            } else {
                panic! ("SPL should be a quantity");
            }
            UnitStatistic::ATK => if let Capacity::Constant (_, _, _) = self.statistics[UnitStatistic::ATK as usize] {
                ()
            } else {
                panic! ("ATK should be a constant");
            }
            UnitStatistic::DEF => if let Capacity::Constant (_, _, _) = self.statistics[UnitStatistic::DEF as usize] {
                ()
            } else {
                panic! ("DEF should be a constant");
            }
            UnitStatistic::MAG => if let Capacity::Constant (_, _, _) = self.statistics[UnitStatistic::MAG as usize] {
                ()
            } else {
                panic! ("MAG should be a constant");
            }
            UnitStatistic::MOV => if let Capacity::Constant (_, _, _) = self.statistics[UnitStatistic::MOV as usize] {
                ()
            } else {
                panic! ("MOV should be a constant");
            }
            UnitStatistic::ORG => if let Capacity::Quantity (_, _) = self.statistics[UnitStatistic::ORG as usize] {
                ()
            } else {
                panic! ("ORG should be a quantity");
            }
            _ => panic! ("Statistic not found")
        }

        match self.statistics[statistic as usize] {
            Capacity::Constant (c, _, b) => {
                (c, b)
            }
            Capacity::Quantity (c, m) => {
                (c, m)
            }
        }
    }

    fn set_statistic (&mut self, statistic: UnitStatistic, value: u16) -> () {
        self.statistics[statistic as usize] = match self.statistics[statistic as usize] {
            Capacity::Constant (_, m, b) => {
                Capacity::Constant (value, m, b)
            }
            Capacity::Quantity (_, m) => {
                assert! (value <= m);

                Capacity::Quantity (value, m)
            }
        };
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
            cmp::min (current + change, maximum)
        } else {
            current.checked_sub (change).unwrap_or (0)
        };

        self.set_statistic (statistic, value);
    }

    fn change_statistic_percentage (&mut self, statistic: UnitStatistic, change: u16, is_add: bool) -> () {
        let base: u16 = match self.statistics[statistic as usize] {
            Capacity::Constant (_, _, b) => {
                b
            }
            Capacity::Quantity (_, m) => {
                m
            }
        };
        let change: f32 = if is_add {
            (change as f32) / 100.0
        } else {
            1.0 - ((change as f32) / 100.0)
        };
        let change: u16 = ((base as f32) * change) as u16;

        self.change_statistic_flat (statistic, change, is_add);
    }

    fn calculate_damage_weapon (&self, other: &Unit) -> u16 {
        let weapon: &Weapon = self.lists.get_weapon (&self.weapon_ids[WEAPON_ACTIVE]);
        let dmg_weapon: u16 = weapon.get_statistic (WeaponStatistic::DMG);
        let atk_self: u16 = self.get_statistic (UnitStatistic::ATK).0;
        let def_other: u16 = other.get_statistic (UnitStatistic::DEF).0;
        let spl_self: (u16, u16) = self.get_statistic (UnitStatistic::SPL);
        let spl_other: (u16, u16) = other.get_statistic (UnitStatistic::SPL);
        let atk_final: u16 = atk_self + dmg_weapon;
        let multiplier_other: f32 = (spl_other.0 as f32) / (spl_other.1 as f32);
        let def_final: u16 = ((def_other as f32) * multiplier_other) as u16;
        let damage: u16 = cmp::max (atk_final.checked_sub (def_final).unwrap_or (1), 1);
        let multiplier_self: f32 = (spl_self.0 as f32) / (spl_self.1 as f32);

        ((damage as f32) * multiplier_self) as u16
    }

    fn calculate_damage_bonus (&self, other: &Unit) -> u16 {
        let weapon: &Weapon = self.lists.get_weapon (&self.weapon_ids[WEAPON_ACTIVE]);
        let mag_self: u16 = self.get_statistic (UnitStatistic::MAG).0;
        let mag_other: u16 = other.get_statistic (UnitStatistic::MAG).0;
        let damage: u16 = cmp::max (mag_self.checked_sub (mag_other).unwrap_or (1), 1);
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

    fn take_damage (&mut self, damage_mrl: u16, damage_hlt: u16) -> bool {
        let damage_spl: u16 = damage_mrl / DAMAGE_DIVIDEND_SPL;

        self.change_statistic_flat (UnitStatistic::MRL, damage_mrl, false);
        self.change_statistic_flat (UnitStatistic::HLT, damage_hlt, false);
        self.change_statistic_flat (UnitStatistic::SPL, damage_spl, false);

println!("{},{}",
self.get_statistic (UnitStatistic::MRL).0,
self.get_statistic (UnitStatistic::HLT).0);

        if self.get_statistic (UnitStatistic::HLT).0 == 0 {
            self.die ();
            
            true
        } else {
            false
        }
    }

    fn die (&mut self) -> () {
        self.notify ((ACTION_UNIT_DIE, OBSERVER_NOTIFICATION, VALUE_NOTIFICATION));
        // TODO: ???
    }

    fn find_targets (&self, target: Target, area: Area, range: u8) -> Vec<ID> {
        let mut unit_ids: Vec<ID> = Vec::new ();

        match target {
            Target::Ally (s) => {
                if s {
                    unit_ids.push (self.id);
                } else {
                    // TODO: Choose target
                    let other_id: ID = ID::MAX;

                    unit_ids.push (other_id);
                }
            }
            Target::Enemy => {

            }
            Target::All (s) => {
                if range > 0 {
                    unit_ids = self.grid.find_nearby_allies (&self.id, target, area, range);
                } else {
                    let faction: &Faction = self.lists.get_faction (&self.faction_id);
                    // TODO: Get allies from faction
                    todo! ()
                };
            }
            Target::Map => {

            }
        }

        unit_ids
    }

    pub fn start_turn (&mut self) -> () {
        self.modifiers.retain (|m| m.get_duration () > 0);
        // TODO: apply all constant passive skills

        let location: &Location = self.grid.get_unit_location (&self.id)
                .expect (&format! ("Location not found for unit {}", self.id));

        if self.grid.is_impassable (location) {
            self.die ();
        }
    }

    pub fn switch_weapon (&mut self) -> ID {
        self.weapon_ids[WEAPON_ACTIVE] = if self.weapon_ids[WEAPON_ACTIVE] == self.weapon_ids[WEAPON_1] {
            self.weapon_ids[WEAPON_1]
        } else {
            self.weapon_ids[WEAPON_2]
        };

        self.weapon_ids[WEAPON_ACTIVE]
    }

    pub fn act_attack (&mut self, other: &mut Unit) -> bool {
        let weapon: &Weapon = self.lists.get_weapon (&self.weapon_ids[WEAPON_ACTIVE]);
        let damage_weapon: u16 = self.calculate_damage_weapon (other);
        let damage_bonus: u16 = self.calculate_damage_bonus (other);
        let multiplier: f32 = self.calculate_damage_multiplier ();
        let damage_base: u16 = (((damage_weapon + damage_bonus) as f32) * multiplier) as u16;
        let damage_mrl: u16 = damage_base + (weapon.get_statistic (WeaponStatistic::SLH) as u16);
        let damage_hlt: u16 = damage_base * ((weapon.get_statistic (WeaponStatistic::PRC) + 1) as u16);

println!("{:?}",weapon);
println!("{},{}",damage_mrl,damage_hlt);

        other.take_damage (damage_mrl, damage_hlt)
    }

    pub fn act_magic (&mut self, magic_id: ID) -> () {
        let magic: &Magic = self.lists.get_magic (&magic_id);
        let unit_ids: Vec<ID> = self.find_targets (magic.get_target (), magic.get_area (), magic.get_range ());

        todo! ()
        // magic.act ();
    }

    pub fn act_skill (&mut self, skill_id: ID) -> () {
        let skill: &Skill = self.lists.get_skill (&skill_id);

        // Skills always target allies
        // if let Target::Ally (s) = target {
        //     if s {
        //         self.add_modifier (modifier.clone ());
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
        // skill.act ();
    }

    pub async fn end_turn (&mut self) -> () {
        self.dec_durations ();
        self.supply_city_ids = self.grid.get_unit_supply_cities (&self.id);

println!("{:?}",self.supply_city_ids);
        if self.supply_city_ids.len () > 0 {
            let mut recover_hlt: u16 = 0;
            let mut recover_spl: u16 = 0;

            for supply_city_id in &self.supply_city_ids {
                // TODO: This eventually needs to mut draw supplies
                self.notify ((ACTION_CITY_DRAW_SUPPLY, (*supply_city_id, 0), VALUE_NOTIFICATION)).await;

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

impl Unique for Unit {
    fn assign_id () -> ID {
        IDS[TYPE_UNIT as usize].fetch_add (1, Ordering::SeqCst)
    }

    fn get_id (&self) -> ID {
        self.id
    }

    fn get_type (&self) -> ID {
        TYPE_UNIT
    }
}

impl Observer for Unit {
    fn update (&mut self, event: Event) -> () {
        match event {
            _ => ()
        }
    }
}

impl Subject for Unit {
    fn add_observer (&mut self, observer: Rc<RefCell<dyn Observer>>) -> () {
        let observer: Rc<RefCell<dyn Observer>> = Rc::clone (&observer);

        self.observers.push (observer);
    }

    fn remove_observer (&mut self, observer: Rc<RefCell<dyn Observer>>) -> () {
        unimplemented! ()
    }

    async fn notify (&self, event: Event) -> Result {
        for observer in self.observers.iter () {
            observer.borrow_mut ().update (event);
        }

        RESULT_NOTIFICATION
    }
}

impl Modifiable for Unit {
    fn add_modifier (&mut self, modifier: Modifier) -> bool {
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

    fn dec_durations (&mut self) -> () {
        for modifier in self.modifiers.iter_mut () {
            modifier.dec_duration ();
        }
    }
}

#[cfg (test)]
mod tests {
    use super::*;
    use crate::engine::tests::generate_lists;
    use crate::engine::map::grid::tests::generate_grid;

    fn generate_units () -> (Unit, Unit, Unit) {
        let lists: Rc<Lists> = generate_lists ();
        let grid: Grid = generate_grid ();
        let grid: Rc<Grid> = Rc::new (grid);

        let statistics_character_1: UnitStatisticsBuilder = UnitStatisticsBuilder::new (100, 1000, 100, 20, 20, 20, 10, 100);
        let character_1: Unit = Unit::new (Rc::clone (&lists), statistics_character_1, vec![], [0, 0, 0], 0, Rc::clone (&grid));

        let statistics_character_2: UnitStatisticsBuilder = UnitStatisticsBuilder::new (100, 1000, 100, 20, 20, 20, 10, 100);
        let character_2: Unit = Unit::new (Rc::clone (&lists), statistics_character_2, vec![], [1, 0, 1], 0, Rc::clone (&grid));

        let statistics_character_3: UnitStatisticsBuilder = UnitStatisticsBuilder::new (100, 1000, 100, 20, 20, 20, 10, 100);
        let character_3: Unit = Unit::new (Rc::clone (&lists), statistics_character_3, vec![], [2, 0, 2], 0, Rc::clone (&grid));

        (character_1, character_2, character_3)
    }

    #[test]
    fn unit_act_attack () {
        let (mut character_1, mut character_2, mut character_3) = generate_units ();

        character_1.act_attack (&mut character_2);
        character_2.act_attack (&mut character_3);
        character_3.act_attack (&mut character_1);
        assert! (false);
    }

    #[test]
    fn unit_act_magic () {

    }

    #[test]
    fn unit_act_skill () {

    }
}
