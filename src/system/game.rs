use crate::common::{Turn, ID, MULTIPLIER_ATTACK, MULTIPLIER_MAGIC, MULTIPLIER_SKILL, MULTIPLIER_WAIT};
use crate::character::{Faction, Magic, Skill, Unit, UnitStatistic, UnitStatistics, Weapon};
use crate::dynamic::{Appliable, Applier, Changeable, ModifierBuilder, Status};
use crate::event::Handler;
use crate::map::{Direction, Grid, Location};
use crate::Lists;
use std::cell::RefCell;
use std::collections::{BinaryHeap, HashSet};
use std::io::Read;
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

#[derive (Debug)]
pub struct Game<R: Read> {
    lists: Rc<Lists>,
    handler: Rc<RefCell<Handler>>,
    grid: Grid,
    factions: Vec<Faction>,
    units: Vec<Unit>,
    turns: BinaryHeap<Turn>,
    number_turns: usize,
    reader: R,
}

impl<R: Read> Game<R> {
    pub fn new (lists: Lists, grid: Grid, reader: R) -> Self {
        let lists: Rc<Lists> = Rc::new (lists);
        let handler: Handler = Handler::new ();
        let handler: RefCell<Handler> = RefCell::new (handler);
        let handler: Rc<RefCell<Handler>> = Rc::new (handler);
        let mut factions: Vec<Faction> = Vec::new ();
        let mut units: Vec<Unit> = Vec::new ();
        let turns: BinaryHeap<Turn> = BinaryHeap::new ();
        let number_turns: usize = 0;
        let reader: R = reader;

        for faction_builder in lists.faction_builders_iter () {
            let faction: Faction = faction_builder.build ();

            factions.push (faction);
        }

        for unit_builder in lists.unit_builders_iter () {
            let unit: Unit = unit_builder.build (Rc::clone (&lists));
            let unit_id: ID = unit.get_id ();
            let faction_id: ID = unit.get_faction_id ();
            let leader_id: ID = unit.get_leader_id ();

            units.push (unit);
            factions[faction_id].add_member (unit_id);
            factions[faction_id].add_follower (unit_id, leader_id);
        }

        Self { lists, handler, grid, factions, units, turns, number_turns, reader }
    }

    fn read_char (&mut self) -> char {
        let mut input: [u8; 1] = [b'0'];
    
        self.reader.read_exact (&mut input).unwrap_or_else (|e| panic! ("{:?}", e));

        char::from (input[0])
    }

    fn read_fixed (&mut self, prompt: &str, inputs_valid: &[char]) -> char {
        print! ("{}: ", prompt);
    
        let mut input: char = self.read_char ();
    
        while !inputs_valid.contains (&input) {
            input = self.read_char ();
            println! ("{}", input);
        }
    
        input
    }

    fn read_input<F, T> (&mut self, prompt: &str, mut validator: F) -> T
    where F: FnMut (u8) -> Option<T> {
        print! ("{}: ", prompt);

        let mut input: u8 = self.read_char () as u8;
        let mut result: Option<T> = validator (input);

        while result.is_none () {
            input = self.read_char () as u8;
            result = validator (input);
        }

        result.unwrap ()
    }

    fn apply_terrain (&mut self, unit_id: ID, terrain_id: ID, location: Location) {
        let modifier_terrain_id: Option<ID> = self.lists.get_terrain (&terrain_id)
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
        let faction_id: ID = self.lists.get_unit_builder (&unit_id).get_faction_id ();

        if let Some ((r, t)) = self.grid.try_spawn_recruit (location) {
            let modifier_terrain_id: Option<ID> = self.lists.get_terrain (&t)
                    .get_modifier_id ();

            self.factions[faction_id].add_follower (r, leader_id);
            self.units[r].set_leader (unit_id);
            self.units[r].change_modifier_terrain (modifier_terrain_id);
            // self.units[r].apply_inactive_skills ();
        }
    }

    fn send_passive (&mut self, unit_id: ID) {
        let leader_id: ID = self.units[unit_id].get_leader_id ();
        let faction_id: ID = self.lists.get_unit_builder (&unit_id).get_faction_id ();
        let follower_ids: &HashSet<ID> = self.factions[faction_id].get_followers (&leader_id);
        let skill_passive_id: ID = self.units[leader_id].get_skill_passive_id ()
                .unwrap_or_else (|| panic! ("Passive not found for leader {}", leader_id));
        let status_passive_id: ID = self.lists.get_skill (&skill_passive_id)
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
        let faction_id: ID = self.lists.get_unit_builder (&unit_id).get_faction_id ();
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

        // fn filter_unit_allegiance (&self, unit_ids: &Vec<ID>, is_ally: bool) -> Vec<ID> {
    //     if is_ally {
    //         unit_ids.iter ().filter_map (|u: &ID| {
    //             let is_member: Vec<Response> = self.notify (Message::FactionIsMember (self.faction_id, *u));
    //             if let Response::FactionIsMember (m) = Handler::reduce_responses (&is_member) {
    //                 if *m {
    //                     Some (*u)
    //                 } else {
    //                     None
    //                 }
    //             } else {
    //                 panic! ("Invalid response")
    //             }
    //         }).collect::<Vec<ID>> ()
    //     } else {
    //         unit_ids.iter ().filter_map (|u: &ID| {
    //             let is_member: Vec<Response> = self.notify (Message::FactionIsMember (self.faction_id, *u));

    //             if let Response::FactionIsMember (m) = Handler::reduce_responses (&is_member) {
    //                 if !(*m) {
    //                     Some (*u)
    //                 } else {
    //                     None
    //                 }
    //             } else {
    //                 panic! ("Invalid response")
    //             }
    //         }).collect::<Vec<ID>> ()
    //     }
    // }

    // fn choose_targets_units (&self, potential_ids: &Vec<ID>, target: TargetType, area: Area, range: u8) -> Vec<ID> {
    //     assert! (potential_ids.len () > 0);

    //     let mut target_id: ID = potential_ids[0];
    //     let target_ids: Vec<ID> = match target {
    //         TargetType::This => {
    //             // TODO: Prompt user for confirmation
    //             if true {
    //                 vec![target_id]
    //             // TODO: User rejected choice
    //             } else {
    //                 Vec::new ()
    //             }
    //         }
    //         TargetType::Ally | TargetType::Enemy => {
    //             loop {
    //                 // TODO: Prompt user to choose ONE
    //                 // target_id = ???;

    //                 // TODO: Prompt user for confirmation
    //                 if true {
    //                     break vec![target_id]
    //                 // TODO: User rejected choice
    //                 } else if 1 > 0 {
    //                     break Vec::new ()
    //                 }
    //                 // TODO: else -> user made another choice
    //             }
    //         }
    //         TargetType::Allies | TargetType::Enemies => {
    //             match area {
    //                 Area::Single => potential_ids.clone (),
    //                 Area::Radial (r) => loop {
    //                     // TODO:: Prompt user to choose ONE centre
    //                     // target_id = ???;

    //                     let target_ids: Vec<Response> = self.notify (Message::GridFindUnits (target_id, Search::Radial (r)));
    //                     let target_ids: Vec<ID> = if let Response::GridFindUnits (t) = Handler::reduce_responses (&target_ids) {
    //                         if let TargetType::Allies = target {
    //                             self.filter_unit_allegiance (t, true)
    //                         } else if let TargetType::Enemies = target {
    //                             self.filter_unit_allegiance (t, false)
    //                         } else {
    //                             panic! ("Invalid target {:?}", target)
    //                         }
    //                     } else {
    //                         panic! ("Invalid response")
    //                     };

    //                     // TODO: Prompt user for confirmation
    //                     if true {
    //                         break target_ids.clone ()
    //                     // TODO: User rejected choice
    //                     } else if 1 > 0 {
    //                         break Vec::new ()
    //                     }
    //                     // TODO: else -> user made another choice
    //                 }
    //                 Area::Path (w) => loop {
    //                     // TODO:: Prompt user to choose ONE direction
    //                     // direction = ???
    //                     let direction = Direction::Right;
    //                     let target_ids: Vec<Response> = self.notify (Message::GridFindUnits (self.id, Search::Path (w, range, direction)));
    //                     let target_ids: Vec<ID> = if let Response::GridFindUnits (t) = Handler::reduce_responses (&target_ids) {
    //                         if let TargetType::Allies = target {
    //                             self.filter_unit_allegiance (t, true)
    //                         } else if let TargetType::Enemies = target {
    //                             self.filter_unit_allegiance (t, false)
    //                         } else {
    //                             panic! ("Invalid target {:?}", target)
    //                         }
    //                     } else {
    //                         panic! ("Invalid response")
    //                     };

    //                     // TODO: Prompt user for confirmation
    //                     if true {
    //                         break target_ids.clone ()
    //                     // TODO: User rejected choice
    //                     } else if 1 > 0 {
    //                         break Vec::new ()
    //                     }
    //                     // TODO: else -> user made another choice
    //                 }
    //             }
    //         }
    //         _ => panic! ("Invalid target {:?}", target),
    //     };

    //     // TODO: Prompt user to confirm
    //     target_ids
    // }

    // fn find_targets_units (&self, target: TargetType, area: Area, range: u8) -> Vec<ID> {
    //     let potential_ids: Vec<ID> = if let TargetType::Map = target {
    //         panic! ("Invalid target {:?}", target)
    //     } else if let TargetType::This = target {
    //         vec![self.id]
    //     } else {
    //         let neighbour_ids: Vec<ID> = if let Area::Path (w) = area {
    //             let neighbour_ids_up: Vec<Response> = self.notify (Message::GridFindUnits (self.id, Search::Path (w, range, Direction::Up)));
    //             let neighbour_ids_up: &Vec<ID> = if let Response::GridFindUnits (n) = Handler::reduce_responses (&neighbour_ids_up) {
    //                 n
    //             } else {
    //                 panic! ("Invalid response")
    //             };
    //             let neighbour_ids_right: Vec<Response> = self.notify (Message::GridFindUnits (self.id, Search::Path (w, range, Direction::Right)));
    //             let neighbour_ids_right: &Vec<ID> = if let Response::GridFindUnits (n) = Handler::reduce_responses (&neighbour_ids_right) {
    //                 n
    //             } else {
    //                 panic! ("Invalid response")
    //             };
    //             let neighbour_ids_left: Vec<Response> = self.notify (Message::GridFindUnits (self.id, Search::Path (w, range, Direction::Left)));
    //             let neighbour_ids_left: &Vec<ID> = if let Response::GridFindUnits (n) = Handler::reduce_responses (&neighbour_ids_left) {
    //                 n
    //             } else {
    //                 panic! ("Invalid response")
    //             };
    //             let neighbour_ids_down: Vec<Response> = self.notify (Message::GridFindUnits (self.id, Search::Path (w, range, Direction::Down)));
    //             let neighbour_ids_down: &Vec<ID> = if let Response::GridFindUnits (n) = Handler::reduce_responses (&neighbour_ids_down) {
    //                 n
    //             } else {
    //                 panic! ("Invalid response")
    //             };
    //             let mut neighbour_ids: Vec<ID> = Vec::new ();

    //             neighbour_ids.extend (neighbour_ids_up.iter ());
    //             neighbour_ids.extend (neighbour_ids_right.iter ());
    //             neighbour_ids.extend (neighbour_ids_left.iter ());
    //             neighbour_ids.extend (neighbour_ids_down.iter ());

    //             neighbour_ids
    //         } else {
    //             let neighbour_ids: Vec<Response> = self.notify (Message::GridFindUnits (self.id, Search::Radial (range)));

    //             if let Response::GridFindUnits (n) = Handler::reduce_responses (&neighbour_ids) {
    //                 n.clone ()
    //             } else {
    //                 panic! ("Invalid response")
    //             }
    //         };

    //         match target {
    //             TargetType::Ally | TargetType::Allies => self.filter_unit_allegiance (&neighbour_ids, true),
    //             TargetType::Enemy | TargetType::Enemies => self.filter_unit_allegiance (&neighbour_ids, false),
    //             _ => panic! ("Invalid target {:?}", target),
    //         }
    //     };

    //     if potential_ids.len () > 0 {
    //         self.choose_targets_units (&potential_ids, target, area, range)
    //     } else {
    //         // TODO: if there are no potential targets, then just give up but wait for the user to cancel
    //         Vec::new ()
    //     }
    // }

    // fn choose_targets_locations (&self, potential_locations: &Vec<Location>, area: Area, range: u8) -> Vec<Location> {
    //     assert! (potential_locations.len () > 0);

    //     let mut target_location: Location = potential_locations[0];

    //     match area {
    //         Area::Single => potential_locations.clone (),
    //         Area::Radial (r) => loop {
    //             // TODO:: Prompt user to choose ONE centre
    //             // target_location = ???;

    //             let target_locations: Vec<Response> = self.notify (Message::GridFindLocations (target_location, Search::Radial (r)));
    //             let target_locations: &Vec<Location> = if let Response::GridFindLocations (t) = Handler::reduce_responses (&target_locations) {
    //                 t
    //             } else {
    //                 panic! ("Invalid response")
    //             };

    //             // TODO: Prompt user for confirmation
    //             if true {
    //                 break target_locations.clone ()
    //             // TODO: User rejected choice
    //             } else if 1 > 0 {
    //                 break Vec::new ()
    //             }
    //             // TODO: else -> user made another choice
    //         }
    //         Area::Path (w) => loop {
    //             // TODO:: Prompt user to choose ONE direction
    //             // direction = ???;
    //             let location: Vec<Response> = self.notify (Message::GridGetUnitLocation (self.id));
    //             let location: Location = if let Response::GridGetUnitLocation (l) = Handler::reduce_responses (&location) {
    //             *l
    //             } else {
    //                 panic! ("Invalid response")
    //             };
    //             let direction = Direction::Right;
    //             let target_locations: Vec<Response> = self.notify (Message::GridFindLocations (location, Search::Path (w, range, direction)));
    //             let target_locations: &Vec<Location> = if let Response::GridFindLocations (t) = Handler::reduce_responses (&target_locations) {
    //                 t
    //             } else {
    //                 panic! ("Invalid response")
    //             };

    //             // TODO: Prompt user for confirmation
    //             if true {
    //                 break target_locations.clone ()
    //             // TODO: User rejected choice
    //             } else if 1 > 0 {
    //                 break Vec::new ()
    //             }
    //             // TODO: else -> user made another choice
    //         }
    //     }
    // }

    // fn find_targets_locations (&self, area: Area, range: u8) -> Vec<Location> {
    //     let location: Vec<Response> = self.notify (Message::GridGetUnitLocation (self.id));
    //     let location: Location = if let Response::GridGetUnitLocation (l) = Handler::reduce_responses (&location) {
    //         *l
    //     } else {
    //         panic! ("Invalid response")
    //     };
    //     let potential_locations: Vec<Location> = if let Area::Path (w) = area {
    //         let neighbour_locations_up: Vec<Response> = self.notify (Message::GridFindLocations (location, Search::Path (w, range, Direction::Up)));
    //         let neighbour_locations_up: &Vec<Location> = if let Response::GridFindLocations (n) = Handler::reduce_responses (&neighbour_locations_up) {
    //             n
    //         } else {
    //             panic! ("Invalid response")
    //         };
    //         let neighbour_locations_right: Vec<Response> = self.notify (Message::GridFindLocations (location, Search::Path (w, range, Direction::Right)));
    //         let neighbour_locations_right: &Vec<Location> = if let Response::GridFindLocations (n) = Handler::reduce_responses (&neighbour_locations_right) {
    //             n
    //         } else {
    //             panic! ("Invalid response")
    //         };
    //         let neighbour_locations_left: Vec<Response> = self.notify (Message::GridFindLocations (location, Search::Path (w, range, Direction::Left)));
    //         let neighbour_locations_left: &Vec<Location> = if let Response::GridFindLocations (n) = Handler::reduce_responses (&neighbour_locations_left) {
    //             n
    //         } else {
    //             panic! ("Invalid response")
    //         };
    //         let neighbour_locations_down: Vec<Response> = self.notify (Message::GridFindLocations (location, Search::Path (w, range, Direction::Down)));
    //         let neighbour_locations_down: &Vec<Location> = if let Response::GridFindLocations (n) = Handler::reduce_responses (&neighbour_locations_down) {
    //             n
    //         } else {
    //             panic! ("Invalid response")
    //         };
    //         let mut neighbour_locations: Vec<Location> = Vec::new ();

    //         neighbour_locations.extend (neighbour_locations_up.iter ());
    //         neighbour_locations.extend (neighbour_locations_right.iter ());
    //         neighbour_locations.extend (neighbour_locations_left.iter ());
    //         neighbour_locations.extend (neighbour_locations_down.iter ());

    //         neighbour_locations
    //     } else {
    //         let potential_locations: Vec<Response> = self.notify (Message::GridFindLocations (location, Search::Radial (range)));

    //         if let Response::GridFindLocations (n) = Handler::reduce_responses (&potential_locations) {
    //             n.clone ()
    //         } else {
    //             panic! ("Invalid response")
    //         }
    //     };

    //     if potential_locations.len () > 0 {
    //         self.choose_targets_locations (&potential_locations, area, range)
    //     } else {
    //         // TODO: if there are no potential targets, then just give up but wait for the user to cancel
    //         Vec::new ()
    //     }
    // }

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
            let status_skill: Status = *self.lists.get_status (&status_skill_id);

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
            let status_magic: Status = *self.lists.get_status (&status_magic_id);

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
        let location: Location = *self.grid.get_unit_location (&unit_id);
        let mut end: Location = location;
        let mut movements: Vec<Direction> = Vec::new ();
        let mut mov: u16 = self.units[unit_id].get_statistic (UnitStatistic::MOV).0;

        loop {
            match self.read_fixed ("Choose direction (u/r/l/d) or stop (s)", &['u', 'r', 'l', 'd']) {
                'u' => if let Some ((neighbour, cost)) = self.grid.try_move (&end, Direction::Up) {
                    if mov >= (cost as u16) {
                        mov -= cost as u16;
                        movements.push (Direction::Up);
                        end = neighbour;
                    } else {
                        break
                    }
                } else {
                    break
                }
                'r' => if let Some ((neighbour, cost)) = self.grid.try_move (&end, Direction::Right) {
                    if mov >= (cost as u16) {
                        mov -= cost as u16;
                        movements.push (Direction::Right);
                        end = neighbour;
                    } else {
                        break
                    }
                } else {
                    break
                }
                'l' => if let Some ((neighbour, cost)) = self.grid.try_move (&end, Direction::Left) {
                    if mov >= (cost as u16) {
                        mov -= cost as u16;
                        movements.push (Direction::Left);
                        end = neighbour;
                    } else {
                        break
                    }
                } else {
                    break
                }
                'd' => if let Some ((neighbour, cost)) = self.grid.try_move (&end, Direction::Down) {
                    if mov >= (cost as u16) {
                        mov -= cost as u16;
                        movements.push (Direction::Down);
                        end = neighbour;
                    } else {
                        break
                    }
                } else {
                    break
                }
                's' => break,
                _ => println! ("Invalid input"),
            }
        };

        self.move_unit (unit_id, &movements);

        println!("{:?}",self.grid.get_unit_location(&unit_id));
        // loop {
            // TODO: switch wepaon
            // break
        // }

        let is_retreat: bool = self.units[unit_id].is_retreat ();
        let is_rout: bool = self.units[unit_id].is_rout ();
        let mov: u16;
        let delay_multiplier: f32;

        // TODO: Choose between attack, skill, magic, wait (ends turn)
        if !is_retreat {
            // Can use magic or skill
            if !is_rout {
                // Can also attack
            }
        }
        // Can always choose to wait

        todo! ()
    }

    fn end_turn (&mut self, unit_id: ID) {
        let faction_id: ID = self.lists.get_unit_builder (&unit_id).get_faction_id ();
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
    use crate::map::grid::tests::generate_grid;
    use UnitStatistic::{MRL, HLT, SPL, ATK, DEF, MAG, ORG};

    fn generate_game<R: Read> (reader: R) -> Game<R> {
        let grid = generate_grid ();

        Game::new (Lists::debug (), grid, reader)
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

    // #[test]
    // fn game_start_turn () {
    //     todo! ("no functionality")
    // }

    #[test]
    fn game_act_attack () {
        let mut game = generate_game (&b""[..]);
        let status_9 = *game.lists.get_status (&9);
        let status_10 = *game.lists.get_status (&10);

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
        let input = b"rdlu";
        let mut game = generate_game (&input[..]);

        // game.place_unit(0, (0, 0));
        // game.act (0);
        // todo! ("further I/O testing")
    }

    // #[test]
    // fn game_end_turn () {
        // todo! ("no functionality")
    // }

    #[test]
    fn game_do_turn () {
        let mut game = generate_game (&b""[..]);

        assert! (true);
        // No functionality right now
    }
}
