use super::{ActionValidator, ConfirmationValidator, DirectionValidator, IndexValidator, MovementValidator, RenderContext, Turn, Validator};
use crate::character::{Faction, FactionBuilder, Magic, Skill, Tool, Unit, UnitBuilder, UnitStatistic, UnitStatistics, Weapon};
use crate::common::{FACTOR_ATTACK, FACTOR_MAGIC, FACTOR_SKILL, FACTOR_WAIT, ID, Scene, Target};
use crate::dynamic::{Appliable, AppliableKind, Applier, Dynamic};
use crate::map::{Area, Direction, Grid, Location, Search};
use sdl2::keyboard::Keycode;
use std::collections::{BinaryHeap, HashSet};
use std::error::Error;
use std::ops::ControlFlow::{Break, Continue};
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
#[derive (Clone, Copy)]
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
    TargetAttack,
    ConfirmAttack,
    ChooseSkill,
    TargetSkill,
    ConfirmSkill,
    ChooseMagic,
    TargetMagic,
    ConfirmMagic,
}

#[derive (Debug)]
pub enum Context<'a> {
    Idle,
    Move {
        location: Location,
        // movements: Vec<Direction>,
        // mov: u16,
    },
    TargetAttack {
        target: Target,
        area: Area,
        range: u8,
        // target_idx: usize,
        potential_ids: &'a [ID],
    },
    ConfirmAttack {
        target_ids: &'a [ID],
    },
    ChooseSkill {
        // skill_idx: usize,
        skill_ids: &'a [ID],
    },
    TargetSkill {
        target: Target,
        area: Area,
        range: u8,
        skill_id: ID,
        // target_idx: usize,
        potential_ids: &'a [ID],
    },
    ConfirmSkill {
        target_ids: &'a [ID],
    },
    ChooseMagic {
        // magic_idx: usize,
        magic_ids: &'a [ID],
    },
    TargetMagic {
        target: Target,
        area: Area,
        range: u8,
        magic_id: ID,
        potential_id: ID,
        potential_locations: &'a [Location], // empty Vec -> This, populated Vec -> Map
        target_location: Location,
    },
    ConfirmMagic {
        target_locations: &'a [Location], // empty Vec -> This, populated Vec -> Map
    },
}

#[derive (Debug)]
pub struct Game {
    scene: Rc<Scene>,
    state: State,
    sender: Sender<String>,
    turn: Option<Turn>,
    turns: BinaryHeap<Turn>,
    number_turns: usize,
    grid: Grid,
    units: Vec<Unit>,
    factions: Vec<Faction>,
    // Action context
    action: Action,
    location: Location,
    movements: Vec<Direction>,
    mov: u16,
    target: Target,
    area: Area,
    range: u8,
    target_idx: usize,
    target_location: Location,
    potential_ids: Vec<ID>,
    potential_locations: Vec<Location>,
    target_ids: Vec<ID>,
    target_locations: Vec<Location>,
    skill_magic_idx: usize,
    skill_magic_ids: Vec<ID>,
    skill_magic_id: ID,
}

impl Game {
    pub fn new (scene: Scene, sender: Sender<String>) -> Self {
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
        let action: Action = Action::Wait;
        let location: Location = (usize::MAX, usize::MAX);
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

        let _ = sender.send (String::from ("Game creation complete"));

        Self { scene, state, sender, turn, turns, number_turns, grid, units, factions, action, location, movements, mov, target, area, range, target_idx, target_location, potential_ids, potential_locations, target_ids, target_locations, skill_magic_idx, skill_magic_ids, skill_magic_id }
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

    // TODO: Maybe just pass Game into Renderer
    pub fn get_render_context (&self) -> RenderContext {
        // TODO: Obviously this can be cached
        let mut terrains: Vec<Vec<ID>> = Vec::new ();
        let mut unit_locations: Vec<Option<Location>> = Vec::new ();

        for (i, row) in self.scene.get_tile_builders ().iter ().enumerate () {
            terrains.push (Vec::new ());

            for tile_builder in row.iter () {
                let terrain_id: ID = tile_builder.get_terrain_id ();

                terrains[i].push (terrain_id);
            }
        }

        for unit_id in 0 .. self.units.len () {
            let location: Option<Location> = self.grid.get_unit_location (&unit_id).copied ();

            unit_locations.push (location);
        }

        RenderContext::new (terrains, unit_locations)
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
        let location: Location = *self.grid.get_unit_location (&unit_id)
                .unwrap_or_else (|| panic! ("Location not found for unit {}", unit_id));
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

    fn move_unit (&mut self, unit_id: ID) -> Location {
        let (location, terrain_id): (Location, ID) = self.grid
                .move_unit (unit_id, &self.movements)
                .unwrap_or_else (|| panic! ("Invalid movements {:?}", self.movements));

        self.apply_terrain (unit_id, terrain_id, location);
        self.try_spawn_recruit (unit_id);

        location
    }

    
    fn attack_unit (&mut self, attacker_id: ID) {
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

            // self.unit_ids_dirty.push (*defender_id);
        }
    }

    fn use_skill_unit (&mut self, user_id: ID) {
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

    fn use_magic_unit (&mut self, user_id: ID) {
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

    // fn find_units_from (&mut self, locations: &[Location]) -> Vec<ID> {
    //     locations.iter ().filter_map (|l: &Location|
    //         self.grid.get_location_unit (l).copied ()
    //     ).collect::<Vec<ID>> ()
    // }

    fn find_units_area (&self, unit_id: ID, target: Target, search: Search) -> Vec<ID> {
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
                    let location: &Location = self.grid.get_unit_location (&target_id)
                            .unwrap_or_else (|| panic! ("Location not found for unit {}", unit_id));
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
                    let location: &Location = self.grid.get_unit_location (&unit_id)
                            .unwrap_or_else (|| panic! ("Location not found for unit {}", unit_id));
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

    fn find_units_area_new (&self, unit_id: ID, search: Search) -> Vec<ID> {
        assert! (!self.potential_ids.is_empty ());

        match self.target {
            Target::This => vec![self.potential_ids[0]],
            Target::Ally | Target::Enemy => {
                let target_id: ID = self.potential_ids[self.target_idx];

                vec![target_id]
            }
            Target::Allies | Target::Enemies => match search {
                Search::Single => panic! ("Invalid search {:?} for target {:?}", search, self.target),
                Search::Radial (r) => {
                    let target_id: ID = self.potential_ids[self.target_idx];
                    let location: &Location = self.grid.get_unit_location (&target_id)
                            .unwrap_or_else (|| panic! ("Location not found for unit {}", target_id));
                    let target_ids: Vec<ID> = self.grid.find_units (location, Search::Radial (r));
                    let faction_id: ID = self.units[unit_id].get_faction_id ();

                    if let Target::Allies = self.target {
                        self.filter_unit_allegiance (&target_ids, faction_id, true)
                    } else if let Target::Enemies = self.target {
                        self.filter_unit_allegiance (&target_ids, faction_id, false)
                    } else {
                        unreachable! ()
                    }
                }
                Search::Path (w, r, d) => {
                    let location: &Location = self.grid.get_unit_location (&unit_id)
                            .unwrap_or_else (|| panic! ("Location not found for unit {}", unit_id));
                    let target_ids: Vec<ID> = self.grid.find_units (location, Search::Path (w, r, d));
                    let faction_id: ID = self.units[unit_id].get_faction_id ();

                    if let Target::Allies = self.target {
                        self.filter_unit_allegiance (&target_ids, faction_id, true)
                    } else if let Target::Enemies = self.target {
                        self.filter_unit_allegiance (&target_ids, faction_id, false)
                    } else {
                        unreachable! ()
                    }
                }
            },
            _ => panic! ("Invalid target {:?}", self.target),
        }
    }

    fn find_units_range (&self, unit_id: ID, target: Target, area: Area, range: u8) -> Vec<ID> {
        if let Target::Map = target {
            panic! ("Invalid target {:?}", target)
        } else if let Target::This = target {
            vec![unit_id]
        } else {
            let location: &Location = self.grid.get_unit_location (&unit_id)
                    .unwrap_or_else (|| panic! ("Location not found for unit {}", unit_id));
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

    fn find_locations_area (&self, search: Search) -> Vec<Location> {
        match search {
            Search::Single => vec![self.target_location],
            Search::Radial ( .. ) | Search::Path ( .. ) => self.grid.find_locations (&self.target_location, search),
        }
    }

    fn find_locations_range (&self, unit_id: ID, range: u8) -> Vec<Location> {
        let location: &Location = self.grid.get_unit_location (&unit_id)
                .unwrap_or_else (|| panic! ("Location not found for unit {}", unit_id));

        if let Area::Path (w) = self.area {
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

    fn start_turn (&mut self, unit_id: ID) {
        let location: &Location = self.grid.get_unit_location (&unit_id)
                .unwrap_or_else (|| panic! ("Location not found for unit {}", unit_id));

        if self.grid.is_impassable (location) {
            self.units[unit_id].set_statistic (UnitStatistic::HLT, 0);
        } else {
            self.units[unit_id].start_turn ();
        }
    }

    fn wait_unit (&mut self, unit_id: ID) -> u16 {
        self.units[unit_id].act_wait ()
    }

    fn end_turn (&mut self, unit_id: ID) {
        let city_ids: Vec<ID> = self.grid.find_unit_cities (&unit_id);
        let location: Location = *self.grid.get_unit_location (&unit_id)
                .unwrap_or_else (|| panic! ("Location not found for unit {}", unit_id));
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

    pub fn load_scene (&mut self) {
        todo! ()
    }

    pub fn display_turn (&self) {
        let turn: &Turn = self.turn.as_ref ().unwrap_or_else (|| self.turns.peek ().unwrap ());
        let unit_id: ID = turn.get_unit_id ();

        println! ("{}'s turn", unit_id);
        print! ("{}", self.grid);
        println! ("Turn order: {:?}\n", self.turns);
    }

    pub fn display_prompt (&self) {
        let prompt: &str = match &self.state {
            State::Idle => ActionValidator::get_prompt (),
            State::Move => MovementValidator::get_prompt (),
            State::TargetAttack => match self.area {
                Area::Single | Area::Radial ( .. ) => IndexValidator::get_prompt (),
                Area::Path ( .. ) => DirectionValidator::get_prompt (),
            }
            State::ConfirmAttack => ConfirmationValidator::get_prompt (),
            State::ChooseSkill => IndexValidator::get_prompt (),
            State::TargetSkill => IndexValidator::get_prompt (),
            State::ConfirmSkill => ConfirmationValidator::get_prompt (),
            State::ChooseMagic => IndexValidator::get_prompt (),
            State::TargetMagic => if self.potential_locations.is_empty () {
                ConfirmationValidator::get_prompt ()
            } else {
                MovementValidator::get_prompt ()
            }
            State::ConfirmMagic => ConfirmationValidator::get_prompt (),
        };

        println! ("{}", prompt);
    }

    fn change_state (&mut self, context: Context) {
        match context {
            Context::Idle => {
                self.state = State::Idle;
                println! ("Idle");
            }
            Context::Move { location } => {
                self.state = State::Move;
                self.location = location;
                self.movements.clear ();
                // mov is updated elsewhere
                println! ("Move");
            }
            Context::TargetAttack { target, area, range, potential_ids } => {
                self.state = State::TargetAttack;
                self.target = target;
                self.area = area;
                self.range = range;
                self.target_idx = 0;
                self.potential_ids.clear ();
                self.potential_ids.extend_from_slice (potential_ids);
                println! ("Target (attack)");
            }
            Context::ConfirmAttack { target_ids } => {
                self.state = State::ConfirmAttack;
                self.target_ids.clear ();
                self.target_ids.extend_from_slice (target_ids);
                println! ("Confirm (attack)");
            }
            Context::ChooseSkill { skill_ids } => {
                self.state = State::ChooseSkill;
                self.skill_magic_idx = 0;
                self.skill_magic_ids.clear ();
                self.skill_magic_ids.extend_from_slice (skill_ids);
                println! ("Choose (skill)");
            }
            Context::TargetSkill { target, area, range, skill_id, potential_ids } => {
                self.state = State::TargetSkill;
                self.target = target;
                self.area = area;
                self.range = range;
                self.skill_magic_id = skill_id;
                self.target_idx = 0;
                self.potential_ids.clear ();
                self.potential_ids.extend_from_slice (potential_ids);
                println! ("Target (skill)");
            }
            Context::ConfirmSkill { target_ids } => {
                self.state = State::ConfirmSkill;
                self.target_ids.clear ();
                self.target_ids.extend_from_slice (target_ids);
                println! ("Confirm (skill)");
            }
            Context::ChooseMagic { magic_ids } => {
                self.state = State::ChooseMagic;
                self.skill_magic_idx = 0;
                self.skill_magic_ids.clear ();
                self.skill_magic_ids.extend_from_slice (magic_ids);
                println! ("Choose (magic)");
            }
            Context::TargetMagic { target, area, range, magic_id, potential_id, potential_locations, target_location } => {
                self.state = State::TargetMagic;
                self.target = target;
                self.area = area;
                self.range = range;
                self.skill_magic_id = magic_id;
                self.target_idx = 0;
                self.potential_ids = vec![potential_id];
                self.potential_locations.clear ();
                self.potential_locations.extend_from_slice (potential_locations);
                self.target_location = target_location;
                println! ("Target (magic)");
            }
            Context::ConfirmMagic { target_locations } => {
                self.state = State::ConfirmMagic;
                self.target_locations.clear ();
                self.target_locations.extend_from_slice (target_locations);
                println! ("Confirm (magic)");
            }
        }
    }

    fn revert_state (&mut self) {
        self.state = match self.state {
            State::Idle => State::Idle,
            State::Move  => State::Idle,
            State::TargetAttack => State::Idle,
            State::ConfirmAttack => {
                self.target_idx = 0;

                State::TargetAttack
            }
            State::ChooseSkill => State::Idle,
            State::TargetSkill => {
                self.skill_magic_idx = 0;

                State::ChooseSkill
            }
            State::ConfirmSkill => {
                self.target_idx = 0;

                State::TargetSkill
            }
            State::ChooseMagic => State::Idle,
            State::TargetMagic => {
                self.skill_magic_idx = 0;

                State::ChooseMagic
            }
            State::ConfirmMagic => {
                // self.target_location = (0, 0);

                State::TargetMagic
            }
        }
    }

    fn act_idle (&mut self, input: Keycode, unit_id: ID) -> Option<Action> {
        let is_retreat: bool = self.units[unit_id].is_retreat ();
        let is_rout: bool = self.units[unit_id].is_rout ();

        match ActionValidator.validate (input) {
            Ok (flow) => {
                match flow {
                    Break (action) => if let Some (action) = action {
                        self.action = action;

                        match action {
                            Action::Attack => {
                                println! ("{}'s action: Attack", unit_id);
                                let _ = self.sender.send (format! ("{}'s action: Attack", unit_id));

                                if is_retreat {
                                    println! ("Unit cannot attack (is retreating)")
                                } else {
                                    let (target, area, range): (Target, Area, u8) = {
                                        let weapon: &Weapon = self.units[unit_id].get_weapon ();

                                        (weapon.get_target (), weapon.get_area (), weapon.get_range ())
                                    };
                                    let potential_ids: Vec<ID> = self.find_units_range (unit_id, target, area, range);

                                    let _ = self.sender.send (format! ("Potential targets: {:?}", potential_ids));

                                    if potential_ids.is_empty () {
                                        println! ("No available targets");
                                    } else {
                                        self.change_state (Context::TargetAttack {
                                            target,
                                            area,
                                            range,
                                            potential_ids: &potential_ids,
                                        });
                                        println! ("Potential targets: {:?}", self.potential_ids);
                                        println! ("Equipped weapon: {:?}", self.units[unit_id].get_weapon ());
                                    }
                                }

                                None
                            }
                            Action::Weapon => {
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
                            Action::Skill => {
                                println! ("{}'s action: Skill", unit_id);
                                let _ = self.sender.send (format! ("{}'s action: Skill", unit_id));

                                if is_rout {
                                    println! ("Unit cannot use skill (is routed)")
                                } else {
                                    let skill_ids: Vec<ID> = self.units[unit_id].get_skill_ids_actionable ();

                                    if skill_ids.is_empty () {
                                        println! ("No available skills");
                                    } else {
                                        self.change_state (Context::ChooseSkill {
                                            skill_ids: &skill_ids,
                                        });
                                        println! ("Skills: {:?}", self.skill_magic_ids);
                                    }
                                }

                                None
                            }
                            Action::Magic => {
                                let _ = self.sender.send (format! ("{}'s action: Magic", unit_id));

                                if is_rout {
                                    println! ("Unit cannot use magic (is routed)")
                                } else {
                                    let magic_ids: Vec<ID> = self.units[unit_id].get_magic_ids ().to_vec ();

                                    if magic_ids.is_empty () {
                                        println! ("No available magics");
                                    } else {
                                        self.change_state (Context::ChooseMagic {
                                            magic_ids: &magic_ids,
                                        });
                                        println! ("Magics: {:?}", self.skill_magic_ids);
                                    }
                                }

                                None
                            }
                            Action::Move => {
                                println! ("{}'s action: Move", unit_id);
                                println! ("Movable locations: {:?}", self.grid.find_unit_movable (&unit_id, self.mov));
                                let _ = self.sender.send (format! ("{}'s action: Move", unit_id));

                                self.change_state (Context::Move {
                                    location: *self.grid.get_unit_location (&unit_id)
                                            .unwrap_or_else (|| panic! ("Location not found for unit {}", unit_id)),
                                    // mov is updated on turn change
                                });
                                self.grid.set_unit_id_passable (Some (unit_id));
                                println! ("Current location: {:?}", self.location);

                                None
                            }
                            Action::Wait => {
                                println! ("{}'s action: Wait", unit_id);
                                let _ = self.sender.send (format! ("{}'s action: Wait", unit_id));
                                self.wait_unit (unit_id);

                                Some (Action::Wait)
                            }
                        }
                    } else {
                        self.revert_state ();

                        None
                    }
                    Continue ( .. ) => unreachable! (),
                }
            }
            Err (e) => {
                println! ("{}", e);

                None
            }
        }
    }

    fn act_move (&mut self, input: Keycode, unit_id: ID) -> Option<Action> {
        match MovementValidator.validate (input) {
            Ok (flow) => {
                match flow {
                    Break (is_confirm) => if is_confirm {
                        // self.grid.set_unit_id_passable (None);
                        self.change_state (Context::Idle);
                        self.move_unit (unit_id);
                        println! ("{:?}", self.movements);
                        println! ("{:?}, {} MOV remaining", self.grid.get_unit_location (&unit_id), self.mov);
                        print! ("{}", self.grid);
                        let _ = self.sender.send (format! ("Movements: {:?}", self.movements));
                    } else {
                        self.revert_state ();
                    }
                    Continue (direction) => if let Some ((end, cost)) = self.grid.try_move (&self.location, direction) {
                        println! ("{:?}", direction);
                        if self.mov >= (cost as u16) {
                            self.location = end;
                            self.movements.push (direction);
                            self.mov -= cost as u16;
                        } else {
                            println! ("Insufficient MOV");
                        }
                    } else {
                        println! ("Invalid direction {:?}", direction);
                    }
                }
            }
            Err (e) => println! ("{}", e),
        }

        None
    }

    fn act_choose (&mut self, input: Keycode, unit_id: ID) -> Option<Action> {
        match IndexValidator::new (self.skill_magic_idx, self.skill_magic_ids.len ()).validate (input) {
            Ok (flow) => {
                match flow {
                    Break (index) => if let Some (index) = index {
                        let skill_magic_id: ID = self.skill_magic_ids[index];
                        let (target, area, range): (Target, Area, u8) = {
                            match self.action {
                                Action::Skill => {
                                    let skill: &Skill = self.scene.get_skill (&skill_magic_id);

                                    (skill.get_target (), skill.get_area (), skill.get_range ())
                                }
                                Action::Magic => {
                                    let magic: &Magic = self.scene.get_magic (&skill_magic_id);

                                    (magic.get_target (), magic.get_area (), magic.get_range ())
                                }
                                _ => panic! ("Invalid action {:?}", self.action),
                            }
                        };

                        match self.action {
                            Action::Skill => {
                                let potential_ids: Vec<ID> = self.find_units_range (unit_id, target, area, range);

                                let _ = self.sender.send (format! ("Potential targets: {:?}", potential_ids));

                                if potential_ids.is_empty () {
                                    println! ("No available targets");
                                } else {
                                    self.change_state (Context::TargetSkill {
                                        target,
                                        area,
                                        range,
                                        skill_id: skill_magic_id,
                                        potential_ids: &potential_ids,
                                    });
                                    println! ("Potential targets: {:?}", self.potential_ids);
                                    println! ("Chosen skill: {:?}", self.scene.get_skill (&skill_magic_id));
                                }
                            }
                            Action::Magic => {
                                let potential_locations: Vec<Location> = match target {
                                    Target::This => Vec::new (),
                                    Target::Map => self.find_locations_range (unit_id, range),
                                    _ => panic! ("Invalid target {:?}", target),
                                };
        
                                self.change_state (Context::TargetMagic {
                                    target,
                                    area,
                                    range,
                                    magic_id: skill_magic_id,
                                    potential_id: unit_id,
                                    potential_locations: &potential_locations,
                                    target_location: *self.grid.get_unit_location (&unit_id)
                                            .unwrap_or_else (|| panic! ("Location not found for unit {}", unit_id)),
                                });

                                if self.potential_locations.is_empty () {
                                    println! ("Potential target: Self");
                                } else {
                                    println! ("Potential targets: {:?}", self.potential_locations);
                                }

                                println! ("Chosen magic: {:?}", self.scene.get_magic (&skill_magic_id));
                                let _ = self.sender.send (format! ("Potential targets: {:?}", self.potential_locations));
                            }
                            _ => panic! ("Invalid action {:?}", self.action),
                        }
                    } else {
                        self.revert_state ();
                    }
                    Continue (index) => self.skill_magic_idx = index,
                }
            }
            Err (e) => println! ("{}", e),
        }

        println! ("{:?}: {}", self.action, self.skill_magic_ids[self.skill_magic_idx]);

        None
    }

    fn act_target (&mut self, input: Keycode, unit_id: ID) -> Option<Action> {
        if let Target::Map = self.target {
            let search: Option<Search> = if let Area::Path (w) = self.area {
                match DirectionValidator.validate (input) {
                    Ok (flow) => {
                        match flow {
                            Break (direction) => if let Some (direction) = direction {
                                println! ("{:?}", direction);

                                Some (Search::Path (w, self.range, direction))
                            } else {
                                self.revert_state ();

                                None
                            }
                            Continue ( .. ) => unreachable! (),
                        }
                    }
                    Err (e) => {
                        println! ("{}", e);

                        None
                    }
                }
            } else {
                match MovementValidator.validate (input) {
                    Ok (flow) => {
                        match flow {
                            Break (is_confirm) => if is_confirm {
                                match self.area {
                                    Area::Single => Some (Search::Single),
                                    Area::Radial (r) => Some (Search::Radial (r)),
                                    Area::Path ( .. ) => unreachable! (),
                                }
                            } else {
                                self.revert_state ();

                                None
                            }
                            Continue (direction) => if let Some (end) = self.grid.try_connect (&self.target_location, direction) {                                
                                if self.potential_locations.contains (&end) {
                                    self.target_location = end;
                                    println! ("Target: {:?}", self.target_location);
                                    println! ("{:?}", direction);
                                } else {
                                    println! ("Invalid direction {:?}", direction);
                                }

                                None
                            } else {
                                println! ("Invalid direction {:?}", direction);

                                None
                            }
                        }
                    }
                    Err (e) => {
                        println! ("{}", e);

                        None
                    }
                }
            };

            if let Some (search) = search {
                let target_locations: Vec<Location> = self.find_locations_area (search);

                if target_locations.is_empty () {
                    println! ("No available targets");
                } else {
                    if let Action::Magic = self.action {
                        self.change_state (Context::ConfirmMagic {
                            target_locations: &target_locations,
                        });
                    } else {
                        panic! ("Invalid action {:?}", self.action);
                    }

                    println! ("Targets: {:?}", self.target_locations);
                }
            }
        } else {
            let search: Option<Search> = if let Area::Path (w) = self.area {
                match DirectionValidator.validate (input) {
                    Ok (flow) => {
                        match flow {
                            Break (direction) => if let Some (direction) = direction {
                                println! ("{:?}", direction);

                                Some (Search::Path (w, self.range, direction))
                            } else {
                                self.revert_state ();

                                None
                            }
                            Continue ( .. ) => unreachable! (),
                        }
                    }
                    Err (e) => {
                        println! ("{}", e);

                        None
                    }
                }
            // TODO: Does Magic really need this special case?
            } else if let Action::Magic = self.action {
                match ConfirmationValidator.validate (input) {
                    Ok (flow) => {
                        match flow {
                            Break (is_confirm) => if is_confirm {
                                println! ("Target: Self");

                                Some (Search::Single)
                            } else {
                                self.revert_state ();

                                None
                            }
                            Continue ( .. ) => unreachable! (),
                        }
                    }
                    Err (e) => {
                        println! ("{}", e);

                        None
                    }
                }
            } else {
                match IndexValidator::new (self.target_idx, self.potential_ids.len ()).validate (input) {
                    Ok (flow) => {
                        match flow {
                            Break (index) => if let Some (index) = index {
                                self.target_idx = index;

                                match self.area {
                                    Area::Single => Some (Search::Single),
                                    Area::Radial (r) => Some (Search::Radial (r)),
                                    Area::Path ( .. ) => unreachable! (),
                                }
                            } else {
                                self.revert_state ();

                                None
                            }
                            Continue (index) => {
                                self.target_idx = index;
                                println! ("Target: {:?}", self.potential_ids[self.target_idx]);

                                None
                            }
                        }
                    }
                    Err (e) => {
                        println! ("{}", e);

                        None
                    }
                }
            };

            if let Some (search) = search {
                let target_ids: Vec<ID> = self.find_units_area_new (unit_id, search);

                if target_ids.is_empty () {
                    println! ("No available targets");
                } else {
                    match self.action {
                        Action::Attack => self.change_state (Context::ConfirmAttack {
                            target_ids: &target_ids,
                        }),
                        Action::Skill => self.change_state (Context::ConfirmSkill {
                            target_ids: &target_ids,
                        }),
                        Action::Magic => self.change_state (Context::ConfirmMagic {
                            target_locations: &[],
                        }),
                        _ => panic! ("Invalid action {:?}", self.action),
                    }

                    println! ("Targets: {:?}", self.target_ids);
                }
            }
        }

        None
    }

    fn act_confirm (&mut self, input: Keycode, unit_id: ID) -> Option<Action> {
        match ConfirmationValidator.validate (input) {
            Ok (flow) => {
                match flow {
                    Break (is_confirm) => if is_confirm {
                        self.change_state (Context::Idle);

                        match self.action {
                            Action::Attack => {
                                self.attack_unit (unit_id);
                                println! ("Attacking {:?}", self.target_ids);

                                for target_id in &self.target_ids {
                                    println! ("{}: {}", target_id, self.units[*target_id].get_statistics ());
                                }

                                println! ("Self: {}", self.units[unit_id].get_statistics ());

                                let target_ids: Vec<ID> = self.target_ids.drain ( .. ).collect ();

                                for target_id in target_ids {
                                    if !self.units[target_id].is_alive () {
                                        self.kill_unit (target_id);
                                    }
                                }
                            }
                            Action::Skill => {
                                for target_id in &self.target_ids {
                                    println! ("{}", self.units[*target_id]);
                                }

                                self.use_skill_unit (unit_id);
                                println! ("Using skill {} on {:?}", self.skill_magic_id, self.target_ids);

                                for target_id in &self.target_ids {
                                    println! ("{}", self.units[*target_id]);
                                }
                            }
                            Action::Magic => {
                                if self.target_locations.is_empty () {
                                    println! ("{}", self.units[unit_id]);
                                }

                                self.use_magic_unit (unit_id);

                                if self.target_locations.is_empty () {
                                    println! ("Using magic {} on {}", self.skill_magic_id, unit_id);
                                    println! ("{}", self.units[unit_id]);
                                } else {
                                    println! ("Using magic {} on {:?}", self.skill_magic_id, self.target_locations);
                
                                    for target_location in &self.target_locations {
                                        println! ("{:?}: {}", target_location, self.grid.get_tile (target_location));
                                    }
                                }
                            }
                            _ => panic! ("Invalid action {:?}", self.action),
                        }

                        Some (self.action)
                    } else {
                        self.revert_state ();

                        None
                    }
                    Continue ( .. ) => unreachable! (),
                }
            }
            Err (e) => {
                println! ("{}", e);

                None
            }
        }
    }

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

        // print!("\x1B[2J\x1B[1;1H"); // Clears the terminal
        // println! ("Start {}'s turn", unit_id);
        // let _ = self.sender.send (format! ("Start {}'s turn", unit_id));
        // print! ("{}", self.grid);
        // println! ("Movable locations: {:?}", self.grid.find_unit_movable (&unit_id, self.mov));
        // println! ("Turn order: {:?}\n", self.turns);
        // println! ("Actions: Move (q), switch weapon (w), attack (a), skill (s), magic (d), wait (z)");

        let action: Option<Action> = match self.state {
            State::Idle => self.act_idle (input, unit_id),
            State::Move => self.act_move (input, unit_id),
            State::TargetAttack => self.act_target (input, unit_id),
            State::ConfirmAttack => self.act_confirm (input, unit_id),
            State::ChooseSkill => self.act_choose (input, unit_id),
            State::TargetSkill => self.act_target (input, unit_id),
            State::ConfirmSkill => self.act_confirm (input, unit_id),
            State::ChooseMagic => self.act_choose (input, unit_id),
            State::TargetMagic => self.act_target (input, unit_id),
            State::ConfirmMagic => self.act_confirm (input, unit_id),
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
    }
}

#[cfg (test)]
mod tests {
    use super::*;
    use UnitStatistic::{MRL, HLT, SPL, ATK, DEF, MAG, ORG};
    use std::sync::mpsc;

    fn generate_game () -> Game {
        let scene = Scene::default ();
        let (sender, _) = mpsc::channel ();

        Game::new (scene, sender)
    }

    #[test]
    fn game_place_unit () {
        let mut game = generate_game ();

        game.place_unit (0, (1, 0));
        assert_eq! (game.grid.get_unit_location (&0).unwrap (), &(1, 0));
        // -10% ATK from passive, +20% ATK from toggle, +20% ATK from terrain
        assert_eq! (game.units[0].get_statistic (ATK).0, 26);
    }

    #[test]
    fn game_try_spawn_recruit () {
        let mut game = generate_game ();

        game.place_unit (0, (0, 0));

        game.try_spawn_recruit (0);
        assert! (game.factions[0].get_followers (&0).contains (&1));
        assert_eq! (game.units[1].get_leader_id (), 0);
    }

    #[test]
    fn game_send_passive () {
        let mut game = generate_game ();

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
        let mut game = generate_game ();

        game.place_unit (0, (0, 0));
        game.movements = vec![Direction::Right, Direction::Down, Direction::Left];
        assert_eq! (game.move_unit (0), (1, 0));
        assert_eq! (game.grid.get_unit_location (&0).unwrap (), &(1, 0));
        // -10% ATK from passive, +20% ATK from toggle, +20% ATK from terrain
        assert_eq! (game.units[0].get_statistic (ATK).0, 26);
        assert_eq! (game.grid.get_unit_location (&3).unwrap (), &(0, 0));
    }

    #[test]
    fn game_attack_unit () {
        todo!();
        // let mut game = generate_game ();
        // let attribute_9 = *game.scene.get_attribute (&9);
        // let attribute_9 = Box::new (attribute_9);
        // let attribute_10 = *game.scene.get_attribute (&10);
        // let attribute_10 = Box::new (attribute_10);

        // game.units[0].add_appliable (attribute_10);
        // game.units[2].add_appliable (attribute_9);

        // let spl_0_0 = game.units[0].get_statistic (SPL).0;
        // let mrl_2_0 = game.units[2].get_statistic (MRL).0;
        // let hlt_2_0 = game.units[2].get_statistic (HLT).0;
        // let spl_2_0 = game.units[2].get_statistic (SPL).0;
        // assert_eq! (game.act_attack (0, &[2]), 10);
        // let spl_0_1 = game.units[0].get_statistic (SPL).0;
        // let mrl_2_1 = game.units[2].get_statistic (MRL).0;
        // let hlt_2_1 = game.units[2].get_statistic (HLT).0;
        // let spl_2_1 = game.units[2].get_statistic (SPL).0;
        // assert! (spl_0_0 > spl_0_1);
        // assert! (mrl_2_0 > mrl_2_1);
        // assert! (hlt_2_0 > hlt_2_1);
        // assert! (spl_2_0 > spl_2_1);
        // assert_eq! (game.units[0].get_statistic (DEF).0, 18);
        // assert_eq! (game.units[2].get_statistic (MAG).0, 18);
    }

    #[test]
    fn game_use_skill_unit () {
        todo!();
        // let mut game = generate_game ();

        // // Test This skill
        // let spl_3_0 = game.units[3].get_statistic (SPL).0;
        // assert_eq! (game.act_skill (3, &[3], 6), 10);
        // let spl_3_1 = game.units[3].get_statistic (SPL).0;
        // assert! (spl_3_0 > spl_3_1);
        // assert_eq! (game.units[3].get_statistic (DEF).0, 18);
        // // Test Ally skill
        // assert_eq! (game.act_skill (3, &[0], 4), 10);
        // let spl_3_2 = game.units[3].get_statistic (SPL).0;
        // assert! (spl_3_1 > spl_3_2);
        // assert_eq! (game.units[0].get_statistic (DEF).0, 18);
        // // Test Allies skill
        // assert_eq! (game.act_skill (3, &[0, 1], 5), 10);
        // let spl_3_3 = game.units[3].get_statistic (SPL).0;
        // assert! (spl_3_2 > spl_3_3);
        // assert_eq! (game.units[0].get_statistic (DEF).0, 16);
        // assert_eq! (game.units[1].get_statistic (DEF).0, 18);
    }

    #[test]
    fn game_use_magic_unit () {
        todo!();
        // let mut game = generate_game ();

        // game.place_unit (0, (1, 0));

        // // Test This magic
        // let hlt_0_0 = game.units[0].get_statistic (HLT).0;
        // let spl_0_0 = game.units[0].get_statistic (SPL).0;
        // let org_0_0 = game.units[0].get_statistic (ORG).0;
        // assert_eq! (game.act_magic (0, None, 0), 10);
        // let hlt_0_1 = game.units[0].get_statistic (HLT).0;
        // let spl_0_1 = game.units[0].get_statistic (SPL).0;
        // let org_0_1 = game.units[0].get_statistic (ORG).0;
        // assert! (hlt_0_0 > hlt_0_1);
        // assert! (spl_0_0 > spl_0_1);
        // assert! (org_0_0 > org_0_1);
        // assert_eq! (game.units[0].get_statistic (DEF).0, 18);
        // // Test Map magic
        // assert_eq! (game.act_magic (0, Some (&[(1, 0)]), 3), 10);
        // let hlt_0_2 = game.units[0].get_statistic (HLT).0;
        // let spl_0_2 = game.units[0].get_statistic (SPL).0;
        // let org_0_2 = game.units[0].get_statistic (ORG).0;
        // assert! (hlt_0_1 > hlt_0_2);
        // assert! (spl_0_1 > spl_0_2);
        // assert! (org_0_1 > org_0_2);
        // assert! (game.grid.try_yield_appliable (&(1, 0)).is_some ());
        // // -40 HLT from magic, -20 HLT from OnOccupy
        // assert_eq! (game.units[0].get_statistic (HLT).0, 940);
    }

    
    #[test]
    fn game_wait_unit () {
        let mut game = generate_game ();

        let spl_0_0 = game.units[0].get_statistic (SPL).0;
        assert_eq! (game.wait_unit (0), 10);
        let spl_0_1 = game.units[0].get_statistic (SPL).0;
        assert! (spl_0_0 > spl_0_1);
    }

    #[test]
    fn game_kill_unit () {
        let mut game = generate_game ();

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
        let game = generate_game ();

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
        let mut game = generate_game ();

        game.place_unit (0, (0, 0));
        game.place_unit (1, (1, 1));
        game.place_unit (2, (1, 0));

        game.potential_ids = vec![0];
        let results: Vec<ID> = game.find_units_area (0, Target::This, Search::Single);
        assert_eq! (results.len (), 1);
        assert! (results.contains (&0));
        game.potential_ids = vec![1];
        let results: Vec<ID> = game.find_units_area (0, Target::Ally, Search::Path (0, 1, Direction::Length));
        assert_eq! (results.len (), 1);
        assert! (results.contains (&1));
        game.potential_ids = vec![0, 1];
        let results: Vec<ID> = game.find_units_area (0, Target::Allies, Search::Radial (1));
        assert_eq! (results.len (), 1);
        assert! (results.contains (&0));
        game.potential_ids = vec![0, 1];
        let results: Vec<ID> = game.find_units_area (0, Target::Allies, Search::Radial (2));
        assert_eq! (results.len (), 2);
        assert! (results.contains (&0));
        assert! (results.contains (&1));
        game.potential_ids = vec![2];
        let results: Vec<ID> = game.find_units_area (0, Target::Enemy, Search::Radial (1));
        assert_eq! (results.len (), 1);
        assert! (results.contains (&2));
    }

    #[test]
    fn game_find_units_range () {
        let mut game = generate_game ();

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
    fn game_find_locations_area () {
        let mut game = generate_game ();

        game.place_unit (0, (0, 0));
        game.target_location = (0, 0);

        let results: Vec<Location> = game.find_locations_area (Search::Single);
        assert_eq! (results.len (), 1);
        assert! (results.contains (&(0, 0)));
        let results: Vec<Location> = game.find_locations_area (Search::Path (0, 1, Direction::Right));
        assert_eq! (results.len (), 1);
        assert! (results.contains (&(0, 1)));
        let results: Vec<Location> = game.find_locations_area (Search::Radial (1));
        assert_eq! (results.len (), 3);
        assert! (results.contains (&(0, 0)));
        assert! (results.contains (&(0, 1)));
        assert! (results.contains (&(1, 0)));
        game.target_location = (1, 0);
        let results: Vec<Location> = game.find_locations_area (Search::Radial (1));
        assert_eq! (results.len (), 3);
        assert! (results.contains (&(0, 0)));
        assert! (results.contains (&(1, 0)));
        assert! (results.contains (&(1, 1)));
    }

    #[test]
    fn game_find_locations_range () {
        let mut game = generate_game ();

        game.place_unit (0, (0, 0));

        let results: Vec<Location> = game.find_locations_range (0, 0);
        assert_eq! (results.len (), 1);
        assert! (results.contains (&(0, 0)));
        let results: Vec<Location> = game.find_locations_range (0, 1);
        assert_eq! (results.len (), 3);
        assert! (results.contains (&(0, 0)));
        assert! (results.contains (&(0, 1)));
        assert! (results.contains (&(1, 0)));
        let results: Vec<Location> = game.find_locations_range (0, 2);
        assert_eq! (results.len (), 5);
        assert! (results.contains (&(0, 0)));
        assert! (results.contains (&(0, 1)));
        assert! (results.contains (&(0, 2)));
        assert! (results.contains (&(1, 0)));
        assert! (results.contains (&(1, 1)));
    }

    #[test]
    fn game_start_turn () {
        let mut game = generate_game ();
        let modifier_9 = *game.scene.get_modifier (&9);
        let mut modifier_9 = Box::new (modifier_9);
        let modifier_4 = *game.scene.get_modifier (&4);
        let modifier_4 = Box::new (modifier_4);

        game.grid.place_unit (0, (1, 1));
        game.grid.place_unit (1, (0, 0));
        game.movements = vec![Direction::Right];
        modifier_9.set_applier_id (0);

        // Test impassable start
        game.grid.add_appliable (&(1, 2), modifier_9);
        game.move_unit (0);
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
    fn game_act () {
        todo!();
        // let input = b"z\nd\ns\na\nz\nz\nw\na\nz\na\n2\nz\n\
        //         z\na\nx\nq\nq\nq\ns\n0\nz\n\
        //         z\na\nz\nd\n0\nz";
        // let mut game = generate_game (&input[..]);

        // game.place_unit (0, (0, 0));
        // game.place_unit (2, (1, 0));

        // // TODO: This test is extremely cursory
        // let spl_0_0 = game.units[0].get_statistic (SPL).0;
        // let mrl_2_0 = game.units[2].get_statistic (MRL).0;
        // let hlt_2_0 = game.units[2].get_statistic (HLT).0;
        // let spl_2_0 = game.units[2].get_statistic (SPL).0;
        // game.act (0);
        // let spl_0_1 = game.units[0].get_statistic (SPL).0;
        // let mrl_2_1 = game.units[2].get_statistic (MRL).0;
        // let hlt_2_1 = game.units[2].get_statistic (HLT).0;
        // let spl_2_1 = game.units[2].get_statistic (SPL).0;
        // assert_eq! (game.grid.get_unit_location (&0), &(0, 0));
        // assert! (spl_0_0 > spl_0_1);
        // assert! (mrl_2_0 > mrl_2_1);
        // assert! (hlt_2_0 > hlt_2_1);
        // assert! (spl_2_0 > spl_2_1);
        // // Test skills and switch weapon
        // game.act (2);
        // assert_eq! (game.units[2].get_weapon ().get_id (), 2);
        // assert! (!game.units[2].get_skill_ids_actionable ().contains (&0));
        // // Test magic
        // game.act (1);
    }

    #[test]
    fn game_end_turn () {
        let mut game = generate_game ();

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
        let mut game = generate_game ();

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
