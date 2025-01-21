use super::{ActionValidator, ConfirmationValidator, DirectionValidator, IDValidator, LocationValidator, MovementValidator, Reader, Turn};
use crate::character::{Faction, FactionBuilder, Magic, Skill, Tool, Unit, UnitBuilder, UnitStatistic, UnitStatistics, Weapon};
use crate::common::{FACTOR_ATTACK, FACTOR_MAGIC, FACTOR_SKILL, FACTOR_WAIT, ID, Scene, Target};
use crate::dynamic::{Appliable, AppliableKind, Applier, Dynamic};
// use crate::event::Handler;
use crate::map::{Area, Direction, Grid, Location, Search};
use sdl2::keyboard::Keycode;
// use std::cell::RefCell;
use std::collections::{BinaryHeap, HashSet};
use std::error::Error;
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

fn get_delay (mov: u16, action: Action) -> u16 {
    let delay: f32 = DELAYS[mov as usize] as f32;
    let factor: f32 = match action {
        Action::Attack => FACTOR_ATTACK,
        Action::Skill => FACTOR_SKILL,
        Action::Magic => FACTOR_MAGIC,
        Action::Wait => FACTOR_WAIT,
        _ => panic! ("Invalid action {:?}", action),
    };

    (delay * factor) as u16
}

#[derive (Debug)]
pub enum Action {
    Attack,
    Weapon,
    Skill,
    Magic,
    Move,
    Wait,
}

#[derive (Debug)]
pub enum State {
    Idle,
    Move,
    AttackTarget,
    AttackConfirm,
    SkillChoose,
    SkillTarget,
    SkillConfirm,
    MagicChoose,
    MagicTarget,
    MagicConfirm,
}

#[derive (Debug)]
pub struct Game<R: BufRead> {
    scene: Rc<Scene>,
    state: State,
    turn: Option<Turn>,
    turns: BinaryHeap<Turn>,
    number_turns: usize,
    // handler: Rc<RefCell<Handler>>,
    grid: Grid,
    units: Vec<Unit>,
    factions: Vec<Faction>,
    // Action context
    // Move context
    unit_location: Location,
    movements: Vec<Direction>,
    mov: u16,
    // Attack, skill, magic context
    target: Target,
    area: Area,
    range: u8,
    target_idx: usize,
    target_location: Location, // Magic context
    potential_ids: Vec<ID>,
    potential_locations: Vec<Location>,
    target_ids: Vec<ID>,
    target_locations: Vec<Location>,
    skill_magic_idx: usize, // Skill/magic context
    skill_magic_ids: Vec<ID>, // Skill/magic context
    skill_magic_id: ID, // Skill/magic context
    unit_ids_dirty: Vec<ID>, // Attack context

    reader: Reader<R>,
    sender: Sender<String>,
}

impl<R: BufRead> Game<R> {
    pub fn new (scene: Scene, reader: Reader<R>, sender: Sender<String>) -> Self {
        let scene: Rc<Scene> = Rc::new (scene);
        let state: State = State::Idle;
        let turn: Option<Turn> = None;
        let turns: BinaryHeap<Turn> = BinaryHeap::new ();
        let number_turns: usize = 0;
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
        let unit_location: Location = (usize::MAX, usize::MAX);
        let movements: Vec<Direction> = Vec::new ();
        let mov: u16 = u16::MAX;
        let target: Target = Target::This;
        let area: Area = Area::Single;
        let range: u8 = u8::MAX;
        let target_idx: usize = 0;
        let target_location: Location = (0, 0);
        let potential_ids: Vec<ID> = Vec::new ();
        let potential_locations: Vec<Location> = Vec::new ();
        let target_ids: Vec<ID> = Vec::new ();
        let target_locations: Vec<Location> = Vec::new ();
        let skill_magic_idx: usize = 0;
        let skill_magic_ids: Vec<ID> = Vec::new ();
        let skill_magic_id: ID = ID::MAX;
        let unit_ids_dirty: Vec<ID> = Vec::new ();
        let reader: Reader<R> = reader;

        let _ = sender.send (String::from ("Game creation complete"));

        Self { scene, state, turn, turns, number_turns, /* handler, */ grid, units, factions, unit_location, movements, mov, target, area, range, target_idx, target_location, potential_ids, potential_locations, target_ids, target_locations, skill_magic_idx, skill_magic_ids, skill_magic_id, unit_ids_dirty, reader, sender }
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
        let delay: u16 = get_delay (mov, Action::Wait);
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
            self.factions[faction_id].add_follower (recruit_id, leader_id);
            self.units[recruit_id].set_leader_id (unit_id);
            self.apply_terrain (recruit_id, terrain_id, location);
            // self.units[r].apply_inactive_skills ();
            self.add_turn (recruit_id);
        }
    }

    fn send_passive (&mut self, unit_id: ID) {
        let leader_id: ID = self.units[unit_id].get_leader_id ();
        let faction_id: ID = self.scene.get_unit_builder (&unit_id).get_faction_id ();
        let follower_ids: &HashSet<ID> = self.factions[faction_id].get_followers (&leader_id);
        let skill_passive_id: ID = self.units[leader_id].get_skill_passive_id ()
                .unwrap_or_else (|| panic! ("Passive not found for leader {}", leader_id));

        for follower_id in follower_ids {
            let distance: usize = self.grid.find_distance_between (follower_id, &leader_id);

            self.units[*follower_id].try_add_passive (&skill_passive_id, distance);
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

    fn move_unit_new (&mut self, unit_id: ID) -> Location {
        let (location, terrain_id): (Location, ID) = self.grid
                .move_unit (unit_id, &self.movements)
                .unwrap_or_else (|| panic! ("Invalid movements {:?}", self.movements));

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

    fn find_units_from (&mut self, locations: &[Location]) -> Vec<ID> {
        locations.iter ().filter_map (|l: &Location|
            self.grid.get_location_unit (l).copied ()
        ).collect::<Vec<ID>> ()
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

    fn find_units_area_new (&mut self, unit_id: ID, target: Target, search: Search) -> Vec<ID> {
        assert! (!self.potential_ids.is_empty ());

        match target {
            Target::This => vec![self.potential_ids[0]],
            Target::Ally | Target::Enemy => {
                let target_id: ID = self.potential_ids[self.target_idx];

                vec![target_id]
            }
            Target::Allies | Target::Enemies => match search {
                Search::Single => panic! ("Invalid search {:?} for target {:?}", search, target),
                Search::Radial (r) => {
                    let target_id: ID = self.potential_ids[self.target_idx];
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
                }
                Search::Path (w, r, d) => {
                    let location: &Location = self.grid.get_unit_location (&unit_id);
                    let target_ids: Vec<ID> = self.grid.find_units (location, Search::Path (w, r, d));
                    let faction_id: ID = self.units[unit_id].get_faction_id ();

                    if let Target::Allies = target {
                        self.filter_unit_allegiance (&target_ids, faction_id, true)
                    } else if let Target::Enemies = target {
                        self.filter_unit_allegiance (&target_ids, faction_id, false)
                    } else {
                        panic! ("Invalid target {:?}", target)
                    }
                }
            },
            _ => panic! ("Invalid target {:?}", target),
        }
    }

    fn find_units_range (&self, unit_id: ID, target: Target, area: Area, range: u8) -> Vec<ID> {
        if let Target::Map = target {
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

    fn find_units_range_new (&self, unit_id: ID) -> Vec<ID> {
        if let Target::Map = self.target {
            panic! ("Invalid target {:?}", self.target)
        } else if let Target::This = self.target {
            vec![unit_id]
        } else {
            let location: &Location = self.grid.get_unit_location (&unit_id);
            let neighbour_ids: Vec<ID> = if let Area::Path (w) = self.area {
                let neighbour_ids_up: Vec<ID> = self.grid.find_units (location, Search::Path (w, self.range, Direction::Up));
                let neighbour_ids_right: Vec<ID> = self.grid.find_units (location, Search::Path (w, self.range, Direction::Right));
                let neighbour_ids_left: Vec<ID> = self.grid.find_units (location, Search::Path (w, self.range, Direction::Left));
                let neighbour_ids_down: Vec<ID> = self.grid.find_units (location, Search::Path (w, self.range, Direction::Down));
                let mut neighbour_ids: HashSet<ID> = HashSet::new ();

                neighbour_ids.extend (neighbour_ids_up.iter ());
                neighbour_ids.extend (neighbour_ids_right.iter ());
                neighbour_ids.extend (neighbour_ids_left.iter ());
                neighbour_ids.extend (neighbour_ids_down.iter ());

                neighbour_ids.into_iter ().collect ()
            } else {
                self.grid.find_units (location, Search::Radial (self.range))
            };
            let faction_id: ID = self.units[unit_id].get_faction_id ();

            match self.target {
                Target::Ally | Target::Allies => self.filter_unit_allegiance (&neighbour_ids, faction_id, true),
                Target::Enemy | Target::Enemies => self.filter_unit_allegiance (&neighbour_ids, faction_id, false),
                _ => panic! ("Invalid target {:?}", self.target),
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

    fn find_locations_area_new (&mut self, search: Search) -> Vec<Location> {
        assert! (!self.potential_locations.is_empty ());

        match search {
            Search::Single => vec![self.target_location],
            Search::Radial ( .. ) | Search::Path ( .. ) => self.grid.find_locations (&self.target_location, search),
        }
    }

    fn find_locations_range (&self, unit_id: ID, area: Area, range: u8) -> Vec<Location> {
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

    fn find_locations_range_new (&self, unit_id: ID) -> Vec<Location> {
        let location: &Location = self.grid.get_unit_location (&unit_id);

        if let Area::Path (w) = self.area {
            let neighbour_locations_up: Vec<Location> = self.grid.find_locations (location, Search::Path (w, self.range, Direction::Up));
            let neighbour_locations_right: Vec<Location> = self.grid.find_locations (location, Search::Path (w, self.range, Direction::Right));
            let neighbour_locations_left: Vec<Location> = self.grid.find_locations (location, Search::Path (w, self.range, Direction::Left));
            let neighbour_locations_down: Vec<Location> = self.grid.find_locations (location, Search::Path (w, self.range, Direction::Down));
            let mut neighbour_locations: HashSet<Location> = HashSet::new ();

            neighbour_locations.extend (neighbour_locations_up.iter ());
            neighbour_locations.extend (neighbour_locations_right.iter ());
            neighbour_locations.extend (neighbour_locations_left.iter ());
            neighbour_locations.extend (neighbour_locations_down.iter ());

            neighbour_locations.into_iter ().collect ()
        } else {
            self.grid.find_locations (location, Search::Radial (self.range))
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

    fn act_attack_new (&mut self, attacker_id: ID) {
        let statistics_attacker: &UnitStatistics = &self.units[attacker_id].get_statistics ();
        let (_, weapon): (u16, &Weapon) = self.units[attacker_id].act_attack ();
        let weapon: Weapon = *weapon;

        for defender_id in &self.target_ids {
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
    }

    fn act_skill (&mut self, user_id: ID, target_ids: &[ID], skill_id: ID) -> u16 {
        let (mov, appliable_skill): (u16, AppliableKind) = {
            let (mov, skill): (u16, &Skill) = self.units[user_id].act_skill (&skill_id);
            let appliable_skill: AppliableKind = skill.get_appliable ();

            (mov, appliable_skill)
        };

        for target_id in target_ids {
            let mut appliable_skill: Box<dyn Appliable> = appliable_skill.appliable (Rc::clone (&self.scene));

            appliable_skill.set_applier_id (user_id);
            self.units[*target_id].add_appliable (appliable_skill);
        }

        mov
    }

    fn act_skill_new (&mut self, user_id: ID) {
        let appliable_skill: AppliableKind = {
            let (_, skill): (_, &Skill) = self.units[user_id].act_skill (&self.skill_magic_id);

            skill.get_appliable ()
        };

        for target_id in &self.target_ids {
            let mut appliable_skill: Box<dyn Appliable> = appliable_skill.appliable (Rc::clone (&self.scene));

            appliable_skill.set_applier_id (user_id);
            self.units[*target_id].add_appliable (appliable_skill);
        }
    }

    fn act_magic (&mut self, user_id: ID, target: Option<&[Location]>, magic_id: ID) -> u16 {
        let (mov, appliable_magic): (u16, AppliableKind) = {
            let (mov, magic): (u16, &Magic) = self.units[user_id].act_magic (&magic_id);
            let appliable_magic: AppliableKind = magic.get_appliable ();

            (mov, appliable_magic)
        };

        match target {
            Some (target_locations) => {

                for target_location in target_locations {
                    let mut appliable_magic: Box<dyn Appliable> = appliable_magic.appliable (Rc::clone (&self.scene));

                    appliable_magic.set_applier_id (user_id);
                    self.grid.add_appliable (target_location, appliable_magic);

                    if let Some (unit_id) = self.grid.get_location_unit (target_location) {
                        let appliable_on_occupy: Option<Box<dyn Appliable>> = self.grid.try_yield_appliable (target_location);

                        if let Some (appliable_on_occupy) = appliable_on_occupy {
                            self.units[*unit_id].add_appliable (appliable_on_occupy);
                        }
                    }
                }
            }
            None => {
                let mut appliable_magic: Box<dyn Appliable> = appliable_magic.appliable (Rc::clone (&self.scene));

                appliable_magic.set_applier_id (user_id);
                self.units[user_id].add_appliable (appliable_magic);
            }
        }

        mov
    }

    fn act_magic_new (&mut self, user_id: ID) {
        let appliable_magic: AppliableKind = {
            let (_, magic): (_, &Magic) = self.units[user_id].act_magic (&self.skill_magic_id);
            let appliable_magic: AppliableKind = magic.get_appliable ();

            appliable_magic
        };

        match self.target {
            Target::This => {
                let mut appliable_magic: Box<dyn Appliable> = appliable_magic.appliable (Rc::clone (&self.scene));

                appliable_magic.set_applier_id (user_id);
                self.units[user_id].add_appliable (appliable_magic);
            }
            Target::Map => for target_location in &self.target_locations {
                let mut appliable_magic: Box<dyn Appliable> = appliable_magic.appliable (Rc::clone (&self.scene));

                appliable_magic.set_applier_id (user_id);
                self.grid.add_appliable (target_location, appliable_magic);

                if let Some (unit_id) = self.grid.get_location_unit (target_location) {
                    let appliable_on_occupy: Option<Box<dyn Appliable>> = self.grid.try_yield_appliable (target_location);

                    if let Some (appliable_on_occupy) = appliable_on_occupy {
                        self.units[*unit_id].add_appliable (appliable_on_occupy);
                    }
                }
            }
            _ => panic! ("Invalid target {:?}", self.target),
        }
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
                                        Target::Map => {
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

    pub fn init (&mut self) -> Result<(), Box<dyn Error>> {
        let mut unit_locations: Vec<(ID, Location)> = Vec::new ();

        for (unit_id, location) in self.scene.unit_locations_iter ().enumerate () {
            if let Some (location) = location {
                unit_locations.push ((unit_id, *location));
            }
        }

        for (unit_id, location) in unit_locations.iter () {
            self.place_unit (*unit_id, *location);
        }

        for (unit_id, _) in unit_locations {
            self.send_passive (unit_id);
        }

        let _ = self.sender.send (String::from ("Game initialisation complete"));

        Ok (())
    }

    pub fn load_scene (&mut self) {
        todo! ()
    }

    pub fn do_turn (&mut self) -> bool {
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

                // get_delay (mov, delay_multiplier)
                get_delay (mov, Action::Wait) // TODO: Temporary to make this compile
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

    pub fn display_turn (&self) {
        let turn: &Turn = self.turn.as_ref ().unwrap_or_else (|| self.turns.peek ().unwrap ());
        let unit_id: ID = turn.get_unit_id ();

        println! ("{}'s turn", unit_id);
        print! ("{}", self.grid);
        println! ("Turn order: {:?}\n", self.turns);
    }

    pub fn display_prompt (&self) {
        match self.state {
            State::Idle => println! ("Actions: Move (q), switch weapon (w), attack (a), skill (s), magic (d), wait (z)"),
            State::Move => println! ("Actions: Up (w), left (a), down (s), right (d), confirm (z), cancel (x)"),
            State::AttackTarget => println! ("Actions: Up (w), left/previous (a), down (s), right/next (d), confirm (z), cancel (x)"),
            State::AttackConfirm => println! ("Actions: Confirm (z), cancel (x)"),
            State::SkillChoose => println! ("Actions: Previous (a), next (d), confirm (z), cancel (x)"),
            State::SkillTarget => println! ("Actions: Up (w), left/previous (a), down (s), right/next (d), confirm (z), cancel (x)"),
            State::SkillConfirm => println! ("Actions: Confirm (z), cancel (x)"),
            State::MagicChoose => println! ("Actions: Previous (a), next (d), confirm (z), cancel (x)"),
            State::MagicTarget => println! ("Actions: Up (w), left (a), down (s), right (d), confirm (z), cancel (x)"),
            State::MagicConfirm => println! ("Actions: Confirm (z), cancel (x)"),
        }
    }

    // TODO: update () should obviously not be processing Keycode
    pub fn update (&mut self, input: Keycode) -> bool {
        let unit_id: ID = if let Some (turn) = &self.turn {
            // println! ("Delay: {}", turn.get_delay ());

            turn.get_unit_id ()
        } else {
            let turn: Turn = self.turns.pop ().expect ("Turn not found");
            let unit_id: ID = turn.get_unit_id ();

            // println! ("Delay: {}", turn.get_delay ());
            self.turn = Some (turn);
            self.mov = self.units[unit_id].get_statistic (UnitStatistic::MOV).0;

            unit_id
        };
        let is_retreat: bool = self.units[unit_id].is_retreat ();
        let is_rout: bool = self.units[unit_id].is_rout ();

        // print!("\x1B[2J\x1B[1;1H"); // Clears the terminal
        // println! ("Start {}'s turn", unit_id);
        // let _ = self.sender.send (format! ("Start {}'s turn", unit_id));
        // print! ("{}", self.grid);
        // println! ("Movable locations: {:?}", self.grid.find_unit_movable (&unit_id, self.mov));
        // println! ("Turn order: {:?}\n", self.turns);
        // println! ("Actions: Move (q), switch weapon (w), attack (a), skill (s), magic (d), wait (z)");

        let action: Option<Action> = match self.state {
            State::Idle => match input {
                // Move
                Keycode::Q => {
                    println! ("{}'s action: Move", unit_id);
                    println! ("Movable locations: {:?}", self.grid.find_unit_movable (&unit_id, self.mov));
                    let _ = self.sender.send (format! ("{}'s action: Move", unit_id));

                    self.state = State::Move;
                    self.unit_location = *self.grid.get_unit_location (&unit_id);
                    self.grid.set_unit_id_passable (Some (unit_id));
                    self.movements.clear ();
                    println! ("Current location: {:?}", self.unit_location);

                    None
                }
                // Switch weapon
                Keycode::W => {
                    println! ("{}'s action: Switch weapon", unit_id);
                    let _ = self.sender.send (format! ("{}'s action: Switch weapon", unit_id));

                    if is_rout {
                        println! ("Unit cannot rearm (is routed)")
                    } else {
                        self.units[unit_id].switch_weapon ();
                        let _ = self.sender.send (format! ("{}'s action: Switch weapon", unit_id));
                        println! ("New weapon: {:?}", self.units[unit_id].get_weapon ());
                    }

                    None
                }
                // Attack
                Keycode::A => {
                    println! ("{}'s action: Attack", unit_id);
                    let _ = self.sender.send (format! ("{}'s action: Attack", unit_id));

                    if is_retreat {
                        println! ("Unit cannot attack (is retreating)")
                    } else {
                        let weapon: &Weapon = self.units[unit_id].get_weapon ();

                        self.target = weapon.get_target ();
                        self.area = weapon.get_area ();
                        self.range = weapon.get_range ();
                        self.potential_ids = self.find_units_range_new (unit_id);
                        let _ = self.sender.send (format! ("Potential targets: {:?}", self.potential_ids));

                        if self.potential_ids.is_empty () {
                            println! ("No available targets");
                        } else {
                            self.state = State::AttackTarget;
                            self.target_idx = 0;
                            println! ("Potential targets: {:?}", self.potential_ids);
                            println! ("Equipped weapon: {:?}", weapon);
                        };
                    }

                    None
                }
                // Skill
                Keycode::S => {
                    println! ("{}'s action: Skill", unit_id);
                    let _ = self.sender.send (format! ("{}'s action: Skill", unit_id));

                    if is_rout {
                        println! ("Unit cannot use skill (is routed)")
                    } else {
                        let skill_ids: Vec<ID> = self.units[unit_id].get_skill_ids_actionable ();

                        if skill_ids.is_empty () {
                            println! ("No available skills");
                        } else {
                            self.state = State::SkillChoose;
                            self.skill_magic_idx = 0;
                            self.skill_magic_ids = skill_ids;
                            self.skill_magic_id = 0;
                            self.target_idx = 0;
                            println! ("Skills: {:?}", self.skill_magic_ids);
                        }
                    }

                    None
                }
                // Magic
                Keycode::D => {
                    let _ = self.sender.send (format! ("{}'s action: Magic", unit_id));

                    if is_rout {
                        println! ("Unit cannot use magic (is routed)")
                    } else {
                        let magic_ids: &[ID] = self.units[unit_id].get_magic_ids ();

                        if magic_ids.is_empty () {
                            println! ("No available magics");
                        } else {
                            self.state = State::MagicChoose;
                            self.skill_magic_ids = magic_ids.to_vec ();
                            self.skill_magic_id = 0;
                            self.target_idx = 0;
                            println! ("Magics: {:?}", self.skill_magic_ids);
                        }
                    }

                    None
                }
                // Wait
                Keycode::Z => {
                    println! ("{}'s action: Wait", unit_id);
                    let _ = self.sender.send (format! ("{}'s action: Wait", unit_id));
                    self.act_wait (unit_id);

                    Some (Action::Wait)
                }
                _ => {
                    println! ("Invalid input");

                    None
                }
            }
            State::Move => {
                match input {
                    // Move
                    Keycode::W | Keycode::A | Keycode::S | Keycode::D => {
                        let direction: Direction = match input {
                            Keycode::W => Direction::Up,
                            Keycode::A => Direction::Left,
                            Keycode::S => Direction::Down,
                            Keycode::D => Direction::Right,
                            _ => unreachable! (),
                        };

                        if let Some ((end, cost)) = self.grid.try_move (&self.unit_location, direction) {
                            println! ("{:?}", direction);
                            if self.mov >= (cost as u16) {
                                self.movements.push (direction);
                                self.unit_location = end;
                                self.mov -= cost as u16;
                            } else {
                                println! ("Insufficient MOV");
                            }
                        } else {
                            println! ("Invalid direction {:?}", direction);
                        }
                    }
                    // Confirm
                    Keycode::Z => {
                        // self.grid.set_unit_id_passable (None);
                        self.move_unit_new (unit_id);
                        self.state = State::Idle;
                        println! ("{:?}", self.movements);
                        println! ("{:?}, {} MOV remaining", self.grid.get_unit_location (&unit_id), self.mov);
                        print! ("{}", self.grid);
                        let _ = self.sender.send (format! ("Movements: {:?}", self.movements));
                    }
                    // Cancel
                    Keycode::X => self.state = State::Idle,
                    _ => println! ("Invalid input"),
                }

                None
            }
            State::AttackTarget => {
                let mut is_confirm: bool = false;
                let mut is_cancel: bool = false;
                let search: Option<Search> = match self.area {
                    Area::Single => {
                        let target_idx: Option<usize> = match input {
                            // Previous
                            Keycode::A => Some (
                                self.target_idx.checked_sub (1)
                                        .unwrap_or_else (|| self.potential_ids.len ().saturating_sub (1))
                            ),
                            // Next
                            Keycode::D => Some ((self.target_idx + 1) % self.potential_ids.len ()),
                            // Confirm
                            Keycode::Z => {
                                is_confirm = true;

                                Some (self.target_idx)
                            }
                            // Cancel
                            Keycode::X => {
                                is_cancel = true;

                                None
                            }
                            _ => {
                                println! ("Invalid input");

                                None
                            }
                        };

                        if let Some (target_idx) = target_idx {
                            self.target_idx = target_idx;

                            Some (Search::Single)
                        } else {
                            None
                        }
                    }
                    Area::Radial (r) => {
                        let target_idx: Option<usize> = match input {
                            // Previous
                            Keycode::A => Some (
                                self.target_idx.checked_sub (1)
                                        .unwrap_or_else (|| self.potential_ids.len ().saturating_sub (1))
                            ),
                            // Next
                            Keycode::D => Some ((self.target_idx + 1) % self.potential_ids.len ()),
                            // Confirm
                            Keycode::Z => {
                                is_confirm = true;

                                Some (self.target_idx)
                            }
                            // Cancel
                            Keycode::X => {
                                is_cancel = true;

                                None
                            }
                            _ => {
                                println! ("Invalid input");

                                None
                            }
                        };

                        if let Some (target_idx) = target_idx {
                            self.target_idx = target_idx;

                            Some (Search::Radial (r))
                        } else {
                            None
                        }
                    }
                    Area::Path (w) => {
                        let direction: Option<Direction> = match input {
                            // Up
                            Keycode::W => Some (Direction::Up),
                            // Right
                            Keycode::A => Some (Direction::Right),
                            // Left
                            Keycode::S => Some (Direction::Left),
                            // Down
                            Keycode::D => Some (Direction::Down),
                            // Cancel
                            Keycode::X => {
                                is_cancel = true;

                                None
                            }
                            _ => {
                                println! ("Invalid input");

                                None
                            }
                        };

                        is_confirm = true;
                        direction.map (|d: Direction| Search::Path (w, self.range, d))
                    }
                };

                if let Some (search) = search {
                    self.target_ids = self.find_units_area_new (unit_id, self.target, search);

                    if !self.target_ids.is_empty () {
                        println! ("Targets: {:?}", self.target_ids);

                        if is_confirm {
                            self.state = State::AttackConfirm;
                        }
                    }
                } else if is_cancel {
                    self.state = State::Idle;
                }

                None
            }
            State::AttackConfirm => {
                match input {
                    // Confirm
                    Keycode::Z => {
                        self.state = State::Idle;
                        self.target_idx = 0;
                        self.act_attack_new (unit_id);
                        println! ("Attacking {:?}", self.target_ids);

                        for target_id in &self.target_ids {
                            println! ("{}: {}", target_id, self.units[*target_id].get_statistics ());
                        }

                        println! ("Self: {}", self.units[unit_id].get_statistics ());

                        if !self.unit_ids_dirty.is_empty () {
                            let unit_ids_attacked: Vec<ID> = self.unit_ids_dirty.drain ( .. ).collect ();
                
                            for unit_id in unit_ids_attacked {
                                if !self.units[unit_id].is_alive () {
                                    self.kill_unit (unit_id);
                                }
                            }
                        }

                        Some (Action::Attack)
                    }
                    // Cancel
                    Keycode::X => {
                        self.state = State::AttackTarget;

                        None
                    }
                    _ => {
                        println! ("Invalid input");

                        None
                    }
                }
            }
            State::SkillChoose => {
                let mut is_confirm: bool = false;
                let mut is_cancel: bool = false;
                let skill_idx: Option<usize> = {
                    match input {
                        // Previous
                        Keycode::A => Some (
                            self.skill_magic_idx.checked_sub (1)
                                    .unwrap_or_else (|| self.skill_magic_ids.len ().saturating_sub (1))
                        ),
                        // Next
                        Keycode::D => Some ((self.skill_magic_idx + 1) % self.skill_magic_ids.len ()),
                        // Confirm
                        Keycode::Z => {
                            is_confirm = true;

                            Some (self.skill_magic_idx)
                        }
                        // Cancel
                        Keycode::X => {
                            is_cancel = true;

                            None
                        }
                        _ => {
                            println! ("Invalid input");

                            None
                        }
                    }
                };

                if let Some (skill_idx) = skill_idx {
                    self.skill_magic_idx = skill_idx;
                    self.skill_magic_id = self.skill_magic_ids[skill_idx];
                    println! ("Skill: {}", self.skill_magic_id);

                    if is_confirm {
                        let skill: &Skill = self.scene.get_skill (&self.skill_magic_id);

                        self.state = State::SkillTarget;
                        self.target = skill.get_target ();
                        self.area = skill.get_area ();
                        self.range = skill.get_range ();
                        self.potential_ids = self.find_units_range_new (unit_id);
                        self.target_idx = 0;
                        println! ("Potential targets: {:?}", self.potential_ids);
                    }
                } else if is_cancel {
                    self.state = State::Idle;
                }

                None
            }
            State::SkillTarget => {
                let mut is_confirm: bool = false;
                let mut is_cancel: bool = false;
                let search: Option<Search> = match self.area {
                    Area::Single => {
                        let target_idx: Option<usize> = match input {
                            // Previous
                            Keycode::A => Some (
                                self.target_idx.checked_sub (1)
                                        .unwrap_or_else (|| self.potential_ids.len ().saturating_sub (1))
                            ),
                            // Next
                            Keycode::D => Some ((self.target_idx + 1) % self.potential_ids.len ()),
                            // Confirm
                            Keycode::Z => {
                                is_confirm = true;

                                Some (self.target_idx)
                            }
                            // Cancel
                            Keycode::X => {
                                is_cancel = true;

                                None
                            }
                            _ => {
                                println! ("Invalid input");

                                None
                            }
                        };

                        if let Some (target_idx) = target_idx {
                            self.target_idx = target_idx;

                            Some (Search::Single)
                        } else {
                            None
                        }
                    }
                    Area::Radial (r) => {
                        let target_idx: Option<usize> = match input {
                            // Previous
                            Keycode::A => Some (
                                self.target_idx.checked_sub (1)
                                        .unwrap_or_else (|| self.potential_ids.len ().saturating_sub (1))
                            ),
                            // Next
                            Keycode::D => Some ((self.target_idx + 1) % self.potential_ids.len ()),
                            // Confirm
                            Keycode::Z => {
                                is_confirm = true;

                                Some (self.target_idx)
                            }
                            // Cancel
                            Keycode::X => {
                                is_cancel = true;

                                None
                            }
                            _ => {
                                println! ("Invalid input");

                                None
                            }
                        };

                        if let Some (target_idx) = target_idx {
                            self.target_idx = target_idx;

                            Some (Search::Radial (r))
                        } else {
                            None
                        }
                    }
                    Area::Path (w) => {
                        let direction: Option<Direction> = match input {
                            // Up
                            Keycode::W => Some (Direction::Up),
                            // Right
                            Keycode::A => Some (Direction::Right),
                            // Left
                            Keycode::S => Some (Direction::Left),
                            // Down
                            Keycode::D => Some (Direction::Down),
                            // Cancel
                            Keycode::X => {
                                is_cancel = true;

                                None
                            }
                            _ => {
                                println! ("Invalid input");

                                None
                            }
                        };

                        is_confirm = true;
                        direction.map (|d: Direction| Search::Path (w, self.range, d))
                    }
                };

                if let Some (search) = search {
                    self.target_ids = self.find_units_area_new (unit_id, self.target, search);

                    if !self.target_ids.is_empty () {
                        println! ("Targets: {:?}", self.target_ids);

                        if is_confirm {
                            self.state = State::SkillConfirm;
                        }
                    }
                } else if is_cancel {
                    self.state = State::SkillChoose;
                }

                None
            }
            State::SkillConfirm => {
                match input {
                    // Confirm
                    Keycode::Z => {
                        for target_id in &self.target_ids {
                            println! ("{}", self.units[*target_id]);
                        }

                        self.state = State::Idle;
                        self.target_idx = 0;
                        self.act_skill_new (unit_id);

                        println! ("Using skill {} on {:?}", self.skill_magic_id, self.target_ids);

                        for target_id in &self.target_ids {
                            println! ("{}", self.units[*target_id]);
                        }

                        Some (Action::Skill)
                    }
                    // Cancel
                    Keycode::X => {
                        self.state = State::SkillTarget;

                        None
                    }
                    _ => {
                        println! ("Invalid input");

                        None
                    }
                }
            }
            State::MagicChoose => {
                let mut is_confirm: bool = false;
                let mut is_cancel: bool = false;
                let magic_idx: Option<usize> = {
                    match input {
                        // Previous
                        Keycode::A => Some (
                            self.skill_magic_idx.checked_sub (1)
                                    .unwrap_or_else (|| self.skill_magic_ids.len ().saturating_sub (1))
                        ),
                        // Next
                        Keycode::D => Some ((self.skill_magic_idx + 1) % self.skill_magic_ids.len ()),
                        // Confirm
                        Keycode::Z => {
                            is_confirm = true;

                            Some (self.skill_magic_idx)
                        }
                        // Cancel
                        Keycode::X => {
                            is_cancel = true;

                            None
                        }
                        _ => {
                            println! ("Invalid input");

                            None
                        }
                    }
                };

                if let Some (magic_idx) = magic_idx {
                    self.skill_magic_idx = magic_idx;
                    self.skill_magic_id = self.skill_magic_ids[magic_idx];
                    println! ("Magic: {}", self.skill_magic_id);

                    if is_confirm {
                        let magic: &Magic = self.scene.get_magic (&self.skill_magic_id);

                        self.state = State::MagicTarget;
                        self.target = magic.get_target ();
                        self.area = magic.get_area ();
                        self.range = magic.get_range ();

                        match self.target {
                            Target::This => println! ("Potential target: Self"),
                            Target::Map => {
                                self.target_location = *self.grid.get_unit_location (&unit_id);
                                self.potential_locations = self.find_locations_range_new (unit_id);
                                println! ("Potential targets: {:?}", self.potential_locations);
                            }
                            _ => panic! ("Invalid target {:?}", self.target),
                        }

                        self.target_idx = 0;
                    }
                } else if is_cancel {
                    self.state = State::Idle;
                }

                None
            }
            State::MagicTarget => {
                let mut is_confirm: bool = false;
                let mut is_cancel: bool = false;

                match self.target {
                    Target::This => match input {
                        // Confirm
                        Keycode::Z => self.state = State::MagicConfirm,
                        // Cancel
                        Keycode::X => self.state = State::MagicChoose,
                        _ => println! ("Invalid input"),
                    }
                    Target::Map => {
                        let direction: Option<Direction> = match input {
                            // Up
                            Keycode::W => Some (Direction::Up),
                            // Left
                            Keycode::A => Some (Direction::Left),
                            // Down
                            Keycode::S => Some (Direction::Down),
                            // Right
                            Keycode::D => Some (Direction::Right),
                            // Confirm
                            Keycode::Z => {
                                is_confirm = true;

                                None
                            }
                            // Cancel
                            Keycode::X => {
                                is_cancel = true;

                                None
                            }
                            _ => {
                                println! ("Invalid input");

                                None
                            }
                        };

                        match self.area {
                            Area::Single | Area::Radial ( .. ) => if let Some (direction) = direction {
                                if let Some (end) = self.grid.try_connect (&self.target_location, direction) {
                                    
                                    if self.potential_locations.contains (&end) {
                                        println! ("{:?}", direction);
                                        self.target_location = end;
                                    } else {
                                        println! ("Invalid direction {:?}", direction);
                                    }
                                } else {
                                    println! ("Invalid direction {:?}", direction);
                                }
                            }
                            Area::Path ( .. ) => is_confirm = direction.is_some (),
                        }

                        if is_confirm {
                            let search: Search = match self.area {
                                Area::Single => Search::Single,
                                Area::Radial (r) => Search::Radial (r),
                                Area::Path (w) => {
                                    let direction: Direction = direction.unwrap_or_else (|| panic! ("Invalid direction {:?}", direction));

                                    Search::Path (w, self.range, direction)
                                }
                            };

                            self.target_locations = self.find_locations_area_new (search);
        
                            if !self.target_locations.is_empty () {
                                println! ("Targets: {:?}", self.target_locations);
                                self.state = State::MagicConfirm;
                            }
                        } else if is_cancel {
                            self.state = State::MagicChoose;
                        }
                    }
                    _ => panic! ("Invalid target {:?}", self.target),
                }

                None
            }
            State::MagicConfirm => {
                match input {
                    // Confirm
                    Keycode::Z => {
                        for target_id in &self.target_ids {
                            println! ("{}", self.units[*target_id]);
                        }

                        self.state = State::Idle;
                        self.target_idx = 0;
                        self.act_magic_new (unit_id);

                        match self.target {
                            Target::This => {
                                println! ("Using magic {} on {}", self.skill_magic_id, unit_id);
                                println! ("{}", self.units[unit_id]);
                            }
                            Target::Map => {
                                println! ("Using magic {} on {:?}", self.skill_magic_id, self.target_locations);

                                for target_location in &self.target_locations {
                                    println! ("{:?}: {}", target_location, self.grid.get_tile (target_location));
                                }
                            }
                            _ => panic! ("Invalid target {:?}", self.target),
                        }



                        Some (Action::Magic)
                    }
                    // Cancel
                    Keycode::X => {
                        self.state = State::MagicTarget;

                        None
                    }
                    _ => {
                        println! ("Invalid input");

                        None
                    }
                }
            }
        };

        if let Some (action) = action {
            let turn: Turn = self.turn.take ()
                    .expect ("Turn not found");

            println! ("End {}'s turn\n", unit_id);
            let _ = self.sender.send (format! ("End {}'s turn", unit_id));
            self.send_passive (unit_id);

            let (delay, mov): (u16, u16) = if self.units[unit_id].is_alive () {
                let mov: u16 = self.units[unit_id].get_statistic (UnitStatistic::MOV).0;

                self.end_turn (unit_id);

                (get_delay (mov, action), mov)
            } else {
                (u16::MAX, u16::MAX)
            };

            if self.units[unit_id].is_alive () {
                self.update_turns (turn, delay, mov);
            } else {
                self.kill_unit (unit_id);
            }

            true
        } else {
            false
        }

        // Action::Magic => {
        //     if is_rout {
        //         println! ("Unit cannot use magic (is routed)")
        //     } else {
        //         let magic_ids: &[ID] = self.units[unit_id].get_magic_ids ();
                
        //         if magic_ids.is_empty () {
        //             println! ("No available magics");
        //         } else {
        //             let validator: IDValidator = IDValidator::new (magic_ids);

        //             println! ("Available magics: {:?}", magic_ids);
        //             let _ = self.sender.send (format! ("{}'s action: Magic", unit_id));

        //             if let Some (magic_id) = self.reader.read_validate (&validator) {
        //                 let magic: &Magic = self.scene.get_magic (&magic_id);
        //                 let target: Target = magic.get_target ();
        //                 let area: Area = magic.get_area ();
        //                 let range: u8 = magic.get_range ();

        //                 match target {
        //                     Target::This => {
        //                         let target_ids: Vec<ID> = self.find_units (unit_id, target, area, range);

        //                         if !target_ids.is_empty () {
        //                             let mov: u16 = self.act_magic (unit_id, None, magic_id);

        //                             println! ("Using magic {} on self", magic_id);

        //                             for target_id in target_ids {
        //                                 println! ("Target: {}", self.units[target_id]);
        //                             }

        //                             break (mov, FACTOR_MAGIC)
        //                         }
        //                     }
        //                     Target::Map => {
        //                         let target_locations: Vec<Location> =
        //                             self.find_locations (unit_id, area, range);

        //                         if !target_locations.is_empty () {
        //                             let mov: u16 = self.act_magic (unit_id, Some (&target_locations), magic_id);

        //                             println! ("Using magic {} on {:?}", magic_id, target_locations);
        //                             println! ("Targets: {:?}", target_locations);

        //                             break (mov, FACTOR_MAGIC)
        //                         }
        //                     }
        //                     _ => panic! ("Invalid target {:?}", target),
        //                 }
        //             }
        //         }
        //     }
        // }
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
        let modifier_9 = *game.scene.get_modifier (&9);
        let mut modifier_9 = Box::new (modifier_9);
        let modifier_4 = *game.scene.get_modifier (&4);
        let modifier_4 = Box::new (modifier_4);

        game.grid.place_unit (0, (1, 1));
        game.grid.place_unit (1, (0, 0));
        modifier_9.set_applier_id (0);

        // Test impassable start
        game.grid.add_appliable (&(1, 2), modifier_9);
        game.move_unit (0, &[Direction::Right]);
        game.grid.decrement_durations (&0);
        game.grid.decrement_durations (&0);
        game.start_turn (0);
        assert! (!game.units[0].is_alive ());
        // Test normal start
        game.units[1].add_appliable (modifier_4);
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
        let attribute_9 = *game.scene.get_attribute (&9);
        let attribute_9 = Box::new (attribute_9);
        let attribute_10 = *game.scene.get_attribute (&10);
        let attribute_10 = Box::new (attribute_10);

        game.units[0].add_appliable (attribute_10);
        game.units[2].add_appliable (attribute_9);

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

        game.place_unit (0, (1, 0));

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
        assert! (game.grid.try_yield_appliable (&(1, 0)).is_some ());
        // -40 HLT from magic, -20 HLT from OnOccupy
        assert_eq! (game.units[0].get_statistic (HLT).0, 940);
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

        game.factions[0].add_follower (1, 0);
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
