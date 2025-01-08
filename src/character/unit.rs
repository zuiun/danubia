use super::*;
use self::UnitStatistic::{MRL, HLT, SPL, ATK, DEF, MAG, MOV, ORG};
use crate::common::{Capacity, Target, Timed, ID, MULTIPLIER_ATTACK, MULTIPLIER_MAGIC, MULTIPLIER_SKILL, MULTIPLIER_WAIT};
use crate::dynamic::{Appliable, Applier, Change, Changeable, Effect, Modifier, Statistic, Status, Trigger};
use crate::Scene;
use std::collections::HashMap;
use std::rc::Rc;

const PERCENT_1: u16 = 1_0;
#[allow (clippy::inconsistent_digit_grouping)]
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
#[allow (clippy::inconsistent_digit_grouping)]
const THRESHOLD_RETREAT_MRL: u16 = 40_0; // 40.0%
#[allow (clippy::inconsistent_digit_grouping)]
const THRESHOLD_ROUT_MRL: u16 = 20_0; // 20.0%
const THRESHOLD_SKILL_PASSIVE: usize = 1; // TODO: probably needs to be balanced

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
    ORG, // cohesion – modifier for physical damage, resistance, and leader passive (permillage)
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
    #[allow (clippy::too_many_arguments)]
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
            MRL => matches! (self.0[MRL as usize], Capacity::Quantity { .. }),
            HLT => matches! (self.0[HLT as usize], Capacity::Quantity { .. }),
            SPL => matches! (self.0[SPL as usize], Capacity::Quantity { .. }),
            ATK => matches! (self.0[ATK as usize], Capacity::Constant { .. }),
            DEF => matches! (self.0[DEF as usize], Capacity::Constant { .. }),
            MAG => matches! (self.0[MAG as usize], Capacity::Constant { .. }),
            MOV => matches! (self.0[MOV as usize], Capacity::Constant { .. }),
            ORG => matches! (self.0[ORG as usize], Capacity::Quantity { .. }),
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

    fn set_statistic (&mut self, statistic: UnitStatistic, value: u16) {
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

    fn change_statistic_flat (&mut self, statistic: UnitStatistic, change: u16, is_add: bool) {
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
            current.saturating_sub (change)
        };

        self.set_statistic (statistic, value);
    }

    fn change_statistic_percentage (&mut self, statistic: UnitStatistic, change: u16, is_add: bool) {
        assert! (self.validate_statistic (statistic));

        let base: f32 = match self.0[statistic as usize] {
            Capacity::Constant (_, _, b) => b,
            Capacity::Quantity (_, m) => m,
        } as f32;
        let change: f32 = (change as f32) / 100.0;
        let change: u16 = (base * change) as u16;

        self.change_statistic_flat (statistic, change, is_add);
    }

    pub fn calculate_damage (attacker: &Self, defender: &Self, weapon: &Weapon) -> (u16, u16, u16) {
        let dmg_weapon: u16 = weapon.get_statistic (WeaponStatistic::DMG);
        let slh_weapon: u16 = weapon.get_statistic (WeaponStatistic::SLH);
        let prc_weapon: u16 = weapon.get_statistic (WeaponStatistic::PRC);
        let dcy_weapon: u16 = weapon.get_statistic (WeaponStatistic::DCY);

        let hlt_attacker: (u16, u16) = attacker.get_statistic (HLT);
        let spl_attacker: (u16, u16) = attacker.get_statistic (SPL);
        let atk_attacker: u16 = attacker.get_statistic (ATK).0;
        let mag_attacker: u16 = attacker.get_statistic (MAG).0;
        let org_attacker: u16 = attacker.get_statistic (ORG).0;

        let mrl_defender: (u16, u16) = defender.get_statistic (MRL);
        let spl_defender: (u16, u16) = defender.get_statistic (SPL);
        let def_defender: u16 = defender.get_statistic (DEF).0;
        let mag_defender: u16 = defender.get_statistic (MAG).0;
        let org_defender: u16 = defender.get_statistic (ORG).0;

        let damage_weapon: f32 = {
            let damage: f32 = (atk_attacker + dmg_weapon) as f32;
            let multiplier: f32 = (spl_attacker.0 as f32) / (spl_attacker.1 as f32);

            damage * multiplier
        };
        let damage_bonus: u16 = {
            let damage: u16 = u16::max (mag_attacker.checked_sub (mag_defender).unwrap_or (1), 1);
            let multiplier: u16 = (dcy_weapon * 2) + 1;

            damage * multiplier
        };
        let multiplier: f32 = {
            let multiplier_mrl: f32 = 1.0 - (mrl_defender.0 as f32) / (mrl_defender.1 as f32);
            let multiplier_hlt: f32 = (hlt_attacker.0 as f32) / (hlt_attacker.1 as f32);
            let multiplier_org: f32 = (org_attacker as f32) / (PERCENT_100 as f32);

            multiplier_mrl + multiplier_hlt + multiplier_org
        };
        let reducer: u16 = {
            let multiplier_spl: f32 = (spl_defender.0 as f32) / (spl_defender.1 as f32);
            let multiplier_org: f32 = (org_defender as f32) / (PERCENT_100 as f32);
            let multiplier: f32 = (multiplier_spl + multiplier_org) / 2.0;

            ((def_defender as f32) * multiplier) as u16
        };
        let damage_base: u16 = u16::max ((damage_weapon * multiplier) as u16 - reducer, 1);
        let damage_mrl: u16 = (damage_base * (slh_weapon + 1)) + damage_bonus;
        let damage_hlt: u16 = damage_base + (damage_bonus * (prc_weapon + 1));
        let damage_spl: u16 = damage_base + damage_bonus + slh_weapon + prc_weapon + dcy_weapon;

        (damage_mrl, damage_hlt, damage_spl)
    }
}

#[derive (Debug)]
pub struct Unit {
    id: ID,
    scene: Rc<Scene>,
    statistics: UnitStatistics,
    modifier_terrain_id: Option<ID>,
    modifiers: Vec<Modifier>,
    statuses: HashMap<Trigger, Vec<Status>>,
    weapons: Vec<Weapon>,
    skill_passive_id: Option<ID>,
    skills: Vec<Skill>,
    magic_ids: Vec<ID>,
    weapon_active: usize,
    faction_id: ID,
    rank: Rank,
    is_alive: bool,
}

impl Unit {
    #[allow (clippy::too_many_arguments)]
    pub fn new (id: ID, scene: Rc<Scene>, statistics: UnitStatistics, weapons: &[ID], skill_passive_id: Option<ID>, skill_ids: &[ID], magics_usable: &[bool; Element::Length as usize], faction_id: ID, rank: Rank) -> Self {
        let modifier_terrain_id: Option<ID> = None;
        let modifiers: Vec<Modifier> = Vec::new ();
        let mut statuses: HashMap<Trigger, Vec<Status>> = HashMap::new ();
        let weapons: Vec<Weapon> = weapons.iter ()
                .map (|w: &ID| *scene.get_weapon (w)).collect ();
        let skills: Vec<Skill> = skill_ids.iter ()
                .map (|s: &ID| *scene.get_skill (s)).collect ();
        let mut magic_ids: Vec<ID> = Vec::new ();
        let weapon_active: usize = 0;
        let is_alive: bool = true;

        statuses.insert (Trigger::None, Vec::new ());

        for magic in scene.magics_iter () {
            if magics_usable[magic.get_element () as usize] && statistics.get_statistic (MAG).0 >= magic.get_cost () {
                magic_ids.push (magic.get_id ());
            }
        }

        Self { id, scene, statistics, modifier_terrain_id, modifiers, statuses, magic_ids, skill_passive_id, skills, weapons, weapon_active, faction_id, rank, is_alive }
    }

    pub fn initialise (&mut self) {
        self.apply_inactive_skills ();
    }

    pub fn get_statistic (&self, statistic: UnitStatistic) -> (u16, u16) {
        self.statistics.get_statistic (statistic)
    }

    pub fn set_statistic (&mut self, statistic: UnitStatistic, value: u16) {
        self.statistics.set_statistic (statistic, value);
    }

    fn change_statistic_flat (&mut self, statistic: UnitStatistic, change: u16, is_add: bool) {
        self.statistics.change_statistic_flat (statistic, change, is_add);

        self.update_is_alive ();
    }

    fn change_statistic_percentage (&mut self, statistic: UnitStatistic, change: u16, is_add: bool) {
        self.statistics.change_statistic_percentage (statistic, change, is_add);

        self.update_is_alive ();
    }

    pub fn apply_inactive_skills (&mut self) {
        if let Some (s) = self.skill_passive_id {
            let status_passive_id: ID = self.scene.get_skill (&s)
                    .get_status_id ();
            let status_passive: Status = *self.scene.get_status (&status_passive_id);
            self.add_status (status_passive);
        }

        let statuses_toggle: Vec<Status> = self.skills.iter ()
                .filter_map (|s: &Skill| 
                    if s.is_toggle () {
                        let status_id: ID = s.get_status_id ();
                        let status: Status = *self.scene.get_status (&status_id);

                        Some (status)
                    } else {
                        None
                    }
                ).collect ();

        for status_toggle in statuses_toggle {
            self.add_status (status_toggle);
        }
    }

    pub fn change_modifier_terrain (&mut self, modifier_id: Option<ID>) {
        if let Some (m) = self.modifier_terrain_id {
            self.remove_modifier (&m);
        }

        self.modifier_terrain_id = match modifier_id {
            Some (m) => {
                let modifier: Modifier = self.scene.get_modifier_builder (&m)
                        .build (true);
                let appliable: Box<dyn Appliable> = Box::new (modifier);
    
                self.add_appliable (appliable);
    
                modifier_id
            }
            None => None,
        };
    }

    pub fn set_leader (&mut self, leader_id: ID) {
        if let Rank::Follower ( .. ) = self.rank {
            self.rank = Rank::Follower (leader_id);
        }
    }

    pub fn try_add_passive (&mut self, skill_id: &ID, distance: usize) -> bool {
        if let Rank::Follower ( .. ) = self.rank {
            let status_id: &ID = &self.scene.get_skill (skill_id)
                    .get_status_id();
            let org: u16 = self.get_statistic (ORG).0;
            let multiplier: f32 = (org / PERCENT_100) as f32;
            let threshold: usize = ((THRESHOLD_SKILL_PASSIVE as f32) * multiplier) as usize;

            if distance > threshold {
                self.remove_status (status_id);
                self.skill_passive_id = None;

                false
            } else if self.skill_passive_id.is_none () {
                let status: Status = *self.scene.get_status (status_id);

                self.skill_passive_id = Some (*skill_id);

                self.add_status (status)
            } else {
                false
            }
        } else {
            false
        }
    }

    pub fn is_retreat (&self) -> bool {
        let mrl: u16 = self.get_statistic (MRL).0;

        mrl < THRESHOLD_RETREAT_MRL
    }

    pub fn is_rout (&self) -> bool {
        let mrl: u16 = self.get_statistic (MRL).0;

        mrl < THRESHOLD_ROUT_MRL
    }

    pub fn start_turn (&mut self) {
        let mut appliables: Vec<Box<dyn Appliable>> = Vec::new ();

        for (_, collection) in self.statuses.iter () {
            for status in collection.iter () {
                if status.is_every_turn () {
                    let appliable: Box<dyn Appliable> = status.get_change ()
                            .appliable (Rc::clone (&self.scene));

                    appliables.push (appliable);
                }
            }
        }

        for appliable in appliables {
            self.add_appliable (appliable);
        }

        self.update_is_alive ();
    }

    pub fn switch_weapon (&mut self) -> ID {
        self.weapon_active = (self.weapon_active + 1) % self.weapons.len ();

        self.weapon_active
    }

    pub fn act_attack (&mut self) -> (u16, Option<Box<dyn Appliable>>) {
        let drain_spl: u16 = (DRAIN_SPL * MULTIPLIER_ATTACK) as u16;

        self.change_statistic_flat (SPL, drain_spl, false);
        self.update_is_alive ();

        (self.get_statistic (MOV).0, self.get_weapon ().try_yield_appliable (Rc::clone (&self.scene)))
    }

    pub fn take_damage (&mut self, damage_mrl: u16, damage_hlt: u16, damage_spl: u16) -> Option<Box<dyn Appliable>> {
        self.change_statistic_flat (MRL, damage_mrl, false);
        self.change_statistic_flat (HLT, damage_hlt, false);
        self.change_statistic_flat (SPL, damage_spl, false);
        self.update_is_alive ();

        self.try_yield_appliable (Rc::clone (&self.scene))
    }

    pub fn act_skill (&mut self, skill_id: &ID) -> (u16, &Skill) {
        let drain_spl: u16 = (DRAIN_SPL * MULTIPLIER_SKILL) as u16;
        let index: usize = self.skills.iter ()
                .position (|s: &Skill| s.get_id () == *skill_id)
                .unwrap_or_else (|| panic! ("Skill {:?} not found", skill_id));
        let status_id_old: Option<ID> = {
            let skill: &mut Skill = &mut self.skills[index];
    
            if skill.is_toggle () {
                let (status_id_old, _): (ID, ID) = skill.switch_status ();

                Some (status_id_old)
            } else {
                None
            }
        };

        if let Some (s) = status_id_old {
            self.remove_status (&s);
        }

        self.change_statistic_flat (SPL, drain_spl, false);
        self.update_is_alive ();

        (self.get_statistic (MOV).0, &self.skills[index])
    }

    pub fn act_magic (&mut self, magic_id: &ID) -> (u16, &Magic) {
        assert! (self.magic_ids.contains (magic_id));

        let mag: u16 = self.get_statistic (MAG).0;
        let cost: u16 = self.scene.get_magic (magic_id)
                .get_cost ();
        let drain_hlt: u16 = u16::max ((cost * DRAIN_HLT).saturating_sub (mag), 1);
        let drain_org: u16 = u16::max ((cost * PERCENT_1).saturating_sub (mag / PERCENT_1), PERCENT_1);
        let drain_spl: u16 = (DRAIN_SPL * MULTIPLIER_MAGIC) as u16;

        self.change_statistic_flat (HLT, drain_hlt, false);
        self.change_statistic_flat (ORG, drain_org, false);
        self.change_statistic_flat (SPL, drain_spl, false);
        self.update_is_alive ();

        (self.get_statistic (MOV).0, self.scene.get_magic (magic_id))
    }

    pub fn act_wait (&mut self) -> u16 {
        let drain_spl: u16 = (DRAIN_SPL * MULTIPLIER_WAIT) as u16;

        self.change_statistic_flat (SPL, drain_spl, false);
        // self.update_is_dead (); // No change to HLT

        self.get_statistic (MOV).0
    }

    fn recover_supplies (&mut self, city_ids: &[ID]) {
        if !self.is_retreat () && !city_ids.is_empty () {
            let mut recover_hlt: u16 = 0;
            let mut recover_spl: u16 = 0;

            for city_id in city_ids {
                let change_hlt: u16 = self.scene.get_city (city_id)
                        .get_manpower ();
                let change_spl: u16 = self.scene.get_city (city_id)
                        .get_equipment ();

                recover_hlt += change_hlt;
                recover_spl += change_spl;
            }

            self.change_statistic_flat (HLT, recover_hlt, true);
            self.change_statistic_flat (SPL, recover_spl, true);
        }
    }

    pub fn end_turn (&mut self, city_ids: &[ID], appliable: Option<Box<dyn Appliable>>) {
        self.recover_supplies (city_ids);
        self.change_statistic_flat (MRL, RECOVER_MRL, true);
        self.decrement_durations ();

        if let Some (a) = appliable {
            self.add_appliable (a);
        }

        self.update_is_alive ();
    }

    pub fn get_id (&self) -> ID {
        self.id
    }

    pub fn get_statistics (&self) -> UnitStatistics {
        self.statistics
    }

    pub fn get_weapon (&self) -> &Weapon {
        &self.weapons[self.weapon_active]
    }

    pub fn get_faction_id (&self) -> ID {
        self.faction_id
    }

    pub fn get_skill_passive_id (&self) -> Option<ID> {
        self.skill_passive_id
    }

    pub fn get_leader_id (&self) -> ID {
        match self.rank {
            Rank::Leader => self.id,
            Rank::Follower (l) => l,
        }
    }

    pub fn is_alive (&self) -> bool {
        self.is_alive
    }

    fn update_is_alive (&mut self) -> bool {
        self.is_alive = self.get_statistic (HLT).0 > 0;

        self.is_alive
    }
}

impl Applier for Unit {
    fn try_yield_appliable (&self, scene: Rc<Scene>) -> Option<Box<dyn Appliable>> {
        self.statuses.get (&Trigger::OnHit).and_then (|c: &Vec<Status>|
            c.first ().and_then (|s: &Status| s.try_yield_appliable (scene))
        )
    }

    fn get_target (&self) -> Target {
        Target::Enemy
    }
}

impl Changeable for Unit {
    fn add_appliable (&mut self, appliable: Box<dyn Appliable>) -> bool {
        let change: Change = appliable.change ();

        match change {
            Change::Modifier ( .. ) => {
                let modifier: Modifier = appliable.modifier ();

                if modifier.can_stack () || !self.modifiers.contains (&modifier){
                    for adjustment in modifier.get_adjustments ().iter () {
                        if let Statistic::Unit (s) = adjustment.0 {
                            self.change_statistic_percentage (s, adjustment.1, adjustment.2);
                        }
                    }

                    self.modifiers.push (modifier);

                    true
                } else {
                    false
                }
            }
            Change::Effect ( .. ) => {
                let effect: Effect = appliable.effect ();

                for adjustment in effect.get_adjustments ().iter () {
                    if let Statistic::Unit (s) = adjustment.0 {
                        if effect.can_stack_or_is_flat () {
                            self.change_statistic_flat (s, adjustment.1, adjustment.2);
                        } else {
                            self.change_statistic_percentage (s, adjustment.1, adjustment.2);
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
            let appliable: Box<dyn Appliable> = status.try_yield_appliable (Rc::clone (&self.scene))
                    .unwrap_or_else (|| panic! ("Appliable not found for status {:?}", status));

            match trigger {
                Trigger::OnAttack => {
                    let weapon: &mut Weapon = &mut self.weapons[self.weapon_active];

                    weapon.add_status (status);

                    true
                }
                Trigger::OnHit => {
                    if let Target::Enemy = status.get_target () {
                        let collection: Vec<Status> = vec![status];

                        self.statuses.insert (trigger, collection);
    
                        true
                    } else {
                        false
                    }
                }
                Trigger::None => {
                    if let Target::This = status.get_target () {
                        let collection: &mut Vec<Status> = self.statuses.get_mut (&trigger)
                                .unwrap_or_else (|| panic! ("Collection not found for trigger {:?}", trigger));

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

    fn remove_modifier (&mut self, modifier_id: &ID) -> bool {
        let index: Option<usize> = self.modifiers.iter ()
                .position (|m: &Modifier| m.get_id () == *modifier_id);

        if let Some (i) = index {
            let modifier: Modifier = self.modifiers.swap_remove (i);

            for adjustment in modifier.get_adjustments ().iter () {
                if let Statistic::Unit (s) = adjustment.0 {
                    self.change_statistic_percentage (s, adjustment.1, !adjustment.2);
                }
            }

            true
        } else {
            false
        }
    }

    fn remove_status (&mut self, status_id: &ID) -> bool {
        for (_, collection) in self.statuses.iter_mut () {
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

    fn decrement_durations (&mut self) {
        let mut statuses_expired: Vec<Status> = Vec::new ();

        self.modifiers.retain_mut (|m: &mut Modifier| !m.decrement_duration ());

        for (_, collection) in self.statuses.iter_mut () {
            for status in collection.iter_mut () {
                if status.decrement_duration () {
                    statuses_expired.push (*status);
                }
            }

            collection.retain (|s: &Status| !s.is_expired ());
        }

        for status_expired in statuses_expired {
            if let Change::Modifier (m, _) = status_expired.get_change () {
                self.remove_modifier (&m);

                if let Some (n) = status_expired.get_next_id () {
                    let status_next: Status = *self.scene.get_status (&n);

                    self.add_status (status_next);
                }
            }
        }
    }
}

#[derive (Debug)]
pub struct UnitBuilder {
    id: ID,
    statistics: UnitStatistics,
    weapon_ids: &'static [ID],
    skill_passive_id: Option<ID>,
    skill_ids: &'static [ID],
    magics_usable: [bool; Element::Length as usize],
    faction_id: ID,
    rank: Rank,
}

impl UnitBuilder {
    #[allow (clippy::too_many_arguments)]
    pub const fn new (id: ID, statistics: UnitStatistics, weapon_ids: &'static [ID], skill_passive_id: Option<ID>, skill_ids: &'static [ID], magics_usable: [bool; Element::Length as usize], faction_id: ID, rank: Rank) -> Self {
        Self { id, statistics, weapon_ids, skill_passive_id, skill_ids, magics_usable, faction_id, rank }
    }

    pub fn build (&self, scene: Rc<Scene>) -> Unit {
        Unit::new (self.id, scene, self.statistics, self.weapon_ids, self.skill_passive_id, self.skill_ids, &self.magics_usable, self.faction_id, self.rank)
    }

    pub fn get_id (&self) -> ID {
        self.id
    }

    pub fn get_faction_id (&self) -> ID {
        self.faction_id
    }
}

#[cfg (test)]
mod tests {
    use super::*;
    use crate::tests::generate_scene;

    fn generate_units () -> (Unit, Unit, Unit) {
        let scene = generate_scene ();
        let unit_builder_0 = scene.get_unit_builder (&0);
        let unit_0 = unit_builder_0.build (Rc::clone (&scene));
        let unit_builder_1 = scene.get_unit_builder (&1);
        let unit_1 = unit_builder_1.build (Rc::clone (&scene));
        let unit_builder_2 = scene.get_unit_builder (&2);
        let unit_2 = unit_builder_2.build (Rc::clone (&scene));

        (unit_0, unit_1, unit_2)
    }

    fn generate_modifiers () -> (Box<Modifier>, Box<Modifier>) {
        let scene = generate_scene ();
        let modifier_builder_3 = scene.get_modifier_builder (&3);
        let modifier_3 = modifier_builder_3.build (true);
        let modifier_3 = Box::new (modifier_3);
        let modifier_builder_4 = scene.get_modifier_builder (&4);
        let modifier_4 = modifier_builder_4.build (false);
        let modifier_4 = Box::new (modifier_4);

        (modifier_3, modifier_4)
    }

    fn generate_effects () -> (Box<Effect>, Box<Effect>) {
        let scene = generate_scene ();
        let effect_0 = *scene.get_effect (&0);
        let effect_0 = Box::new (effect_0);
        let effect_1 = *scene.get_effect (&1);
        let effect_1 = Box::new (effect_1);

        (effect_0, effect_1)
    }

    fn generate_statuses () -> (Status, Status, Status) {
        let scene = generate_scene ();
        let status_0 = *scene.get_status (&0);
        let status_1 = *scene.get_status (&1);
        let status_5 = *scene.get_status (&5);

        (status_0, status_1, status_5)
    }

    #[test]
    fn unit_change_statistic_flat () {
        let (mut unit_0, _, _) = generate_units ();

        // Test constant
        unit_0.change_statistic_flat (ATK, 5, true); // Test additive change
        assert_eq! (unit_0.get_statistic (ATK).0, 25);
        unit_0.change_statistic_flat (ATK, 5, false); // Test subtractive change
        assert_eq! (unit_0.get_statistic (ATK).0, 20);
        unit_0.change_statistic_flat (ATK, ATK_MAX, true); // Test maximum change
        assert_eq! (unit_0.get_statistic (ATK).0, ATK_MAX);
        unit_0.change_statistic_flat (ATK, ATK_MAX, false); // Test minimum change
        assert_eq! (unit_0.get_statistic (ATK).0, 0);
        // Test quantity
        unit_0.change_statistic_flat (HLT, 10, false); // Test subtractive change
        assert_eq! (unit_0.get_statistic (HLT).0, 990);
        unit_0.change_statistic_flat (HLT, 5, true); // Test additive change
        assert_eq! (unit_0.get_statistic (HLT).0, 995);
        unit_0.change_statistic_flat (HLT, HLT_MAX, true); // Test maximum change
        assert_eq! (unit_0.get_statistic (HLT).0, HLT_MAX);
        unit_0.change_statistic_flat (HLT, HLT_MAX, false); // Test minimum change
        assert_eq! (unit_0.get_statistic (HLT).0, 0);
    }

    #[test]
    fn unit_change_statistic_percentage () {
        let (mut unit_0, _, _) = generate_units ();

        // Test constant
        unit_0.change_statistic_percentage (ATK, 10, true); // Test additive change
        assert_eq! (unit_0.get_statistic (ATK).0, 22);
        unit_0.change_statistic_percentage (ATK, 10, false); // Test subtractive change
        assert_eq! (unit_0.get_statistic (ATK).0, 20);
        unit_0.change_statistic_percentage (ATK, 1000, true); // Test maximum change
        assert_eq! (unit_0.get_statistic (ATK).0, ATK_MAX);
        unit_0.change_statistic_percentage (ATK, 1000, false); // Test minimum change
        assert_eq! (unit_0.get_statistic (ATK).0, 0);
        // Test quantity
        unit_0.change_statistic_percentage (HLT, 10, false); // Test subtractive change
        assert_eq! (unit_0.get_statistic (HLT).0, 900);
        unit_0.change_statistic_percentage (HLT, 5, true); // Test additive change
        assert_eq! (unit_0.get_statistic (HLT).0, 950);
        unit_0.change_statistic_percentage (HLT, 1000, true); // Test maximum change
        assert_eq! (unit_0.get_statistic (HLT).0, HLT_MAX);
        unit_0.change_statistic_percentage (HLT, 1000, false); // Test minimum change
        assert_eq! (unit_0.get_statistic (HLT).0, 0);
    }

    #[test]
    fn unit_apply_inactive_skills () {
        let (mut unit_0, _, _) = generate_units ();

        unit_0.apply_inactive_skills ();
        assert_eq! (unit_0.statuses.get (&Trigger::None).unwrap ().len (), 2);
        assert_eq! (unit_0.modifiers.len (), 2);
    }

    #[test]
    fn unit_change_modifier_terrain () {
        let (mut unit_0, _, _) = generate_units ();

        // Test empty modifier
        unit_0.change_modifier_terrain (None);
        assert! (unit_0.modifier_terrain_id.is_none ());
        assert! (unit_0.modifiers.is_empty ());
        // Test non-empty modifier
        unit_0.change_modifier_terrain (Some (3));
        assert_eq! (unit_0.modifier_terrain_id.unwrap (), 3);
        assert_eq! (unit_0.modifiers.len (), 1);
    }

    #[test]
    fn unit_set_leader () {
        let (mut unit_0, mut unit_1, _) = generate_units ();

        // Test leader set
        unit_0.set_leader (1);
        assert! (matches! (unit_0.rank, Rank::Leader));
        // Test follower set
        unit_1.set_leader (0);
        assert! (matches! (unit_1.rank, Rank::Follower (0)));
    }

    #[test]
    fn unit_try_add_passive () {
        let scene = generate_scene ();
        let (mut unit_0, mut unit_1, _) = generate_units ();
        let skill_passive_id = unit_0.skill_passive_id.unwrap ();
        let skill_passive = scene.get_skill (&skill_passive_id);
        let status_passive_id = skill_passive.get_status_id ();

        // Test leader add
        assert! (!unit_0.try_add_passive (&status_passive_id, 0));
        // Test near add
        assert! (unit_1.try_add_passive (&status_passive_id, 1));
        assert_eq! (unit_1.skill_passive_id.unwrap (), status_passive_id);
        assert_eq! (unit_1.statuses.get (&Trigger::None).unwrap ().len (), 1);
        assert_eq! (unit_1.modifiers.len (), 1);
        // Test far add
        assert! (!unit_1.try_add_passive (&status_passive_id, 2));
        assert! (unit_1.skill_passive_id.is_none ());
        assert! (unit_1.statuses.get (&Trigger::None).unwrap ().is_empty ());
        assert! (unit_1.modifiers.is_empty ());
    }

    #[test]
    fn unit_start_turn () {
        let scene = generate_scene ();
        let (mut unit_0, _, _) = generate_units ();
        let (status_0, _, _) = generate_statuses ();
        let status_8 = *scene.get_status (&8);

        // Test normal status
        unit_0.add_status (status_0);
        assert_eq! (unit_0.get_statistic (ATK).0, 24);
        assert_eq! (unit_0.get_statistic (DEF).0, 20);
        unit_0.start_turn ();
        assert_eq! (unit_0.get_statistic (ATK).0, 24);
        assert_eq! (unit_0.get_statistic (DEF).0, 20);
        // Test repeatable status
        unit_0.add_status (status_8);
        assert_eq! (unit_0.get_statistic (ATK).0, 26);
        assert_eq! (unit_0.get_statistic (DEF).0, 18);
        unit_0.start_turn ();
        assert_eq! (unit_0.get_statistic (ATK).0, 28);
        assert_eq! (unit_0.get_statistic (DEF).0, 16);
    }

    #[test]
    fn unit_act_attack () {
        let scene = generate_scene ();
        let (mut unit_0, _, _) = generate_units ();
        let status = *scene.get_status (&6);

        // Test normal attack
        let spl_0_0 = unit_0.get_statistic (SPL).0;
        let response = unit_0.act_attack ();
        assert_eq! (response.0, 10);
        assert! (response.1.is_none ());
        let spl_0_1 = unit_0.get_statistic (SPL).0;
        assert! (spl_0_0 > spl_0_1);
        // Test OnAttack attack
        unit_0.add_status (status);
        let response = unit_0.act_attack ();
        assert_eq! (response.0, 10);
        assert! (response.1.is_some ());
        let spl_0_2 = unit_0.get_statistic (SPL).0;
        assert! (spl_0_1 > spl_0_2);
    }

    #[test]
    fn unit_take_damage () {
        let scene = generate_scene ();
        let (mut unit_0, _, mut unit_2) = generate_units ();
        let statistics_0 = unit_0.statistics;
        let statistics_2 = unit_2.statistics;
        let weapon = *unit_0.get_weapon ();
        let status = *scene.get_status (&5);

        // Test normal attack
        let (damage_mrl, damage_hlt, damage_spl) = UnitStatistics::calculate_damage (&statistics_0, &statistics_2, &weapon);
        let mrl_2_0 = unit_2.get_statistic (MRL).0;
        let hlt_2_0 = unit_2.get_statistic (HLT).0;
        let spl_2_0 = unit_2.get_statistic (SPL).0;
        assert! (unit_2.take_damage (damage_mrl, damage_hlt, damage_spl).is_none ());
        let mrl_2_1 = unit_2.get_statistic (MRL).0;
        let hlt_2_1 = unit_2.get_statistic (HLT).0;
        let spl_2_1 = unit_2.get_statistic (SPL).0;
        assert_eq! (mrl_2_0 - damage_mrl, mrl_2_1);
        assert_eq! (hlt_2_0 - damage_hlt, hlt_2_1);
        assert_eq! (spl_2_0 - damage_spl, spl_2_1);
        // Test OnHit attack
        unit_0.add_status (status);
        let weapon = *unit_2.get_weapon ();
        let (damage_mrl, damage_hlt, damage_spl) = UnitStatistics::calculate_damage (&statistics_2, &statistics_0, &weapon);
        let mrl_0_0 = unit_0.get_statistic (MRL).0;
        let hlt_0_0 = unit_0.get_statistic (HLT).0;
        let spl_0_0 = unit_0.get_statistic (SPL).0;
        assert! (unit_0.take_damage (damage_mrl, damage_hlt, damage_spl).is_some ());
        let mrl_0_1 = unit_0.get_statistic (MRL).0;
        let hlt_0_1 = unit_0.get_statistic (HLT).0;
        let spl_0_1 = unit_0.get_statistic (SPL).0;
        assert_eq! (mrl_0_0 - damage_mrl, mrl_0_1);
        assert_eq! (hlt_0_0 - damage_hlt, hlt_0_1);
        assert_eq! (spl_0_0 - damage_spl, spl_0_1);
        // Test switch attack
        assert_eq! (unit_2.switch_weapon (), 1);
        let weapon = *unit_2.get_weapon ();
        let (damage_mrl, damage_hlt, damage_spl) = UnitStatistics::calculate_damage (&statistics_2, &statistics_0, &weapon);
        assert! (unit_0.take_damage (damage_mrl, damage_hlt, damage_spl).is_some ());
        let mrl_0_2 = unit_0.get_statistic (MRL).0;
        let hlt_0_2 = unit_0.get_statistic (HLT).0;
        let spl_0_2 = unit_0.get_statistic (SPL).0;
        assert_eq! (mrl_0_1 - damage_mrl, mrl_0_2);
        assert_eq! (hlt_0_1 - damage_hlt, hlt_0_2);
        assert_eq! (spl_0_1 - damage_spl, spl_0_2);
    }

    #[test]
    fn unit_act_skill () {
        let (mut unit_0, _, _) = generate_units ();

        let spl_0_0 = unit_0.get_statistic (SPL).0;
        let response = unit_0.act_skill (&0);
        assert_eq! (response.0, 10);
        assert_eq! (response.1.get_id (), 0);
        let spl_0_1 = unit_0.get_statistic (SPL).0;
        assert! (spl_0_0 > spl_0_1);
    }

    #[test]
    fn unit_act_magic () {
        let (mut unit_0, _, _) = generate_units ();

        let hlt_0_0 = unit_0.get_statistic (HLT).0;
        let org_0_0 = unit_0.get_statistic (ORG).0;
        let spl_0_0 = unit_0.get_statistic (SPL).0;
        let response = unit_0.act_magic (&0);
        assert_eq! (response.0, 10);
        assert_eq! (response.1.get_status_id (), 8);
        let hlt_0_1 = unit_0.get_statistic (HLT).0;
        let org_0_1 = unit_0.get_statistic (ORG).0;
        let spl_0_1 = unit_0.get_statistic (SPL).0;
        assert! (hlt_0_0 > hlt_0_1);
        assert! (org_0_0 > org_0_1);
        assert! (spl_0_0 > spl_0_1);
    }

    #[test]
    fn unit_recover_supplies () {
        let (mut unit_0, _, _) = generate_units ();

        unit_0.change_statistic_flat (HLT, 500, false);
        unit_0.change_statistic_flat (SPL, 500, false);

        // Test encircled recover
        let hlt_0_0 = unit_0.get_statistic (HLT).0;
        let spl_0_0 = unit_0.get_statistic (SPL).0;
        unit_0.recover_supplies (&[]);
        let hlt_0_1 = unit_0.get_statistic (HLT).0;
        let spl_0_1 = unit_0.get_statistic (SPL).0;
        assert_eq! (hlt_0_0, hlt_0_1);
        assert_eq! (spl_0_0, spl_0_1);
        // Test normal recover
        unit_0.recover_supplies (&[0]);
        let hlt_0_2 = unit_0.get_statistic (HLT).0;
        let spl_0_2 = unit_0.get_statistic (SPL).0;
        assert! (hlt_0_1 < hlt_0_2);
        assert! (spl_0_1 < spl_0_2);
    }

    #[test]
    fn unit_end_turn () {
        let scene = generate_scene ();
        let (mut unit_0, _, _) = generate_units ();
        let effect_0 = *scene.get_effect (&0);
        let effect_0 = Box::new (effect_0) as Box<dyn Appliable>;
        let effect_0 = Some (effect_0);

        unit_0.change_statistic_flat (MRL, 500, false);
        unit_0.change_statistic_flat (HLT, 500, false);
        unit_0.change_statistic_flat (SPL, 500, false);

        // Test encircled recover
        let mrl_0_0 = unit_0.get_statistic (MRL).0;
        let hlt_0_0 = unit_0.get_statistic (HLT).0;
        let spl_0_0 = unit_0.get_statistic (SPL).0;
        unit_0.end_turn (&[], effect_0);
        let mrl_0_1 = unit_0.get_statistic (MRL).0;
        let hlt_0_1 = unit_0.get_statistic (HLT).0;
        let spl_0_1 = unit_0.get_statistic (SPL).0;
        assert_eq! (mrl_0_0 + RECOVER_MRL, mrl_0_1);
        assert! (hlt_0_0 > hlt_0_1);
        assert_eq! (spl_0_0, spl_0_1);
        // Test normal recover
        unit_0.end_turn (&[1], None);
        let mrl_0_2 = unit_0.get_statistic (MRL).0;
        let hlt_0_2 = unit_0.get_statistic (HLT).0;
        let spl_0_2 = unit_0.get_statistic (SPL).0;
        assert_eq! (mrl_0_1 + RECOVER_MRL, mrl_0_2);
        assert! (hlt_0_1 < hlt_0_2);
        assert! (spl_0_1 < spl_0_2);
    }

    #[test]
    fn unit_add_appliable () {
        let (mut unit_0, mut unit_1, _) = generate_units ();
        let (modifier_3, modifier_4) = generate_modifiers ();
        let (effect_0, effect_1) = generate_effects ();

        // Test additive modifier
        assert! (unit_0.add_appliable (modifier_3.clone ()));
        assert_eq! (unit_0.modifiers.len (), 1);
        assert_eq! (unit_0.get_statistic (ATK).0, 24);
        // Test subtractive modifier
        assert! (unit_0.add_appliable (modifier_4.clone ())); // Test multiple adjustments
        assert_eq! (unit_0.modifiers.len (), 2);
        assert_eq! (unit_0.get_statistic (ATK).0, 26);
        assert_eq! (unit_0.get_statistic (DEF).0, 18);
        // Test stacking modifier
        assert! (unit_0.add_appliable (modifier_3.clone ()));
        assert_eq! (unit_0.modifiers.len (), 3);
        assert_eq! (unit_0.get_statistic (ATK).0, 30);
        assert! (unit_0.add_appliable (modifier_3));
        assert_eq! (unit_0.modifiers.len (), 4);
        assert_eq! (unit_0.get_statistic (ATK).0, 34);
        // Test non-stacking modifier
        assert! (!unit_0.add_appliable (modifier_4));
        assert_eq! (unit_0.modifiers.len (), 4);
        assert_eq! (unit_0.get_statistic (ATK).0, 34);
        assert_eq! (unit_0.get_statistic (DEF).0, 18);

        // Test flat effect
        assert! (unit_1.add_appliable (effect_0));
        assert_eq! (unit_1.get_statistic (HLT).0, 998);
        // Test percentage effect
        assert! (unit_1.add_appliable (effect_1)); // Test multiple adjustments
        assert_eq! (unit_1.get_statistic (ATK).0, 21);
        assert_eq! (unit_1.get_statistic (DEF).0, 19);
    }

    #[test]
    fn unit_add_status () {
        let scene = generate_scene ();
        let (mut unit_0, _, _) = generate_units ();
        let (status_0, _, status_5) = generate_statuses ();
        let status_6 = *scene.get_status (&6);

        // Test unit status
        assert! (unit_0.add_status (status_0));
        assert_eq! (unit_0.get_statistic (ATK).0, 24);
        // Test applier status
        assert! (unit_0.add_status (status_5));
        assert_eq! (unit_0.statuses.get (&Trigger::OnHit).unwrap ().len (), 1);
        assert! (unit_0.try_yield_appliable (Rc::clone (&scene)).is_some ());
        // Test weapon status
        assert! (unit_0.add_status (status_6));
        assert! (unit_0.weapons[unit_0.weapon_active].try_yield_appliable (Rc::clone (&scene)).is_some ());
    }

    #[test]
    fn unit_remove_modifier () {
        let (mut unit_0, _, _) = generate_units ();
        let (modifier_3, _) = generate_modifiers ();

        // Test empty remove
        assert! (!unit_0.remove_modifier (&3));
        assert! (unit_0.modifiers.is_empty ());
        // Test non-empty remove
        unit_0.add_appliable (modifier_3);
        assert! (unit_0.remove_modifier (&3));
        assert! (unit_0.modifiers.is_empty ());
    }

    #[test]
    fn unit_remove_status () {
        let (mut unit_0, _, _) = generate_units ();
        let (status_0, _, status_5) = generate_statuses ();

        // Test empty remove
        assert! (!unit_0.remove_status (&0));
        assert! (unit_0.statuses.get (&Trigger::None).unwrap ().is_empty ());
        assert! (unit_0.modifiers.is_empty ());
        // Test non-empty remove
        unit_0.add_status (status_0);
        assert_eq! (unit_0.get_statistic (ATK).0, 24);
        assert! (unit_0.remove_status (&0));
        assert_eq! (unit_0.get_statistic (ATK).0, 20);
        assert! (unit_0.statuses.get (&Trigger::None).unwrap ().is_empty ());
        assert! (unit_0.modifiers.is_empty ());
        // Test applier remove
        unit_0.add_status (status_5);
        assert! (unit_0.remove_status (&5));
        assert! (unit_0.statuses.get (&Trigger::OnHit).unwrap ().is_empty ());
        assert! (unit_0.modifiers.is_empty ());
    }

    #[test]
    fn unit_decrement_durations () {
        let (mut unit_0, mut unit_1, _) = generate_units ();
        let (modifier_3, modifier_4) = generate_modifiers ();
        let (status_0, status_1, status_5) = generate_statuses ();

        // Test empty modifier
        unit_0.decrement_durations ();
        assert! (unit_0.modifiers.is_empty ());
        // Test timed modifier
        unit_0.add_appliable (modifier_3.clone ());
        unit_0.decrement_durations ();
        assert_eq! (unit_0.modifiers.len (), 1);
        unit_0.decrement_durations ();
        assert_eq! (unit_0.modifiers.len (), 1);
        unit_0.decrement_durations ();
        assert! (unit_0.modifiers.is_empty ());
        // Test permanent modifier
        unit_0.add_appliable (modifier_4);
        unit_0.decrement_durations ();
        assert_eq! (unit_0.modifiers.len (), 1);
        unit_0.decrement_durations ();
        assert_eq! (unit_0.modifiers.len (), 1);

        // Test empty status
        unit_1.decrement_durations ();
        assert! (unit_1.statuses.get (&Trigger::None).unwrap ().is_empty ());
        // Test timed status
        unit_1.add_status (status_1);
        unit_1.decrement_durations ();
        assert_eq! (unit_1.statuses.get (&Trigger::None).unwrap ().len (), 1);
        unit_1.decrement_durations ();
        assert_eq! (unit_1.statuses.get (&Trigger::None).unwrap ().len (), 1);
        unit_1.decrement_durations ();
        assert! (unit_1.statuses.get (&Trigger::None).unwrap ().is_empty ());
        // Test permanent status
        unit_1.add_status (status_0);
        unit_1.decrement_durations ();
        assert_eq! (unit_1.statuses.get (&Trigger::None).unwrap ().len (), 1);
        unit_1.decrement_durations ();
        assert_eq! (unit_1.statuses.get (&Trigger::None).unwrap ().len (), 1);
        unit_1.decrement_durations ();
        assert_eq! (unit_1.statuses.get (&Trigger::None).unwrap ().len (), 1);
        // Test linked status
        unit_1.add_status (status_5);
        unit_1.decrement_durations ();
        assert_eq! (unit_1.statuses.get (&Trigger::OnHit).unwrap ().len (), 1);
        assert_eq! (unit_1.statuses.get (&Trigger::OnHit).unwrap ()[0].get_next_id ().unwrap (), 0);
        unit_1.decrement_durations ();
        assert_eq! (unit_1.statuses.get (&Trigger::OnHit).unwrap ().len (), 1);
        assert_eq! (unit_1.statuses.get (&Trigger::OnHit).unwrap ()[0].get_next_id ().unwrap (), 0);
        unit_1.decrement_durations ();
        assert! (unit_1.statuses.get (&Trigger::OnHit).unwrap ().is_empty ());
        assert_eq! (unit_1.statuses.get (&Trigger::None).unwrap ().len (), 2);
        assert! (unit_1.statuses.get (&Trigger::None).unwrap ()[1].get_next_id ().is_none ());
    }
}
