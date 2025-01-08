use super::{ActionValidator, ConfirmationValidator, DirectionValidator, LocationValidator, Reader, UnitValidator};
use crate::character::{Faction, FactionBuilder, Magic, Skill, Unit, UnitBuilder, UnitStatistic, UnitStatistics, Weapon};
use crate::common::{Target, Turn, ID, MULTIPLIER_ATTACK, MULTIPLIER_MAGIC, MULTIPLIER_SKILL, MULTIPLIER_WAIT};
use crate::dynamic::{Appliable, Changeable, Status};
use crate::event::Handler;
use crate::map::{Area, Direction, Grid, Location, Search};
use crate::Scene;
use std::cell::RefCell;
use std::collections::{BinaryHeap, HashSet};
use std::io::BufRead;
use std::rc::Rc;

/*
 * Calculated from build.rs
 * Unit MOV is an index into the table
 * Attack (* 1.0): 21 delay at 0, 20 delay at 1, 2 delay at 77, and 1 delay at 100
 * Skill/Magic (* 1.4): 29 delay at 0, 28 delay at 1, 2 delay at 77, and 1 delay at 100
 * Wait (* 0.67): 14 delay at 0, 13 delay at 1, 2 delay at 54, and 1 delay at 77
 */
const DELAYS: [u8; 101] = [21, 20, 19, 19, 18, 18, 17, 17, 16, 16, 15, 15, 14, 14, 14, 13, 13, 13, 12, 12, 11, 11, 11, 11, 10, 10, 10, 9, 9, 9, 9, 8, 8, 8, 8, 7, 7, 7, 7, 7, 7, 6, 6, 6, 6, 6, 6, 5, 5, 5, 5, 5, 5, 5, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 1];

fn get_delay (mov: u16, delay_multiplier: f32) -> u8 {
    let delay: f32 = DELAYS[mov as usize] as f32;

    (delay * delay_multiplier) as u8
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
    handler: Rc<RefCell<Handler>>,
    grid: Grid,
    units: Vec<Unit>,
    factions: Vec<Faction>,
    turns: BinaryHeap<Turn>,
    number_turns: usize,
    reader: Reader<R>,
}

impl<R: BufRead> Game<R> {
    pub fn new (scene: Scene, reader: Reader<R>) -> Self {
        let scene: Rc<Scene> = Rc::new (scene);
        let handler: Handler = Handler::new ();
        let handler: RefCell<Handler> = RefCell::new (handler);
        let handler: Rc<RefCell<Handler>> = Rc::new (handler);
        let grid: Grid = Grid::new (Rc::clone (&scene));
        let units: Vec<Unit> = scene.unit_builders_iter ().map (|u: &UnitBuilder|
            u.build (Rc::clone (&scene))
        ).collect ();
        let factions: Vec<Faction> = scene.faction_builders_iter ().map (|f: &FactionBuilder|
            f.build (&units)
        ).collect ();
        let turns: BinaryHeap<Turn> = BinaryHeap::new ();
        let number_turns: usize = 0;
        let reader: Reader<R> = reader;

        Self { scene, handler, grid, units, factions, turns, number_turns, reader }
    }

    fn apply_terrain (&mut self, unit_id: ID, terrain_id: ID, location: Location) {
        let modifier_terrain_id: Option<ID> = self.scene.get_terrain (&terrain_id)
                .get_modifier_id ();
        let appliable: Option<Box<dyn Appliable>> = self.grid.try_yield_appliable (&location);

        self.units[unit_id].change_modifier_terrain (modifier_terrain_id);

        if let Some (a) = appliable {
            self.units[unit_id].add_appliable (a);
        }
    }

    fn place_unit (&mut self, unit_id: ID, location: Location) {
        let terrain_id: ID = self.grid.place_unit (unit_id, location)
                .unwrap_or_else (|| panic! ("Terrain not found for location {:?}", location));

        self.apply_terrain (unit_id, terrain_id, location);
        self.units[unit_id].apply_inactive_skills ();
    }

    fn move_unit (&mut self, unit_id: ID, movements: &[Direction]) -> Location {
        let (location, terrain_id): (Location, ID) = self.grid
                .move_unit (unit_id, movements)
                .unwrap_or_else (|| panic! ("Invalid movements {:?}", movements));

        self.apply_terrain (unit_id, terrain_id, location);

        location
    }

    fn try_spawn_recruit (&mut self, unit_id: ID) {
        let location: Location = *self.grid.get_unit_location (&unit_id);
        let leader_id: ID = self.units[unit_id].get_leader_id ();
        let faction_id: ID = self.scene.get_unit_builder (&unit_id).get_faction_id ();

        if let Some ((r, t)) = self.grid.try_spawn_recruit (location) {
            let modifier_terrain_id: Option<ID> = self.scene.get_terrain (&t)
                    .get_modifier_id ();

            self.factions[faction_id].add_follower (r, leader_id);
            self.units[r].set_leader (unit_id);
            self.units[r].change_modifier_terrain (modifier_terrain_id);
            // self.units[r].apply_inactive_skills ();
        }
    }

    fn send_passive (&mut self, unit_id: ID) {
        let leader_id: ID = self.units[unit_id].get_leader_id ();
        let faction_id: ID = self.scene.get_unit_builder (&unit_id).get_faction_id ();
        let follower_ids: &HashSet<ID> = self.factions[faction_id].get_followers (&leader_id);
        let skill_passive_id: ID = self.units[leader_id].get_skill_passive_id ()
                .unwrap_or_else (|| panic! ("Passive not found for leader {}", leader_id));
        let status_passive_id: ID = self.scene.get_skill (&skill_passive_id)
                .get_status_id ();

        for follower_id in follower_ids {
            if *follower_id != unit_id {
                let distance: usize = self.grid.find_distance_between (follower_id, &leader_id);

                self.units[*follower_id].try_add_passive (&status_passive_id, distance);
            }
        }
    }

    fn kill_unit (&mut self, unit_id: ID) {
        // TODO: If player leader died, then end game and don't worry about all this
        let faction_id: ID = self.scene.get_unit_builder (&unit_id).get_faction_id ();
        let mut others: Vec<Turn> = Vec::new ();

        self.factions[faction_id].remove_follower (&unit_id);

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
        }).collect::<Vec<ID>> ()
    }

    fn choose_targets_units (&mut self, unit_id: ID, potential_ids: &[ID], target: Target, search: Search) -> Vec<ID> {
        assert! (!potential_ids.is_empty ());

        let target_ids: Vec<ID> = match target {
            Target::This => vec![potential_ids[0]],
            Target::Ally | Target::Enemy => {
                println! ("Potential targets: {:?}", potential_ids);

                let validator: UnitValidator = UnitValidator::new (potential_ids);

                vec![self.reader.read_validate (&validator).expect ("Invalid input")]
            }
            Target::Allies | Target::Enemies => {
                match search {
                    Search::Single => panic! ("Invalid search {:?} for target {:?}", search, target),
                    Search::Radial (r) => {
                        println! ("Potential targets: {:?}", potential_ids);

                        let validator: UnitValidator = UnitValidator::new (potential_ids);
                        let target_id: ID = self.reader.read_validate (&validator).expect ("Invalid input");
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
                    Search::Path (w, r, _) => {
                        println! ("Potential targets: {:?}", potential_ids);

                        let validator: DirectionValidator = DirectionValidator::new ();
                        let direction: Direction = self.reader.read_validate (&validator).expect ("Invalid input");
                        let location: &Location = self.grid.get_unit_location (&unit_id);
                        let target_ids: Vec<ID> = self.grid.find_units (location, Search::Path (w, r, direction));
                        let faction_id: ID = self.units[unit_id].get_faction_id ();

                        if let Target::Allies = target {
                            self.filter_unit_allegiance (&target_ids, faction_id, true)
                        } else if let Target::Enemies = target {
                            self.filter_unit_allegiance (&target_ids, faction_id, false)
                        } else {
                            panic! ("Invalid target {:?}", target)
                        }
                    }
                }
            }
            _ => panic! ("Invalid target {:?}", target),
        };

        println! ("Targets: {:?}", target_ids);

        let validator: ConfirmationValidator = ConfirmationValidator::new ();
        let confirmation: bool = self.reader.read_validate (&validator)
                .expect ("Invalid input");

        if confirmation {
            target_ids
        } else {
            Vec::new ()
        }
    }

    fn find_targets_units (&mut self, unit_id: ID, target: Target, area: Area, range: u8) -> Vec<ID> {
        let potential_ids: Vec<ID> = if let Target::Map = target {
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
        };
        let search: Search = match area {
            Area::Single => Search::Single,
            Area::Radial (r) => Search::Radial (r),
            Area::Path (w) => Search::Path (w, range, Direction::Length), // Placeholder direction
        };

        if potential_ids.is_empty () {
            // TODO: if there are no potential targets, then just give up but wait for the user to cancel
            println! ("No valid targets");

            Vec::new ()
        } else {
            self.choose_targets_units (unit_id, &potential_ids, target, search)
        }
    }

    fn choose_targets_locations (&mut self, unit_id: ID, potential_locations: &[Location], search: Search) -> Vec<Location> {
        assert! (!potential_locations.is_empty ());

        let target_locations: Vec<Location> = match search {
            Search::Single => {
                println! ("Potential targets: {:?}", potential_locations);

                let validator: LocationValidator = LocationValidator::new (potential_locations);

                vec![self.reader.read_validate (&validator).expect ("Invalid input")]
            }
            Search::Radial (r) => {
                println! ("Potential targets: {:?}", potential_locations);

                let validator: LocationValidator = LocationValidator::new (potential_locations);
                let target_location: Location = self.reader.read_validate (&validator).expect ("Invalid input");

                self.grid.find_locations (&target_location, Search::Radial (r))
            }
            Search::Path (w, r, _) => {
                println! ("Potential targets: {:?}", potential_locations);

                let validator: DirectionValidator = DirectionValidator::new ();
                let direction: Direction = self.reader.read_validate (&validator).expect ("Invalid input");
                let location: &Location = self.grid.get_unit_location (&unit_id);

                self.grid.find_locations (location, Search::Path (w, r, direction))
            }
        };

        println! ("Targets: {:?}", target_locations);

        let validator: ConfirmationValidator = ConfirmationValidator::new ();
        let confirmation: bool = self.reader.read_validate (&validator)
                .expect ("Invalid input");

        if confirmation {
            target_locations
        } else {
            Vec::new ()
        }
    }

    fn find_targets_locations (&mut self, unit_id: ID, area: Area, range: u8) -> Vec<Location> {
        let location: &Location = self.grid.get_unit_location (&unit_id);
        let potential_locations: Vec<Location> = if let Area::Path (w) = area {
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
        };
        let search: Search = match area {
            Area::Single => Search::Single,
            Area::Radial (r) => Search::Radial (r),
            Area::Path (w) => Search::Path (w, range, Direction::Length), // Placeholder direction
        };

        self.choose_targets_locations (unit_id, &potential_locations, search)
    }

    fn start_turn (&mut self, unit_id: ID) {
        let location: &Location = self.grid.get_unit_location (&unit_id);

        if !self.grid.is_impassable (location) {
            self.units[unit_id].start_turn ();
        }
    }

    fn act_attack (&mut self, attacker_id: ID, defender_id: ID) -> u16 {
        let statistics_attacker: &UnitStatistics = &self.units[attacker_id]
                .get_statistics ();
        let statistics_defender: &UnitStatistics = &self.units[defender_id]
                .get_statistics ();
        let weapon: &Weapon = self.units[attacker_id].get_weapon ();
        let (damage_mrl, damage_hlt, damage_spl): (u16, u16, u16) = UnitStatistics::calculate_damage (statistics_attacker, statistics_defender, weapon);
        let (mov, appliable_on_attack): (u16, Option<Box<dyn Appliable>>) = self.units[attacker_id].act_attack ();
        let appliable_on_hit: Option<Box<dyn Appliable>> = self.units[defender_id].take_damage (damage_mrl, damage_hlt, damage_spl);

        if let Some (a) = appliable_on_attack {
            self.units[defender_id].add_appliable (a);
        }

        if let Some (a) = appliable_on_hit {
            self.units[attacker_id].add_appliable (a);
        }

        mov
    }

    fn act_skill (&mut self, user_id: ID, target_id: ID, skill_id: ID) -> u16 {
        let (mov, status_skill): (u16, Status) = {
            let (mov, skill): (u16, &Skill) = self.units[user_id].act_skill (&skill_id);
            let status_skill_id: ID = skill.get_status_id ();
            let status_skill: Status = *self.scene.get_status (&status_skill_id);

            (mov, status_skill)
        };

        self.units[target_id]
                .add_status (status_skill);

        mov
    }

    fn act_magic (&mut self, user_id: ID, target: Option<Location>, magic_id: ID) -> u16 {
        let (mov, status_magic): (u16, Status) = {
            let (mov, magic): (u16, &Magic) = self.units[user_id].act_magic (&magic_id);
            let status_magic_id: ID = magic.get_status_id ();
            let status_magic: Status = *self.scene.get_status (&status_magic_id);

            (mov, status_magic)
        };

        match target {
            Some (l) => {
                self.grid.add_status (&l, status_magic);
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
        let validator: ActionValidator = ActionValidator::new ();

        let (mov, delay_multiplier): (u16, f32) = loop {
            if let Some (action) = self.reader.read_validate (&validator) {
                match action {
                    Action::Attack => if is_retreat {
                        println! ("Unit cannot attack (is retreating)")
                    } else {
                        todo! ();

                        break (self.act_wait (unit_id), MULTIPLIER_ATTACK)
                    }
                    Action::Weapon => if is_rout {
                        println! ("Unit cannot rearm (is routed)")
                    } else {
                        self.units[unit_id].switch_weapon ();
                    }
                    Action::Skill => if is_rout {
                        println! ("Unit cannot use skill (is routed)")
                    } else {
                        todo! ();

                        break (self.act_wait (unit_id), MULTIPLIER_SKILL)
                    }
                    Action::Magic => if is_rout {
                        println! ("Unit cannot use magic (is routed)")
                    } else {
                        todo! ();

                        break (self.act_wait (unit_id), MULTIPLIER_MAGIC)
                    }
                    Action::Move => {
                        let mut start: Location = *self.grid.get_unit_location (&unit_id);
                        let mut mov: u16 = self.units[unit_id].get_statistic (UnitStatistic::MOV).0;
                        let mut movements: Vec<Direction> = Vec::new ();
                        let validator: DirectionValidator = DirectionValidator::new ();
                
                        while let Some (direction) = self.reader.read_validate (&validator) {
                            if let Some ((end, cost)) = self.grid.try_move (&start, Direction::Up) {
                                if mov >= (cost as u16) {
                                    movements.push (direction);
                                    start = end;
                                    mov -= cost as u16;
                                }
                            } else {
                                break
                            }
                        }

                        self.move_unit (unit_id, &movements);
                    }
                    Action::Wait => break (self.act_wait (unit_id), MULTIPLIER_WAIT)
                }
            }
        };

        (mov, delay_multiplier)
    }

    fn end_turn (&mut self, unit_id: ID) {
        let faction_id: ID = self.scene.get_unit_builder (&unit_id).get_faction_id ();
        let city_ids: Vec<ID> = self.grid.find_unit_cities (&unit_id, &faction_id);
        let location: Location = *self.grid.get_unit_location (&unit_id);
        let appliable: Option<Box<dyn Appliable>> = self.grid.try_yield_appliable (&location);

        self.units[unit_id].end_turn (&city_ids, appliable);
        self.grid.expand_control (&faction_id);
    }

    fn do_turn (&mut self, unit_id: ID) {
        let mut turn: Turn = self.turns.pop ()
                .expect ("Turn not found");

        self.number_turns += 1;
        self.start_turn (unit_id);

        let (mov, delay_multiplier): (u16, f32) = if self.units[unit_id].is_alive () {
            self.act (unit_id)
        } else {
            (u16::MAX, f32::MAX)
        };
        let delay: u8 = if self.units[unit_id].is_alive () {
            self.end_turn (unit_id);

            get_delay (mov, delay_multiplier)
        } else {
            u8::MAX
        };

        if self.units[unit_id].is_alive () {
            if turn.update (delay, mov) {
                self.turns.push (turn);
            } else {
                let reduction: u8 = self.turns.peek ()
                        .expect ("Turn not found")
                        .get_delay ();
                let turns: Vec<Turn> = self.turns.drain ().collect ();
    
                for mut turn in turns {
                    turn.reduce_delay (reduction);
                    self.turns.push (turn);
                }
            }
        } else {
            self.kill_unit (unit_id);
        }
    }
}

#[cfg (test)]
mod tests {
    use super::*;
    use UnitStatistic::{MRL, HLT, SPL, ATK, DEF, MAG, ORG};

    fn generate_game<R: BufRead> (stream: R) -> Game<R> {
        let scene = Scene::debug ();
        let reader = Reader::new (stream);

        Game::new (scene, reader)
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
    fn game_move_unit () {
        let mut game = generate_game (&b""[..]);

        game.place_unit (0, (0, 0));
        assert_eq! (game.move_unit (0, &[Direction::Right, Direction::Down, Direction::Left]), (1, 0));
        assert_eq! (game.grid.get_unit_location (&0), &(1, 0));
        // -10% ATK from passive, +20% ATK from toggle, +20% ATK from terrain
        assert_eq! (game.units[0].get_statistic (ATK).0, 26);
        // check moved, modifier, appliable
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
        game.send_passive (0);

        // Test near send
        // -10% ATK from passive
        assert_eq! (game.units[1].get_statistic (ATK).0, 18);
        // Test far send
        game.move_unit (1, &[Direction::Down]);
        game.send_passive (0);
        // +20% ATK from terrain
        assert_eq! (game.units[1].get_statistic (ATK).0, 24);
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

        let response = game.filter_unit_allegiance (&[0, 1, 2], 0,true);
        assert_eq! (response.len (), 2);
        assert! (response.contains (&0));
        assert! (response.contains (&1));
        let response = game.filter_unit_allegiance (&[0, 1, 2], 0, false);
        assert_eq! (response.len (), 1);
        assert! (response.contains (&2));
    }

    #[test]
    fn game_choose_targets_units () {
        let mut game = generate_game (&b"z\n1\nz\n0\nz\n0\nz\n2\nz"[..]);

        game.place_unit (0, (0, 0));
        game.place_unit (1, (1, 1));
        game.place_unit (2, (1, 0));

        let results: Vec<ID> = game.choose_targets_units (0, &[0], Target::This, Search::Single);
        assert_eq! (results.len (), 1);
        assert! (results.contains (&0));
        let results: Vec<ID> = game.choose_targets_units (0, &[1], Target::Ally, Search::Path (0, 1, Direction::Length));
        assert_eq! (results.len (), 1);
        assert! (results.contains (&1));
        let results: Vec<ID> = game.choose_targets_units (0, &[0, 1], Target::Allies, Search::Radial (1));
        assert_eq! (results.len (), 1);
        assert! (results.contains (&0));
        let results: Vec<ID> = game.choose_targets_units (0, &[0, 1], Target::Allies, Search::Radial (2));
        assert_eq! (results.len (), 2);
        assert! (results.contains (&0));
        assert! (results.contains (&1));
        let results: Vec<ID> = game.choose_targets_units (0, &[2], Target::Enemy, Search::Radial (1));
        assert_eq! (results.len (), 1);
        assert! (results.contains (&2));
    }

    #[test]
    fn game_find_targets_units () {
        let mut game = generate_game (&b"z\n0\nz\n0\nz\n0\nz\n2\nz\nr\nz\nr\nz"[..]);

        game.place_unit (0, (0, 0));
        game.place_unit (1, (1, 1));
        game.place_unit (2, (1, 0));

        let results: Vec<ID> = game.find_targets_units (0, Target::This, Area::Single, 1);
        assert_eq! (results.len (), 1);
        assert! (results.contains (&0));
        let results: Vec<ID> = game.find_targets_units (0, Target::Ally, Area::Single, 1);
        assert_eq! (results.len (), 1);
        assert! (results.contains (&0));
        let results: Vec<ID> = game.find_targets_units (0, Target::Allies, Area::Radial (1), 0);
        assert_eq! (results.len (), 1);
        assert! (results.contains (&0));
        let results: Vec<ID> = game.find_targets_units (0, Target::Allies, Area::Radial (2), 0);
        assert_eq! (results.len (), 2);
        assert! (results.contains (&0));
        assert! (results.contains (&1));
        let results: Vec<ID> = game.find_targets_units (0, Target::Enemy, Area::Radial (0), 1);
        assert_eq! (results.len (), 1);
        assert! (results.contains (&2));
        let results: Vec<ID> = game.find_targets_units (2, Target::Enemies, Area::Path (0), 1);
        assert_eq! (results.len (), 1);
        assert! (results.contains (&1));
        let results: Vec<ID> = game.find_targets_units (0, Target::Enemies, Area::Path (0), 1); // Test empty find
        assert! (results.is_empty ());
    }

    #[test]
    fn game_choose_targets_locations () {
        let mut game = generate_game (&b"0, 0\nz\nr\nz\n0, 0\nz\n1, 0\nz"[..]);

        game.place_unit (0, (0, 0));

        let results: Vec<Location> = game.choose_targets_locations (0, &[(0, 0)], Search::Single);
        assert_eq! (results.len (), 1);
        assert! (results.contains (&(0, 0)));
        let results: Vec<Location> = game.choose_targets_locations (0, &[(0, 0), (0, 1)], Search::Path (0, 1, Direction::Length));
        assert_eq! (results.len (), 1);
        assert! (results.contains (&(0, 1)));
        let results: Vec<Location> = game.choose_targets_locations (0, &[(0, 0), (0, 1), (1, 0)], Search::Radial (1));
        assert_eq! (results.len (), 3);
        assert! (results.contains (&(0, 0)));
        assert! (results.contains (&(0, 1)));
        assert! (results.contains (&(1, 0)));
        let results: Vec<Location> = game.choose_targets_locations (0, &[(1, 0), (0, 1), (0, 0)], Search::Radial (1));
        assert_eq! (results.len (), 3);
        assert! (results.contains (&(1, 0)));
        assert! (results.contains (&(0, 0)));
        assert! (results.contains (&(1, 1)));
    }

    #[test]
    fn game_find_targets_locations () {
        let mut game = generate_game (&b"0, 1\nzr\nz\nz\nr\nz\nr\nz\nr\nz\nr\nz\n0, 0\nz\n1, 0\nz\n0, 1\nz"[..]);

        game.place_unit (0, (0, 0));

        let results: Vec<Location> = game.find_targets_locations (0, Area::Single, 1);
        assert_eq! (results.len (), 1);
        assert! (results.contains (&(0, 1)));
        let results: Vec<Location> = game.find_targets_locations (0, Area::Path (0), 1);
        assert_eq! (results.len (), 1);
        assert! (results.contains (&(0, 1)));
        let results: Vec<Location> = game.find_targets_locations (0, Area::Path (0), 2);
        assert_eq! (results.len (), 2);
        assert! (results.contains (&(0, 1)));
        assert! (results.contains (&(0, 2)));
        let results: Vec<Location> = game.find_targets_locations (0, Area::Path (1), 1);
        assert_eq! (results.len (), 2);
        assert! (results.contains (&(0, 1)));
        assert! (results.contains (&(1, 1)));
        let results: Vec<Location> = game.find_targets_locations (0, Area::Path (2), 2);
        assert_eq! (results.len (), 4);
        assert! (results.contains (&(0, 1)));
        assert! (results.contains (&(0, 2)));
        assert! (results.contains (&(1, 1)));
        assert! (results.contains (&(1, 2)));
        let results: Vec<Location> = game.find_targets_locations (0, Area::Radial (1), 1);
        assert_eq! (results.len (), 3);
        assert! (results.contains (&(0, 0)));
        assert! (results.contains (&(0, 1)));
        assert! (results.contains (&(1, 0)));
        let results: Vec<Location> = game.find_targets_locations (0, Area::Radial (1), 1);
        assert_eq! (results.len (), 3);
        assert! (results.contains (&(1, 0)));
        assert! (results.contains (&(0, 0)));
        assert! (results.contains (&(1, 1)));
        let results: Vec<Location> = game.find_targets_locations (0, Area::Radial (1), 1);
        assert_eq! (results.len (), 4);
        assert! (results.contains (&(0, 1)));
        assert! (results.contains (&(0, 0)));
        assert! (results.contains (&(1, 1)));
        assert! (results.contains (&(0, 2)));
    }

    // #[test]
    // fn game_start_turn () {
    //     todo! ("no functionality")
    // }

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
        assert_eq! (game.act_attack (0, 2), 10);
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
        assert_eq! (game.act_skill (3, 3, 6), 10);
        let spl_3_1 = game.units[3].get_statistic (SPL).0;
        assert! (spl_3_0 > spl_3_1);
        assert_eq! (game.units[3].get_statistic (DEF).0, 18);
        // Test Ally skill
        assert_eq! (game.act_skill (3, 0, 4), 10);
        let spl_3_2 = game.units[3].get_statistic (SPL).0;
        assert! (spl_3_1 > spl_3_2);
        assert_eq! (game.units[0].get_statistic (DEF).0, 18);
        // Test Allies skill
        assert_eq! (game.act_skill (3, 0, 5), 10);
        assert_eq! (game.act_skill (3, 1, 5), 10);
        let spl_3_3 = game.units[3].get_statistic (SPL).0;
        assert! (spl_3_2 > spl_3_3);
        assert_eq! (game.units[0].get_statistic (DEF).0, 16);
        assert_eq! (game.units[1].get_statistic (DEF).0, 18);
        // Test toggled skill
        let spl_0_0 = game.units[0].get_statistic (SPL).0;
        assert_eq! (game.act_skill (0, 0, 2), 10);
        let spl_0_1 = game.units[0].get_statistic (SPL).0;
        assert! (spl_0_0 > spl_0_1);
        assert_eq! (game.units[0].get_statistic (ATK).0, 22);
        assert_eq! (game.act_skill (0, 0, 2), 10);
        let spl_0_2 = game.units[0].get_statistic (SPL).0;
        assert! (spl_0_1 > spl_0_2);
        assert_eq! (game.units[0].get_statistic (ATK).0, 28);
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
        assert_eq! (game.act_magic (0, Some ((1, 0)), 3), 10);
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
        let input = b"w\nr\nd\nl\nu";
        let mut game = generate_game (&input[..]);

        game.place_unit(0, (0, 0));
        // game.act (0);
        // todo! ("further I/O testing")
    }

    // #[test]
    // fn game_end_turn () {
        // todo! ("no functionality")
    // }

    // #[test]
    // fn game_do_turn () {
        // let mut game = generate_game (&b""[..]);

        // todo! ("no functionality")
    // }
}
