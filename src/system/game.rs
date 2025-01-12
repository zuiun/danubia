use super::{ActionValidator, ConfirmationValidator, DirectionValidator, IDValidator, LocationValidator, MovementValidator, Reader, Turn};
use crate::character::{Faction, FactionBuilder, Magic, Skill, Tool, Unit, UnitBuilder, UnitStatistic, UnitStatistics, Weapon};
use crate::common::{FACTOR_ATTACK, FACTOR_MAGIC, FACTOR_SKILL, FACTOR_WAIT, ID, Target};
use crate::dynamic::{Appliable, Applier, Dynamic, Status};
// use crate::event::Handler;
use crate::map::{Area, Direction, Grid, Location, Search};
use crate::Scene;
// use std::cell::RefCell;
use std::collections::{BinaryHeap, HashSet};
use std::io::BufRead;
use std::rc::Rc;
use std::sync::mpsc::Sender;

/*
 * Calculated from build.rs
 * Unit MOV is an index into the table
 * Attack (* 1.0): 21 delay at 0, 20 delay at 1, 2 delay at 77, and 1 delay at 100
 * Skill/Magic (* 1.4): 29 delay at 0, 28 delay at 1, 2 delay at 77, and 1 delay at 100
 * Wait (* 0.67): 14 delay at 0, 13 delay at 1, 2 delay at 54, and 1 delay at 77
 */
const DELAYS: [u8; 101] = [21, 20, 19, 19, 18, 18, 17, 17, 16, 16, 15, 15, 14, 14, 14, 13, 13, 13, 12, 12, 11, 11, 11, 11, 10, 10, 10, 9, 9, 9, 9, 8, 8, 8, 8, 7, 7, 7, 7, 7, 7, 6, 6, 6, 6, 6, 6, 5, 5, 5, 5, 5, 5, 5, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 1];

fn get_delay (mov: u16, delay_multiplier: f32) -> u16 {
    let delay: f32 = DELAYS[mov as usize] as f32;

    (delay * delay_multiplier) as u16
}

pub enum Action {
    Attack,
    Weapon,
    Skill,
    Magic,
    Move,
    Wait,
}

#[derive (Debug)]
pub struct Game<R: BufRead> {
    scene: Rc<Scene>,
    // handler: Rc<RefCell<Handler>>,
    grid: Grid,
    units: Vec<Unit>,
    factions: Vec<Faction>,
    turns: BinaryHeap<Turn>,
    number_turns: usize,
    unit_ids_dirty: Vec<ID>,
    reader: Reader<R>,
    sender: Sender<String>,
}

impl<R: BufRead> Game<R> {
    pub fn new (scene: Scene, reader: Reader<R>, sender: Sender<String>) -> Self {
        let scene: Rc<Scene> = Rc::new (scene);
        // let handler: Handler = Handler::new ();
        // let handler: RefCell<Handler> = RefCell::new (handler);
        // let handler: Rc<RefCell<Handler>> = Rc::new (handler);
        let grid: Grid = Grid::new (Rc::clone (&scene));
        let units: Vec<Unit> = scene.unit_builders_iter ().map (|u: &UnitBuilder|
            u.build (Rc::clone (&scene))
        ).collect ();
        let factions: Vec<Faction> = scene.faction_builders_iter ().map (|f: &FactionBuilder|
            f.build (&units)
        ).collect ();
        let turns: BinaryHeap<Turn> = BinaryHeap::new ();
        let number_turns: usize = 0;
        let unit_ids_dirty: Vec<ID> = Vec::new ();
        let reader: Reader<R> = reader;

        let _ = sender.send (String::from ("Game initialisation complete"));

        Self { scene, /* handler, */ grid, units, factions, turns, number_turns, unit_ids_dirty, reader, sender }
    }

    fn apply_terrain (&mut self, unit_id: ID, terrain_id: ID, location: Location) {
        let modifier_terrain_id: Option<ID> = self.scene.get_terrain (&terrain_id).get_modifier_id ();
        let appliable: Option<Box<dyn Appliable>> = self.grid.try_yield_appliable (&location);

        self.units[unit_id].change_modifier_terrain (modifier_terrain_id);

        if let Some (a) = appliable {
            self.units[unit_id].add_appliable (a);
        }
    }

    fn add_turn (&mut self, unit_id: ID) {
        let mov: u16 = self.units[unit_id].get_statistic (UnitStatistic::MOV).0;
        let delay: u16 = get_delay (mov, FACTOR_WAIT);
        let turn: Turn = Turn::new (unit_id, delay, mov);

        self.turns.push (turn);
    }

    fn place_unit (&mut self, unit_id: ID, location: Location) {
        let terrain_id: ID = self.grid.place_unit (unit_id, location)
                .unwrap_or_else (|| panic! ("Terrain not found for location {:?}", location));

        self.apply_terrain (unit_id, terrain_id, location);
        self.units[unit_id].apply_inactive_skills ();
        self.add_turn (unit_id);
    }

    fn try_spawn_recruit (&mut self, unit_id: ID) {
        let location: Location = *self.grid.get_unit_location (&unit_id);
        let leader_id: ID = self.units[unit_id].get_leader_id ();
        let faction_id: ID = self.units[unit_id].get_faction_id ();

        if let Some ((recruit_id, terrain_id)) = self.grid.try_spawn_recruit (location, &faction_id) {
            let modifier_terrain_id: Option<ID> = self.scene.get_terrain (&terrain_id).get_modifier_id ();

            self.factions[faction_id].add_follower (recruit_id, leader_id);
            self.units[recruit_id].set_leader_id (unit_id);
            self.units[recruit_id].change_modifier_terrain (modifier_terrain_id);
            self.add_turn (recruit_id);
            // self.units[r].apply_inactive_skills ();
        }
    }

    fn send_passive (&mut self, unit_id: ID) {
        let leader_id: ID = self.units[unit_id].get_leader_id ();
        let faction_id: ID = self.scene.get_unit_builder (&unit_id).get_faction_id ();
        let follower_ids: &HashSet<ID> = self.factions[faction_id].get_followers (&leader_id);
        let skill_passive_id: ID = self.units[leader_id].get_skill_passive_id ()
                .unwrap_or_else (|| panic! ("Passive not found for leader {}", leader_id));
        let status_passive_id: ID = self.scene.get_skill (&skill_passive_id).get_status_id ();

        for follower_id in follower_ids {
            let distance: usize = self.grid.find_distance_between (follower_id, &leader_id);

            self.units[*follower_id].try_add_passive (&status_passive_id, distance);
        }
    }

    fn move_unit (&mut self, unit_id: ID, movements: &[Direction]) -> Location {
        let (location, terrain_id): (Location, ID) = self.grid
                .move_unit (unit_id, movements)
                .unwrap_or_else (|| panic! ("Invalid movements {:?}", movements));

        self.apply_terrain (unit_id, terrain_id, location);
        self.try_spawn_recruit (unit_id);

        location
    }

    fn kill_unit (&mut self, unit_id: ID) {
        // TODO: If player leader died, then end game and don't worry about all this
        let faction_id: ID = self.scene.get_unit_builder (&unit_id).get_faction_id ();
        let mut others: Vec<Turn> = Vec::new ();

        println! ("{} died", unit_id);
        self.factions[faction_id].remove_follower (&unit_id);
        self.grid.remove_unit (&unit_id);

        while let Some (t) = self.turns.pop () {
            if t.get_unit_id () == unit_id {
                break
            } else {
                others.push (t);
            }
        }

        self.turns.extend (others);
    }

    fn filter_unit_allegiance (&self, unit_ids: &[ID], faction_id: ID, is_ally: bool) -> Vec<ID> {
        unit_ids.iter ().filter_map (|u: &ID| {
            let faction_id_other: ID = self.units[*u].get_faction_id ();

            if self.factions[faction_id].is_ally (&faction_id_other) == is_ally {
                Some (*u)
            } else {
                None
            }
        }).collect ()
    }

    fn find_units_area (&mut self, unit_id: ID, potential_ids: &[ID], target: Target, search: Search) -> Vec<ID> {
        assert! (!potential_ids.is_empty ());

        let target_ids: Vec<ID> = match target {
            Target::This => vec![potential_ids[0]],
            Target::Ally | Target::Enemy => {
                println! ("Potential targets: {:?}", potential_ids);

                let validator: IDValidator = IDValidator::new (potential_ids);
                let target_id: Option<ID> = self.reader.read_validate (&validator);

                if let Some (target_id) = target_id {
                    vec![target_id]
                } else {
                    Vec::new ()
                }
            }
            Target::Allies | Target::Enemies => match search {
                Search::Single => panic! ("Invalid search {:?} for target {:?}", search, target),
                Search::Radial (r) => {
                    println! ("Potential targets: {:?}", potential_ids);

                    let validator: IDValidator = IDValidator::new (potential_ids);
                    let target_id: Option<ID> = self.reader.read_validate (&validator);

                    if let Some (target_id) = target_id {
                        let location: &Location = self.grid.get_unit_location (&target_id);
                        let target_ids: Vec<ID> = self.grid.find_units (location, Search::Radial (r));
                        let faction_id: ID = self.units[unit_id].get_faction_id ();

                        if let Target::Allies = target {
                            self.filter_unit_allegiance (&target_ids, faction_id, true)
                        } else if let Target::Enemies = target {
                            self.filter_unit_allegiance (&target_ids, faction_id, false)
                        } else {
                            panic! ("Invalid target {:?}", target)
                        }
                    } else {
                        Vec::new ()
                    }
                }
                Search::Path (w, r, _) => loop {
                    println! ("Potential targets: {:?}", potential_ids);

                    let validator: DirectionValidator = DirectionValidator::new ();
                    let direction: Option<Direction> = self.reader.read_validate (&validator);

                    if let Some (direction) = direction {
                        let location: &Location = self.grid.get_unit_location (&unit_id);
                        let target_ids: Vec<ID> = self.grid.find_units (location, Search::Path (w, r, direction));
                        let faction_id: ID = self.units[unit_id].get_faction_id ();

                        if target_ids.is_empty () {
                            println! ("No available targets");
                        } else if let Target::Allies = target {
                            break self.filter_unit_allegiance (&target_ids, faction_id, true)
                        } else if let Target::Enemies = target {
                            break self.filter_unit_allegiance (&target_ids, faction_id, false)
                        } else {
                            panic! ("Invalid target {:?}", target)
                        }
                    } else {
                        break Vec::new ()
                    }
                }
            },
            _ => panic! ("Invalid target {:?}", target),
        };

        if target_ids.is_empty () {
            println! ("Cancelled");

            target_ids
        } else {
            println! ("Targets: {:?}", target_ids);
    
            let validator: ConfirmationValidator = ConfirmationValidator::new ();
            let confirmation: bool = self.reader.read_validate (&validator).expect ("Invalid input");
    
            if confirmation {
                target_ids
            } else {
                Vec::new ()
            }
        }
    }

    fn find_units_range (&mut self, unit_id: ID, target: Target, area: Area, range: u8) -> Vec<ID> {
        if let Target::Map ( .. ) = target {
            panic! ("Invalid target {:?}", target)
        } else if let Target::This = target {
            vec![unit_id]
        } else {
            let location: &Location = self.grid.get_unit_location (&unit_id);
            let neighbour_ids: Vec<ID> = if let Area::Path (w) = area {
                let neighbour_ids_up: Vec<ID> = self.grid.find_units (location, Search::Path (w, range, Direction::Up));
                let neighbour_ids_right: Vec<ID> = self.grid.find_units (location, Search::Path (w, range, Direction::Right));
                let neighbour_ids_left: Vec<ID> = self.grid.find_units (location, Search::Path (w, range, Direction::Left));
                let neighbour_ids_down: Vec<ID> = self.grid.find_units (location, Search::Path (w, range, Direction::Down));
                let mut neighbour_ids: HashSet<ID> = HashSet::new ();

                neighbour_ids.extend (neighbour_ids_up.iter ());
                neighbour_ids.extend (neighbour_ids_right.iter ());
                neighbour_ids.extend (neighbour_ids_left.iter ());
                neighbour_ids.extend (neighbour_ids_down.iter ());

                neighbour_ids.into_iter ().collect ()
            } else {
                self.grid.find_units (location, Search::Radial (range))
            };
            let faction_id: ID = self.units[unit_id].get_faction_id ();

            match target {
                Target::Ally | Target::Allies => self.filter_unit_allegiance (&neighbour_ids, faction_id, true),
                Target::Enemy | Target::Enemies => self.filter_unit_allegiance (&neighbour_ids, faction_id, false),
                _ => panic! ("Invalid target {:?}", target),
            }
        }
    }

    fn find_units (&mut self, unit_id: ID, target: Target, area: Area, range: u8) -> Vec<ID> {
        let potential_ids: Vec<ID> = self.find_units_range (unit_id, target, area, range);
        let search: Search = match area {
            Area::Single => Search::Single,
            Area::Radial (r) => Search::Radial (r),
            Area::Path (w) => Search::Path (w, range, Direction::Length), // Placeholder direction
        };
        let _ = self.sender.send (format! ("Found potential targets: {:?}", potential_ids));
        let target_ids: Vec<ID> = if potential_ids.is_empty () {
            println! ("No available targets");

            Vec::new ()
        } else {
            self.find_units_area (unit_id, &potential_ids, target, search)
        };
        let _ = self.sender.send (format! ("Found targets: {:?}", target_ids));

        target_ids
    }

    fn find_locations_area (&mut self, unit_id: ID, potential_locations: &[Location], search: Search) -> Vec<Location> {
        assert! (!potential_locations.is_empty ());

        let target_locations: Vec<Location> = match search {
            Search::Single => {
                println! ("Potential targets: {:?}", potential_locations);

                let validator: LocationValidator = LocationValidator::new (potential_locations);
                let target_location: Option<Location> = self.reader.read_validate (&validator);

                if let Some (target_location) = target_location {
                    vec![target_location]
                } else {
                    Vec::new ()
                }
            }
            Search::Radial (r) => {
                println! ("Potential targets: {:?}", potential_locations);

                let validator: LocationValidator = LocationValidator::new (potential_locations);
                let target_location: Option<Location> = self.reader.read_validate (&validator);

                if let Some (target_location) = target_location {
                    self.grid.find_locations (&target_location, Search::Radial (r))
                } else {
                    Vec::new ()
                }
            }
            Search::Path (w, r, _) => {
                println! ("Potential targets: {:?}", potential_locations);

                let validator: DirectionValidator = DirectionValidator::new ();
                let direction: Option<Direction> = self.reader.read_validate (&validator);

                if let Some (direction) = direction {
                    let location: &Location = self.grid.get_unit_location (&unit_id);

                    self.grid.find_locations (location, Search::Path (w, r, direction))
                } else {
                    Vec::new ()
                }
            }
        };

        if target_locations.is_empty () {
            println! ("Cancelled");

            target_locations
        } else {
            println! ("Targets: {:?}", target_locations);
    
            let validator: ConfirmationValidator = ConfirmationValidator::new ();
            let confirmation: bool = self.reader.read_validate (&validator).expect ("Invalid input");
    
            if confirmation {
                target_locations
            } else {
                Vec::new ()
            }
        }
    }

    fn find_locations_range (&mut self, unit_id: ID, area: Area, range: u8) -> Vec<Location> {
        let location: &Location = self.grid.get_unit_location (&unit_id);

        if let Area::Path (w) = area {
            let neighbour_locations_up: Vec<Location> = self.grid.find_locations (location, Search::Path (w, range, Direction::Up));
            let neighbour_locations_right: Vec<Location> = self.grid.find_locations (location, Search::Path (w, range, Direction::Right));
            let neighbour_locations_left: Vec<Location> = self.grid.find_locations (location, Search::Path (w, range, Direction::Left));
            let neighbour_locations_down: Vec<Location> = self.grid.find_locations (location, Search::Path (w, range, Direction::Down));
            let mut neighbour_locations: HashSet<Location> = HashSet::new ();

            neighbour_locations.extend (neighbour_locations_up.iter ());
            neighbour_locations.extend (neighbour_locations_right.iter ());
            neighbour_locations.extend (neighbour_locations_left.iter ());
            neighbour_locations.extend (neighbour_locations_down.iter ());

            neighbour_locations.into_iter ().collect ()
        } else {
            self.grid.find_locations (location, Search::Radial (range))
        }
    }

    fn find_locations (&mut self, unit_id: ID, area: Area, range: u8) -> Vec<Location> {
        let potential_locations: Vec<Location> = self.find_locations_range (unit_id, area, range);
        let search: Search = match area {
            Area::Single => Search::Single,
            Area::Radial (r) => Search::Radial (r),
            Area::Path (w) => Search::Path (w, range, Direction::Length), // Placeholder direction
        };
        let _ = self.sender.send (format! ("Found potential targets: {:?}", potential_locations));
        let target_locations: Vec<Location> = self.find_locations_area (unit_id, &potential_locations, search);
        let _ = self.sender.send (format! ("Found targets: {:?}", potential_locations));

        target_locations
    }

    fn start_turn (&mut self, unit_id: ID) {
        let location: &Location = self.grid.get_unit_location (&unit_id);

        if self.grid.is_impassable (location) {
            self.units[unit_id].set_statistic (UnitStatistic::HLT, 0);
        } else {
            self.units[unit_id].start_turn ();
        }
    }

    fn act_attack (&mut self, attacker_id: ID, defender_ids: &[ID]) -> u16 {
        let statistics_attacker: &UnitStatistics = &self.units[attacker_id].get_statistics ();
        let (mov, weapon): (u16, &Weapon) = self.units[attacker_id].act_attack ();
        let weapon: Weapon = *weapon;

        for defender_id in defender_ids {
            let statistics_defender: &UnitStatistics = &self.units[*defender_id].get_statistics ();
            let (damage_mrl, damage_hlt, damage_spl): (u16, u16, u16) = UnitStatistics::calculate_damage (statistics_attacker, statistics_defender, &weapon);
            let appliable_on_attack: Option<Box<dyn Appliable>> = weapon.try_yield_appliable (Rc::clone (&self.scene));
            let appliable_on_hit: Option<Box<dyn Appliable>> = self.units[*defender_id].take_damage (damage_mrl, damage_hlt, damage_spl);

            if let Some (a) = appliable_on_attack {
                self.units[*defender_id].add_appliable (a);
            }

            if let Some (a) = appliable_on_hit {
                self.units[attacker_id].add_appliable (a);
            }

            self.unit_ids_dirty.push (*defender_id);
        }

        mov
    }

    fn act_skill (&mut self, user_id: ID, target_ids: &[ID], skill_id: ID) -> u16 {
        let (mov, status_skill): (u16, Status) = {
            let (mov, skill): (u16, &Skill) = self.units[user_id].act_skill (&skill_id);
            let status_skill_id: ID = skill.get_status_id ();
            let status_skill: Status = *self.scene.get_status (&status_skill_id);

            (mov, status_skill)
        };

        for target_id in target_ids {
            self.units[*target_id].add_status (status_skill);
        }

        mov
    }

    fn act_magic (&mut self, user_id: ID, target: Option<&[Location]>, magic_id: ID) -> u16 {
        let (mov, mut status_magic): (u16, Status) = {
            let (mov, magic): (u16, &Magic) = self.units[user_id].act_magic (&magic_id);
            let status_magic_id: ID = magic.get_status_id ();
            let status_magic: Status = *self.scene.get_status (&status_magic_id);

            (mov, status_magic)
        };

        match target {
            Some (target_locations) => {
                status_magic.set_applier_id (user_id);

                for target_location in target_locations {
                    self.grid.add_status (target_location, status_magic);
                }
            }
            None => {
                self.units[user_id].add_status (status_magic);
            }
        }

        mov
    }

    fn act_wait (&mut self, unit_id: ID) -> u16 {
        self.units[unit_id].act_wait ()
    }

    fn act (&mut self, unit_id: ID) -> (u16, f32) {
        let is_retreat: bool = self.units[unit_id].is_retreat ();
        let is_rout: bool = self.units[unit_id].is_rout ();
        let mut mov: u16 = self.units[unit_id].get_statistic (UnitStatistic::MOV).0;
        let validator: ActionValidator = ActionValidator::new ();

        // print!("\x1B[2J\x1B[1;1H"); // Clears the terminal
        println! ("Start {}'s turn", unit_id);
        let _ = self.sender.send (format! ("Start {}'s turn", unit_id));

        let (mov, delay_multiplier): (u16, f32) = loop {
            print! ("{}", self.grid);
            println! ("Movable locations: {:?}", self.grid.find_unit_movable (&unit_id, mov));
            println! ("Turn order: {:?}\n", self.turns);

            if let Some (action) = self.reader.read_validate (&validator) {
                match action {
                    Action::Attack => {
                        if is_retreat {
                            println! ("Unit cannot attack (is retreating)")
                        } else {
                            let target: Target = self.units[unit_id].get_weapon ().get_target ();
                            let area: Area = self.units[unit_id].get_weapon ().get_area ();
                            let range: u8 = self.units[unit_id].get_weapon ().get_range ();
                            let target_ids: Vec<ID> = self.find_units (unit_id, target, area, range);

                            if !target_ids.is_empty () {
                                let mov: u16 = self.act_attack (unit_id, &target_ids);

                                let _ = self.sender.send (format! ("{}'s action: Attack", unit_id));
                                println! ("Attacking {:?}", target_ids);

                                for target_id in target_ids {
                                    println! ("Defender: {}", self.units[target_id].get_statistics ());
                                }

                                println! ("Self: {}", self.units[unit_id].get_statistics ());

                                break (mov, FACTOR_ATTACK)
                            }
                        }
                    }
                    Action::Weapon => {
                        if is_rout {
                            println! ("Unit cannot rearm (is routed)")
                        } else {
                            self.units[unit_id].switch_weapon ();
                            let _ = self.sender.send (format! ("{}'s action: Switch weapon", unit_id));
                            println! ("New weapon: {:?}", self.units[unit_id].get_weapon ());
                        }
                    }
                    Action::Skill => {
                        if is_rout {
                            println! ("Unit cannot use skill (is routed)")
                        } else {
                            let skill_ids: Vec<ID> = self.units[unit_id].get_skill_ids_actionable ();

                            if skill_ids.is_empty () {
                                println! ("No skills available");
                            } else {
                                let validator: IDValidator = IDValidator::new (&skill_ids);

                                println! ("Available skills: {:?}", skill_ids);
                                let _ = self.sender.send (format! ("{}'s action: Skill", unit_id));

                                if let Some (skill_id) = self.reader.read_validate (&validator) {
                                    let skill: &Skill = self.scene.get_skill (&skill_id);
                                    let target: Target = skill.get_target ();
                                    let area: Area = skill.get_area ();
                                    let range: u8 = skill.get_range ();
                                    let target_ids: Vec<ID> = self.find_units (unit_id, target, area, range);

                                    if !target_ids.is_empty () {
                                        let mov: u16 = self.act_skill (unit_id, &target_ids, skill_id);

                                        println! ("Using skill {} on {:?}", skill_id, target_ids);

                                        for target_id in target_ids {
                                            println! ("Target: {}", self.units[target_id]);
                                        }

                                        break (mov, FACTOR_SKILL)
                                    }
                                }
                            }
                        }
                    }
                    Action::Magic => {
                        if is_rout {
                            println! ("Unit cannot use magic (is routed)")
                        } else {
                            let magic_ids: &[ID] = self.units[unit_id].get_magic_ids ();
                            
                            if magic_ids.is_empty () {
                                println! ("No available magics");
                            } else {
                                let validator: IDValidator = IDValidator::new (magic_ids);

                                println! ("Available magics: {:?}", magic_ids);
                                let _ = self.sender.send (format! ("{}'s action: Magic", unit_id));

                                if let Some (magic_id) = self.reader.read_validate (&validator) {
                                    let magic: &Magic = self.scene.get_magic (&magic_id);
                                    let target: Target = magic.get_target ();
                                    let area: Area = magic.get_area ();
                                    let range: u8 = magic.get_range ();

                                    match target {
                                        Target::This => {
                                            let target_ids: Vec<ID> = self.find_units (unit_id, target, area, range);

                                            if !target_ids.is_empty () {
                                                let mov: u16 = self.act_magic (unit_id, None, magic_id);

                                                println! ("Using magic {} on self", magic_id);

                                                for target_id in target_ids {
                                                    println! ("Target: {}", self.units[target_id]);
                                                }

                                                break (mov, FACTOR_MAGIC)
                                            }
                                        }
                                        Target::Map ( .. ) => {
                                            let target_locations: Vec<Location> =
                                                self.find_locations (unit_id, area, range);

                                            if !target_locations.is_empty () {
                                                let mov: u16 = self.act_magic (unit_id, Some (&target_locations), magic_id);

                                                println! ("Using magic {} on {:?}", magic_id, target_locations);
                                                println! ("Targets: {:?}", target_locations);

                                                break (mov, FACTOR_MAGIC)
                                            }
                                        }
                                        _ => panic! ("Invalid target {:?}", target),
                                    }
                                }
                            }
                        }
                    }
                    Action::Move => {
                        let mut start: Location = *self.grid.get_unit_location (&unit_id);
                        let mut movements: Vec<Direction> = Vec::new ();
                        let validator: MovementValidator = MovementValidator::new ();

                        let _ = self.sender.send (format! ("{}'s action: Move", unit_id));
                        println! ("Current location: {:?}", start);
                        self.grid.set_unit_id_passable (Some (unit_id));

                        while let Some (direction) = self.reader.read_validate (&validator) {
                            if let Direction::Length = direction {
                                // self.grid.set_unit_id_passable (None);
                                println! ("{:?}", movements);
                                self.move_unit (unit_id, &movements);
                                println! ("{:?}, {} MOV remaining", self.grid.get_unit_location (&unit_id), mov);
                                let _ = self.sender.send (format! ("Movements: {:?}", movements));

                                break
                            } else if let Some ((end, cost)) = self.grid.try_move (&start, direction) {
                                if mov >= (cost as u16) {
                                    movements.push (direction);
                                    start = end;
                                    mov -= cost as u16;
                                } else {
                                    println! ("Insufficient MOV");
                                }
                            } else {
                                println! ("Invalid movement");
                            }
                        }
                    }
                    Action::Wait => {
                        let _ = self.sender.send (format! ("{}'s action: Wait", unit_id));

                        break (self.act_wait (unit_id), FACTOR_WAIT)
                    }
                }
            } else {
                break (0, 0.0)
            }
        };

        println! ("End {}'s turn\n", unit_id);
        let _ = self.sender.send (format! ("End {}'s turn", unit_id));
        self.send_passive (unit_id);

        (mov, delay_multiplier)
    }

    fn end_turn (&mut self, unit_id: ID) {
        let city_ids: Vec<ID> = self.grid.find_unit_cities (&unit_id);
        let location: Location = *self.grid.get_unit_location (&unit_id);
        let appliable: Option<Box<dyn Appliable>> = self.grid.try_yield_appliable (&location);

        self.units[unit_id].end_turn (&city_ids, appliable);
        self.grid.decrement_durations (&unit_id);
        self.grid.expand_control (&unit_id);
    }

    fn update_turns (&mut self, mut turn: Turn, delay: u16, mov: u16) {
        self.number_turns += 1;

        if turn.update (delay, mov) {
            self.turns.push (turn);
        } else {
            let reduction: u16 = turn.get_delay ();
            let turns: Vec<Turn> = self.turns.drain ().collect ();

            for mut turn in turns {
                turn.reduce_delay (reduction);
                self.turns.push (turn);
            }

            turn.reduce_delay (reduction);
            turn.update (delay, mov);
            self.turns.push (turn);
        }
    }

    fn do_turn (&mut self) -> bool {
        let turn: Turn = self.turns.pop ().expect ("Turn not found");
        let unit_id: ID = turn.get_unit_id ();

        self.start_turn (unit_id);

        let (mov, delay_multiplier): (u16, f32) = if self.units[unit_id].is_alive () {
            self.act (unit_id)
        } else {
            (u16::MAX, f32::MAX)
        };

        if !self.unit_ids_dirty.is_empty () {
            let unit_ids_attacked: Vec<ID> = self.unit_ids_dirty.drain ( .. ).collect ();

            for unit_id in unit_ids_attacked {
                if !self.units[unit_id].is_alive () {
                    self.kill_unit (unit_id);
                }
            }
        }

        if mov > 0 {
            let delay: u16 = if self.units[unit_id].is_alive () {
                self.end_turn (unit_id);
    
                get_delay (mov, delay_multiplier)
            } else {
                u16::MAX
            };

            if self.units[unit_id].is_alive () {
                self.update_turns (turn, delay, mov);
            } else {
                self.kill_unit (unit_id);
            }

            true
        } else {
            let _ = self.sender.send (String::from ("Quitting game"));

            false
        }
        
    }

    pub fn load_scene (&mut self) {
        todo! ()
    }

    pub fn do_game (&mut self) {
        self.place_unit (0, (0, 0));
        self.place_unit (2, (1, 0));

        while self.do_turn () {

        }
    }
}

#[cfg (test)]
mod tests {
    use super::*;
    use UnitStatistic::{MRL, HLT, SPL, ATK, DEF, MAG, ORG};
    use std::sync::mpsc;

    fn generate_game<R: BufRead> (stream: R) -> Game<R> {
        let scene = Scene::default ();
        let reader = Reader::new (stream);
        let (sender, _) = mpsc::channel ();

        Game::new (scene, reader, sender)
    }

    #[test]
    fn game_place_unit () {
        let mut game = generate_game (&b""[..]);

        game.place_unit (0, (1, 0));
        assert_eq! (game.grid.get_unit_location (&0), &(1, 0));
        // -10% ATK from passive, +20% ATK from toggle, +20% ATK from terrain
        assert_eq! (game.units[0].get_statistic (ATK).0, 26);
    }

    #[test]
    fn game_try_spawn_recruit () {
        let mut game = generate_game (&b""[..]);

        game.place_unit (0, (0, 0));

        game.try_spawn_recruit (0);
        assert! (game.factions[0].get_followers (&0).contains (&1));
        assert_eq! (game.units[1].get_leader_id (), 0);
    }

    #[test]
    fn game_send_passive () {
        let mut game = generate_game (&b""[..]);

        game.place_unit (0, (0, 0));
        game.place_unit (1, (0, 1));
        game.factions[0].add_follower (1, 0);
        game.units[1].set_leader_id (0);
        
        // Test near send
        game.send_passive (1);
        // -10% ATK from passive
        assert_eq! (game.units[1].get_statistic (ATK).0, 18);
        // Test far send
        game.grid.remove_unit (&1);
        game.place_unit (1, (0, 2));
        game.send_passive (1);
        assert_eq! (game.units[1].get_statistic (ATK).0, 20);
    }

    #[test]
    fn game_move_unit () {
        let mut game = generate_game (&b""[..]);

        game.place_unit (0, (0, 0));
        assert_eq! (game.move_unit (0, &[Direction::Right, Direction::Down, Direction::Left]), (1, 0));
        assert_eq! (game.grid.get_unit_location (&0), &(1, 0));
        // -10% ATK from passive, +20% ATK from toggle, +20% ATK from terrain
        assert_eq! (game.units[0].get_statistic (ATK).0, 26);
        assert_eq! (game.grid.get_unit_location (&3), &(0, 0));
    }

    #[test]
    fn game_kill_unit () {
        let mut game = generate_game (&b""[..]);

        game.factions[0].add_follower (1, 0);
        game.turns.push (Turn::new (0, 0, 0));
        game.turns.push (Turn::new (1, 1, 0));
        game.turns.push (Turn::new (2, 2, 0));
        game.turns.push (Turn::new (3, 3, 0));

        game.kill_unit (1);
        assert! (!game.factions[0].get_followers (&0).contains (&1));
        assert_eq! (game.turns.pop ().unwrap ().get_unit_id (), 0);
        assert_eq! (game.turns.pop ().unwrap ().get_unit_id (), 2);
        assert_eq! (game.turns.pop ().unwrap ().get_unit_id (), 3);
        assert! (game.turns.pop ().is_none ());
        // TODO: Surely there will be more later
    }

    #[test]
    fn game_filter_unit_allegiance () {
        let game = generate_game (&b"y"[..]);

        let response = game.filter_unit_allegiance (&[0, 1, 2], 0, true);
        assert_eq! (response.len (), 2);
        assert! (response.contains (&0));
        assert! (response.contains (&1));
        let response = game.filter_unit_allegiance (&[0, 1, 2], 0, false);
        assert_eq! (response.len (), 1);
        assert! (response.contains (&2));
    }

    #[test]
    fn game_find_units_area () {
        let mut game = generate_game (&b"z\n1\nz\n0\nz\n0\nz\n2\nz"[..]);

        game.place_unit (0, (0, 0));
        game.place_unit (1, (1, 1));
        game.place_unit (2, (1, 0));

        let results: Vec<ID> = game.find_units_area (0, &[0], Target::This, Search::Single);
        assert_eq! (results.len (), 1);
        assert! (results.contains (&0));
        let results: Vec<ID> = game.find_units_area (0, &[1], Target::Ally, Search::Path (0, 1, Direction::Length));
        assert_eq! (results.len (), 1);
        assert! (results.contains (&1));
        let results: Vec<ID> = game.find_units_area (0, &[0, 1], Target::Allies, Search::Radial (1));
        assert_eq! (results.len (), 1);
        assert! (results.contains (&0));
        let results: Vec<ID> = game.find_units_area (0, &[0, 1], Target::Allies, Search::Radial (2));
        assert_eq! (results.len (), 2);
        assert! (results.contains (&0));
        assert! (results.contains (&1));
        let results: Vec<ID> = game.find_units_area (0, &[2], Target::Enemy, Search::Radial (1));
        assert_eq! (results.len (), 1);
        assert! (results.contains (&2));
    }

    #[test]
    fn game_find_units_range () {
        let mut game = generate_game (&b""[..]);

        game.place_unit (0, (0, 0));
        game.place_unit (1, (1, 1));
        game.place_unit (2, (1, 0));

        let results: Vec<ID> = game.find_units_range (0, Target::This, Area::Single, 1);
        assert_eq! (results.len (), 1);
        assert! (results.contains (&0));
        let results: Vec<ID> = game.find_units_range (0, Target::Ally, Area::Single, 1);
        assert_eq! (results.len (), 1);
        assert! (results.contains (&0));
        let results: Vec<ID> = game.find_units_range (0, Target::Allies, Area::Radial (1), 0);
        assert_eq! (results.len (), 1);
        assert! (results.contains (&0));
        let results: Vec<ID> = game.find_units_range (0, Target::Allies, Area::Radial (2), 0);
        assert_eq! (results.len (), 1);
        assert! (results.contains (&0));
        let results: Vec<ID> = game.find_units_range (0, Target::Enemy, Area::Radial (0), 1);
        assert_eq! (results.len (), 1);
        assert! (results.contains (&2));
        let results: Vec<ID> = game.find_units_range (2, Target::Enemies, Area::Path (0), 1);
        assert_eq! (results.len (), 2);
        assert! (results.contains (&0));
        assert! (results.contains (&1));
        let results: Vec<ID> = game.find_units_range (0, Target::Enemies, Area::Path (0), 1);
        assert_eq! (results.len (), 1);
        assert! (results.contains (&2));
    }

    #[test]
    fn game_find_units () {
        let mut game = generate_game (&b"z\n0\nz\n0\nz\n0\nz\n2\nz\nd\nz\nd\nx\nz"[..]);

        game.place_unit (0, (0, 0));
        game.place_unit (1, (1, 1));
        game.place_unit (2, (1, 0));

        let results: Vec<ID> = game.find_units (0, Target::This, Area::Single, 1);
        assert_eq! (results.len (), 1);
        assert! (results.contains (&0));
        let results: Vec<ID> = game.find_units (0, Target::Ally, Area::Single, 1);
        assert_eq! (results.len (), 1);
        assert! (results.contains (&0));
        let results: Vec<ID> = game.find_units (0, Target::Allies, Area::Radial (1), 0);
        assert_eq! (results.len (), 1);
        assert! (results.contains (&0));
        let results: Vec<ID> = game.find_units (0, Target::Allies, Area::Radial (2), 0);
        assert_eq! (results.len (), 2);
        assert! (results.contains (&0));
        assert! (results.contains (&1));
        let results: Vec<ID> = game.find_units (0, Target::Enemy, Area::Radial (0), 1);
        assert_eq! (results.len (), 1);
        assert! (results.contains (&2));
        let results: Vec<ID> = game.find_units (2, Target::Enemies, Area::Path (0), 1);
        assert_eq! (results.len (), 1);
        assert! (results.contains (&1));
        let results: Vec<ID> = game.find_units (0, Target::Enemies, Area::Path (0), 1); // Test empty find
        assert! (results.is_empty ());
    }

    #[test]
    fn game_find_locations_area () {
        let mut game = generate_game (&b"0, 0\nz\nd\nz\n0, 0\nz\n1, 0\nz"[..]);

        game.place_unit (0, (0, 0));

        let results: Vec<Location> = game.find_locations_area (0, &[(0, 0)], Search::Single);
        assert_eq! (results.len (), 1);
        assert! (results.contains (&(0, 0)));
        let results: Vec<Location> = game.find_locations_area (0, &[(0, 0), (0, 1)], Search::Path (0, 1, Direction::Length));
        assert_eq! (results.len (), 1);
        assert! (results.contains (&(0, 1)));
        let results: Vec<Location> = game.find_locations_area (0, &[(0, 0), (0, 1), (1, 0)], Search::Radial (1));
        assert_eq! (results.len (), 3);
        assert! (results.contains (&(0, 0)));
        assert! (results.contains (&(0, 1)));
        assert! (results.contains (&(1, 0)));
        let results: Vec<Location> = game.find_locations_area (0, &[(1, 0), (0, 1), (0, 0)], Search::Radial (1));
        assert_eq! (results.len (), 3);
        assert! (results.contains (&(1, 0)));
        assert! (results.contains (&(0, 0)));
        assert! (results.contains (&(1, 1)));
    }

    #[test]
    fn game_find_locations_range () {
        let mut game = generate_game (&b"0, 1\nz\nd\nz\nd\nz\nd\nz\nd\nz\n0, 0\nz\n1, 0\nz\n0, 1\nz"[..]);

        game.place_unit (0, (0, 0));

        let results: Vec<Location> = game.find_locations_range (0, Area::Single, 1);
        assert_eq! (results.len (), 3);
        assert! (results.contains (&(0, 0)));
        assert! (results.contains (&(0, 1)));
        assert! (results.contains (&(1, 0)));
        let results: Vec<Location> = game.find_locations_range (0, Area::Path (0), 1);
        assert_eq! (results.len (), 2);
        assert! (results.contains (&(0, 1)));
        assert! (results.contains (&(1, 0)));
        let results: Vec<Location> = game.find_locations_range (0, Area::Path (0), 2);
        assert_eq! (results.len (), 3);
        assert! (results.contains (&(0, 1)));
        assert! (results.contains (&(0, 2)));
        assert! (results.contains (&(1, 0)));
        let results: Vec<Location> = game.find_locations_range (0, Area::Path (1), 1);
        assert_eq! (results.len (), 3);
        assert! (results.contains (&(0, 1)));
        assert! (results.contains (&(1, 1)));
        assert! (results.contains (&(1, 0)));
        let results: Vec<Location> = game.find_locations_range (0, Area::Path (2), 2);
        assert_eq! (results.len (), 5);
        assert! (results.contains (&(0, 1)));
        assert! (results.contains (&(0, 2)));
        assert! (results.contains (&(1, 1)));
        assert! (results.contains (&(1, 2)));
        assert! (results.contains (&(1, 0)));
        let results: Vec<Location> = game.find_locations_range (0, Area::Radial (1), 1);
        assert_eq! (results.len (), 3);
        assert! (results.contains (&(0, 0)));
        assert! (results.contains (&(0, 1)));
        assert! (results.contains (&(1, 0)));
    }

    #[test]
    fn game_find_locations () {
        let mut game = generate_game (&b"0, 1\nz\nd\nz\nd\nz\nd\nz\nd\nz\n0, 0\nz\n1, 0\nz\n0, 1\nz"[..]);

        game.place_unit (0, (0, 0));

        let results: Vec<Location> = game.find_locations (0, Area::Single, 1);
        assert_eq! (results.len (), 1);
        assert! (results.contains (&(0, 1)));
        let results: Vec<Location> = game.find_locations (0, Area::Path (0), 1);
        assert_eq! (results.len (), 1);
        assert! (results.contains (&(0, 1)));
        let results: Vec<Location> = game.find_locations (0, Area::Path (0), 2);
        assert_eq! (results.len (), 2);
        assert! (results.contains (&(0, 1)));
        assert! (results.contains (&(0, 2)));
        let results: Vec<Location> = game.find_locations (0, Area::Path (1), 1);
        assert_eq! (results.len (), 2);
        assert! (results.contains (&(0, 1)));
        assert! (results.contains (&(1, 1)));
        let results: Vec<Location> = game.find_locations (0, Area::Path (2), 2);
        assert_eq! (results.len (), 4);
        assert! (results.contains (&(0, 1)));
        assert! (results.contains (&(0, 2)));
        assert! (results.contains (&(1, 1)));
        assert! (results.contains (&(1, 2)));
        let results: Vec<Location> = game.find_locations (0, Area::Radial (1), 1);
        assert_eq! (results.len (), 3);
        assert! (results.contains (&(0, 0)));
        assert! (results.contains (&(0, 1)));
        assert! (results.contains (&(1, 0)));
        let results: Vec<Location> = game.find_locations (0, Area::Radial (1), 1);
        assert_eq! (results.len (), 3);
        assert! (results.contains (&(1, 0)));
        assert! (results.contains (&(0, 0)));
        assert! (results.contains (&(1, 1)));
        let results: Vec<Location> = game.find_locations (0, Area::Radial (1), 1);
        assert_eq! (results.len (), 4);
        assert! (results.contains (&(0, 1)));
        assert! (results.contains (&(0, 0)));
        assert! (results.contains (&(1, 1)));
        assert! (results.contains (&(0, 2)));
    }

    #[test]
    fn game_start_turn () {
        let mut game = generate_game (&b""[..]);
        let status_8 = *game.scene.get_status (&8);
        let mut status_11 = *game.scene.get_status (&11);

        game.grid.place_unit (0, (1, 1));
        game.grid.place_unit (1, (0, 0));
        status_11.set_applier_id (0);

        // Test impassable start
        game.grid.add_status (&(1, 2), status_11);
        game.move_unit (0, &[Direction::Right]);
        game.grid.decrement_durations (&0);
        game.grid.decrement_durations (&0);
        game.start_turn (0);
        assert! (!game.units[0].is_alive ());
        // Test normal start
        game.units[1].add_status (status_8);
        game.start_turn (1);
        assert! (game.units[1].is_alive ());
        assert_eq! (game.units[1].get_statistic (DEF).0, 16);
        game.start_turn (1);
        assert! (game.units[1].is_alive ());
        assert_eq! (game.units[1].get_statistic (DEF).0, 14);
    }

    #[test]
    fn game_act_attack () {
        let mut game = generate_game (&b""[..]);
        let status_9 = *game.scene.get_status (&9);
        let status_10 = *game.scene.get_status (&10);

        game.units[0].add_status (status_10);
        game.units[2].add_status (status_9);

        let spl_0_0 = game.units[0].get_statistic (SPL).0;
        let mrl_2_0 = game.units[2].get_statistic (MRL).0;
        let hlt_2_0 = game.units[2].get_statistic (HLT).0;
        let spl_2_0 = game.units[2].get_statistic (SPL).0;
        assert_eq! (game.act_attack (0, &[2]), 10);
        let spl_0_1 = game.units[0].get_statistic (SPL).0;
        let mrl_2_1 = game.units[2].get_statistic (MRL).0;
        let hlt_2_1 = game.units[2].get_statistic (HLT).0;
        let spl_2_1 = game.units[2].get_statistic (SPL).0;
        assert! (spl_0_0 > spl_0_1);
        assert! (mrl_2_0 > mrl_2_1);
        assert! (hlt_2_0 > hlt_2_1);
        assert! (spl_2_0 > spl_2_1);
        assert_eq! (game.units[0].get_statistic (DEF).0, 18);
        assert_eq! (game.units[2].get_statistic (MAG).0, 18);
    }

    #[test]
    fn game_act_skill () {
        let mut game = generate_game (&b""[..]);

        // Test This skill
        let spl_3_0 = game.units[3].get_statistic (SPL).0;
        assert_eq! (game.act_skill (3, &[3], 6), 10);
        let spl_3_1 = game.units[3].get_statistic (SPL).0;
        assert! (spl_3_0 > spl_3_1);
        assert_eq! (game.units[3].get_statistic (DEF).0, 18);
        // Test Ally skill
        assert_eq! (game.act_skill (3, &[0], 4), 10);
        let spl_3_2 = game.units[3].get_statistic (SPL).0;
        assert! (spl_3_1 > spl_3_2);
        assert_eq! (game.units[0].get_statistic (DEF).0, 18);
        // Test Allies skill
        assert_eq! (game.act_skill (3, &[0, 1], 5), 10);
        let spl_3_3 = game.units[3].get_statistic (SPL).0;
        assert! (spl_3_2 > spl_3_3);
        assert_eq! (game.units[0].get_statistic (DEF).0, 16);
        assert_eq! (game.units[1].get_statistic (DEF).0, 18);
    }

    #[test]
    fn game_act_magic () {
        let mut game = generate_game (&b""[..]);

        // Test This magic
        let hlt_0_0 = game.units[0].get_statistic (HLT).0;
        let spl_0_0 = game.units[0].get_statistic (SPL).0;
        let org_0_0 = game.units[0].get_statistic (ORG).0;
        assert_eq! (game.act_magic (0, None, 0), 10);
        let hlt_0_1 = game.units[0].get_statistic (HLT).0;
        let spl_0_1 = game.units[0].get_statistic (SPL).0;
        let org_0_1 = game.units[0].get_statistic (ORG).0;
        assert! (hlt_0_0 > hlt_0_1);
        assert! (spl_0_0 > spl_0_1);
        assert! (org_0_0 > org_0_1);
        assert_eq! (game.units[0].get_statistic (DEF).0, 18);
        // Test Map magic
        assert_eq! (game.act_magic (0, Some (&[(1, 0)]), 3), 10);
        let hlt_0_2 = game.units[0].get_statistic (HLT).0;
        let spl_0_2 = game.units[0].get_statistic (SPL).0;
        let org_0_2 = game.units[0].get_statistic (ORG).0;
        assert! (hlt_0_1 > hlt_0_2);
        assert! (spl_0_1 > spl_0_2);
        assert! (org_0_1 > org_0_2);
        assert! (game.grid.try_yield_appliable (&(1, 0)).is_some ())
    }

    #[test]
    fn game_act_wait () {
        let mut game = generate_game (&b""[..]);

        let spl_0_0 = game.units[0].get_statistic (SPL).0;
        assert_eq! (game.act_wait (0), 10);
        let spl_0_1 = game.units[0].get_statistic (SPL).0;
        assert! (spl_0_0 > spl_0_1);
    }

    #[test]
    fn game_act () {
        let input = b"z\nd\ns\na\nz\nz\nw\na\nz\na\n2\nz\n\
                z\na\nx\nq\nq\nq\ns\n0\nz\n\
                z\na\nz\nd\n0\nz";
        let mut game = generate_game (&input[..]);

        game.place_unit (0, (0, 0));
        game.place_unit (2, (1, 0));

        // TODO: This test is extremely cursory
        let spl_0_0 = game.units[0].get_statistic (SPL).0;
        let mrl_2_0 = game.units[2].get_statistic (MRL).0;
        let hlt_2_0 = game.units[2].get_statistic (HLT).0;
        let spl_2_0 = game.units[2].get_statistic (SPL).0;
        game.act (0);
        let spl_0_1 = game.units[0].get_statistic (SPL).0;
        let mrl_2_1 = game.units[2].get_statistic (MRL).0;
        let hlt_2_1 = game.units[2].get_statistic (HLT).0;
        let spl_2_1 = game.units[2].get_statistic (SPL).0;
        assert_eq! (game.grid.get_unit_location (&0), &(0, 0));
        assert! (spl_0_0 > spl_0_1);
        assert! (mrl_2_0 > mrl_2_1);
        assert! (hlt_2_0 > hlt_2_1);
        assert! (spl_2_0 > spl_2_1);
        // Test skills and switch weapon
        game.act (2);
        assert_eq! (game.units[2].get_weapon ().get_id (), 2);
        assert! (!game.units[2].get_skill_ids_actionable ().contains (&0));
        // Test magic
        game.act (1);
    }

    #[test]
    fn game_end_turn () {
        let mut game = generate_game (&b""[..]);

        game.place_unit (0, (0, 0));
        game.place_unit (1, (1, 1));
        game.units[0].set_statistic (MRL, 500);
        game.units[0].set_statistic (HLT, 500);
        game.units[0].set_statistic (SPL, 500);
        game.units[1].set_statistic (MRL, 500);
        game.units[1].set_statistic (HLT, 500);
        game.units[1].set_statistic (SPL, 500);

        // Test encircled end
        let controlled_0_0 = game.grid.get_faction_locations (&0).unwrap ().len ();
        let mrl_1_0 = game.units[1].get_statistic (MRL).0;
        let hlt_1_0 = game.units[1].get_statistic (HLT).0;
        let spl_1_0 = game.units[1].get_statistic (SPL).0;
        game.end_turn (1);
        let controlled_0_1 = game.grid.get_faction_locations (&0).unwrap ().len ();
        let mrl_1_1 = game.units[1].get_statistic (MRL).0;
        let hlt_1_1 = game.units[1].get_statistic (HLT).0;
        let spl_1_1 = game.units[1].get_statistic (SPL).0;
        assert_eq! (controlled_0_0, controlled_0_1);
        assert! (mrl_1_0 < mrl_1_1);
        assert_eq! (hlt_1_0, hlt_1_1);
        assert_eq! (spl_1_0, spl_1_1);
        // Test normal end
        let mrl_0_0 = game.units[0].get_statistic (MRL).0;
        let hlt_0_0 = game.units[0].get_statistic (HLT).0;
        let spl_0_0 = game.units[0].get_statistic (SPL).0;
        game.end_turn (0);
        let controlled_0_2 = game.grid.get_faction_locations (&0).unwrap ().len ();
        let mrl_0_1 = game.units[0].get_statistic (MRL).0;
        let hlt_0_1 = game.units[0].get_statistic (HLT).0;
        let spl_0_1 = game.units[0].get_statistic (SPL).0;
        assert_eq! (controlled_0_1 + 1, controlled_0_2);
        assert! (mrl_0_0 < mrl_0_1);
        assert! (hlt_0_0 < hlt_0_1);
        assert! (spl_0_0 < spl_0_1);
    }

    #[test]
    fn game_update_turns () {
        let mut game = generate_game (&b""[..]);

        // Test normal update
        game.turns.push (Turn::new (0, 0, 0));
        game.turns.push (Turn::new (1, 1, 0));
        game.turns.push (Turn::new (2, 2, 0));
        game.turns.push (Turn::new (3, 3, 0));
        let turn: Turn = game.turns.pop ().unwrap ();
        assert_eq! (turn.get_unit_id (), 0);
        game.update_turns (turn, 1, 1); // Test MOV update (0)
        let turn: Turn = game.turns.pop ().unwrap ();
        assert_eq! (turn.get_unit_id (), 0); // (0)
        game.update_turns (turn, 2, 0); // Test ID update (1)
        let turn: Turn = game.turns.pop ().unwrap ();
        assert_eq! (turn.get_unit_id (), 1);
        game.update_turns (turn, 2, 0); // Test ID update (2)
        let turn: Turn = game.turns.pop ().unwrap ();
        assert_eq! (turn.get_unit_id (), 2);
        game.update_turns (turn, 10, 0);
        assert_eq! (game.turns.pop ().unwrap ().get_unit_id (), 0); // (1)
        assert_eq! (game.turns.pop ().unwrap ().get_unit_id (), 1); // (2)
        let turn: Turn = game.turns.pop ().unwrap ();
        assert_eq! (turn.get_unit_id (), 3);
        game.update_turns (turn, 8, 0); // Test delay update (3)
        assert_eq! (game.turns.pop ().unwrap ().get_unit_id (), 3);
        assert_eq! (game.turns.pop ().unwrap ().get_unit_id (), 2);
        // Test reduce update
        game.turns.push (Turn::new (0, 65530, 0));
        game.turns.push (Turn::new (1, 65531, 0));
        game.turns.push (Turn::new (2, 65532, 0));
        game.turns.push (Turn::new (3, 65533, 0));
        let turn: Turn = game.turns.pop ().unwrap ();
        assert_eq! (turn.get_unit_id (), 0);
        game.update_turns (turn, 5, 0);
        let turn: Turn = game.turns.pop ().unwrap ();
        assert_eq! (turn.get_unit_id (), 1);
        game.update_turns (turn, 5, 0);
        let turn: Turn = game.turns.pop ().unwrap ();
        assert_eq! (turn.get_unit_id (), 2);
        assert_eq! (turn.get_delay (), 1);
        let turn: Turn = game.turns.pop ().unwrap ();
        assert_eq! (turn.get_unit_id (), 3);
        assert_eq! (turn.get_delay (), 2);
        let turn: Turn = game.turns.pop ().unwrap ();
        assert_eq! (turn.get_unit_id (), 0);
        assert_eq! (turn.get_delay (), 4);
        let turn: Turn = game.turns.pop ().unwrap ();
        assert_eq! (turn.get_unit_id (), 1);
        assert_eq! (turn.get_delay (), 5);
    }

    // #[test]
    // fn game_do_turn () {
    // let mut game = generate_game (&b""[..]);

    // todo! ("no functionality")
    // }
}
