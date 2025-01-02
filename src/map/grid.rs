use std::cell::{Cell, RefCell};
use std::collections::{HashSet, VecDeque};
use std::fmt;
use std::rc::{Rc, Weak};
use crate::Lists;
use crate::common::{ID, ID_UNINITIALISED};
use crate::join_map::{InnerJoinMap, OuterJoinMap};
use crate::event::{Handler, Message, Subject, Observer, Response};
use crate::dynamic::{Appliable, Applier, Change, Changeable, Status, Trigger};
use super::{City, Search, Terrain, Tile, COST_IMPASSABLE};

pub type Location = (usize, usize); // row, column
type Adjacency = [u8; Direction::Length as usize]; // cost, climb

const DIRECTIONS: [Direction; Direction::Length as usize] = [Direction::Up, Direction::Right, Direction::Left, Direction::Down];

const fn switch_direction (direction: Direction) -> Direction {
    DIRECTIONS[(Direction::Length as usize) - (direction as usize) - 1]
}

fn is_rectangular<T> (grid: &Vec<Vec<T>>) -> bool {
    assert! (grid.len () > 0);
    assert! (grid[0].len () > 0);

    grid.iter ().all (|r: &Vec<T>| r.len () == grid[0].len ())
}

fn is_in_bounds<T> (grid: &Vec<Vec<T>>, location: &Location) -> bool {
    assert! (grid.len () > 0);
    assert! (grid[0].len () > 0);

    location.0 < grid.len () && location.1 < grid[0].len ()
}

#[derive (Debug)]
#[derive (Clone, Copy)]
pub enum Direction {
    Up,
    Right,
    Left,
    Down,
    Length,
}

#[derive (Debug)]
pub struct Grid {
    lists: Rc<Lists>,
    tiles: Vec<Vec<Tile>>,
    unit_locations: InnerJoinMap<ID, Location>,
    faction_locations: OuterJoinMap<ID, Location>,
    // Safety guarantee: Grid will never borrow_mut Handler
    handler: Weak<RefCell<Handler>>,
    observer_id: Cell<ID>,
    // Cache
    // Safety guarantee: Only Grid can reference its own adjacencies
    adjacencies: RefCell<Vec<Vec<Adjacency>>>,
    faction_units: OuterJoinMap<ID, ID>,
}

impl Grid {
    pub fn new (lists: Rc<Lists>, tiles: Vec<Vec<Tile>>, handler: Weak<RefCell<Handler>>) -> Self {
        assert! (is_rectangular (&tiles));

        let mut faction_locations: OuterJoinMap<ID, Location> = OuterJoinMap::new ();
        let unit_locations: InnerJoinMap<ID, Location> = InnerJoinMap::new ();
        let observer_id: Cell<ID> = Cell::new (ID_UNINITIALISED);
        let adjacencies: Vec<Vec<Adjacency>> = Grid::build_adjacencies (&tiles);
        let adjacencies: RefCell<Vec<Vec<Adjacency>>> = RefCell::new (adjacencies);
        let faction_units: OuterJoinMap<ID, ID> = OuterJoinMap::new ();

        for i in 0 .. tiles.len () {
            for j in 0 .. tiles[i].len () {
                faction_locations.insert ((ID_UNINITIALISED, (i, j)));
            }
        }

        Self { lists, tiles, unit_locations, faction_locations, handler, observer_id, adjacencies, faction_units }
    }

    fn build_adjacencies (tiles: &Vec<Vec<Tile>>) -> Vec<Vec<Adjacency>> {
        assert! (is_rectangular (tiles));

        let mut adjacencies: Vec<Vec<Adjacency>> = Vec::new ();

        for i in 0 .. tiles.len () {
            adjacencies.push (Vec::new ());

            for j in 0 .. tiles[i].len () {
                let tile: &Tile = &tiles[i][j];
                let up: Option<&Tile> = i.checked_sub (1).map (|i: usize| &tiles[i][j]);
                let right: Option<&Tile> = j.checked_add (1).and_then (|j: usize|
                    if j < tiles[i].len () {
                        Some (&tiles[i][j])
                    } else {
                        None
                    }
                );
                let left: Option<&Tile> = j.checked_sub (1).map (|j: usize| &tiles[i][j]);
                let down: Option<&Tile> = i.checked_add (1).and_then (|i: usize|
                    if i < tiles.len () {
                        Some (&tiles[i][j])
                    } else {
                        None
                    }
                );
                let cost_up: u8 = up.map_or (COST_IMPASSABLE, |u: &Tile| tile.find_cost (u));
                let cost_right: u8 = right.map_or (COST_IMPASSABLE, |r: &Tile| tile.find_cost (r));
                let cost_left: u8 = left.map_or (COST_IMPASSABLE, |l: &Tile| tile.find_cost (l));
                let cost_down: u8 = down.map_or (COST_IMPASSABLE, |d: &Tile| tile.find_cost (d));
                let mut adjacency: Adjacency = [0; Direction::Length as usize];

                adjacency[Direction::Up as usize] = cost_up;
                adjacency[Direction::Right as usize] = cost_right;
                adjacency[Direction::Left as usize] = cost_left;
                adjacency[Direction::Down as usize] = cost_down;
                adjacencies[i].push (adjacency);
            }
        }

        adjacencies
    }

    fn get_unit_faction (&mut self, unit_id: &ID) -> ID {
        match self.faction_units.get_second (unit_id) {
            Some (f) => *f,
            None => {
                let faction_id: Vec<Response> = self.notify (Message::UnitGetFactionId (*unit_id));
                let faction_id: ID = if let Response::UnitGetFactionId (f) = Handler::reduce_responses (&faction_id) {
                    *f
                } else {
                    panic! ("Invalid response")
                };

                self.faction_units.insert ((faction_id, *unit_id));

                faction_id
            }
        }
    }

    pub fn get_cost (&self, location: &Location, direction: Direction) -> u8 {
        assert! (is_rectangular (&self.adjacencies.borrow ()));
        assert! (is_in_bounds (&self.adjacencies.borrow (), location));

        self.adjacencies.borrow ()[location.0][location.1][direction as usize]
    }

    pub fn is_impassable (&self, location: &Location) -> bool {
        assert! (is_rectangular (&self.tiles));
        assert! (is_in_bounds (&self.tiles, location));

        self.tiles[location.0][location.1].is_impassable ()
    }

    pub fn is_occupied (&self, location: &Location) -> bool {
        assert! (is_in_bounds (&self.tiles, location));

        self.unit_locations.get_second (location).is_some ()
    }

    fn is_placeable (&self, location: &Location) -> bool {
        assert! (is_in_bounds (&self.tiles, location));

        !self.is_impassable(location) && !self.is_occupied (location)
    }

    fn find_nearest_placeable (&self, location: &Location) -> Location {
        assert! (is_rectangular (&self.tiles));
        assert! (is_in_bounds (&self.tiles, location));

        let mut is_visited: Vec<Vec<bool>> = vec![vec![false; self.tiles[0].len ()]; self.tiles.len ()];
        let mut locations: VecDeque<Location> = VecDeque::new ();

        locations.push_back (*location);
        is_visited[location.0][location.1] = true;

        while locations.len () > 0 {
            let location: Location = locations.pop_front ()
                    .expect ("Location not found");

            if self.is_placeable (&location) {
                return location
            }

            for direction in DIRECTIONS {
                if let Some (n) = self.try_connect (&location, direction) {
                    if !is_visited[n.0][n.1] {
                        locations.push_back (n);
                        is_visited[n.0][n.1] = true;
                    }
                }
            }
        }

        panic! ("Placeable not found")
    }

    fn find_distance_between (&self, unit_id_first: &ID, unit_id_second: &ID) -> usize {
        let location_first: &Location = self.get_unit_location (unit_id_first)
                .expect (&format! ("Location not found for unit {}", unit_id_first));
        let location_second: &Location = self.get_unit_location (unit_id_second)
                .expect (&format! ("Location not found for unit {}", unit_id_second));

        location_first.0.abs_diff (location_second.0) + location_first.1.abs_diff (location_second.1)
    }

    fn try_spawn_recruit (&mut self, location: &Location, unit_id: &ID) -> () {
        let tile: &Tile = &self.tiles[location.0][location.1];

        if let Some (c) = tile.get_city_id () {
            if !tile.is_recruited () {
                let city: &City = self.lists.get_city (&c);
                let recruit_id: ID = city.get_recruit_id ();

                if recruit_id < ID_UNINITIALISED {
                    let spawn: Location = self.find_nearest_placeable (&location);
                    let faction_id: ID = self.get_unit_faction (unit_id);

                    self.notify (Message::FactionAddMember (faction_id, recruit_id));
                    self.notify (Message::FactionAddFollower (faction_id, recruit_id, *unit_id));
                    self.notify (Message::UnitSetLeader (recruit_id, *unit_id));
                    self.notify (Message::UnitSendPassive (*unit_id));
                    self.place_unit (recruit_id, spawn);
                }
            }
        }
    }

    pub fn try_connect (&self, start: &Location, direction: Direction) -> Option<Location> {
        assert! (is_rectangular (&self.adjacencies.borrow ()));
        assert! (is_in_bounds (&self.adjacencies.borrow (), start));

        let mut end: Location = start.clone ();

        match direction {
            Direction::Up => end.0 = start.0.checked_sub (1)?,
            Direction::Right => end.1 = start.1.checked_add (1)?,
            Direction::Left => end.1 = start.1.checked_sub (1)?,
            Direction::Down => end.0 = start.0.checked_add (1)?,
            _ => panic! ("Invalid direction {:?}", direction),
        }

        if is_in_bounds (&self.adjacencies.borrow (), &end) {
            Some (end)
        } else {
            None
        }
    }

    pub fn try_move (&self, start: &Location, direction: Direction) -> Option<(Location, u8)> {
        assert! (is_rectangular (&self.adjacencies.borrow ()));
        assert! (is_in_bounds (&self.adjacencies.borrow (), start));

        let cost: u8 = self.adjacencies.borrow ()[start.0][start.1][direction as usize];

        if cost > COST_IMPASSABLE {
            let mut end: Location = start.clone ();

            match direction {
                Direction::Up => end.0 = start.0.checked_sub (1)?,
                Direction::Right => end.1 = start.1.checked_add (1)?,
                Direction::Left => end.1 = start.1.checked_sub (1)?,
                Direction::Down => end.0 = start.0.checked_add (1)?,
                _ => panic! ("Invalid direction {:?}", direction),
            }

            if is_in_bounds (&self.adjacencies.borrow (), &end) {
                if self.is_placeable (&end) {
                    Some ((end, cost))
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn update_adjacency (&self, location: &Location) -> () {
        assert! (is_rectangular (&self.tiles));
        assert! (is_in_bounds (&self.tiles, location));
        assert! (is_rectangular (&self.adjacencies.borrow ()));
        assert! (is_in_bounds (&self.adjacencies.borrow (), location));

        let tile: &Tile = &self.tiles[location.0][location.1];

        for direction in DIRECTIONS {
            match self.try_connect (&location, direction) {
                Some (n) => {
                    let neighbour: &Tile = &self.tiles[n.0][n.1];
                    let cost: u8 = neighbour.find_cost (tile);

                    self.adjacencies.borrow_mut ()[n.0][n.1][switch_direction (direction) as usize] = cost;
                }
                None => (),
            }
        }
    }

    pub fn place_unit (&mut self, unit_id: ID, location: Location) -> bool {
        assert! (is_in_bounds (&self.tiles, &location));
        assert! (!self.unit_locations.contains_key_first (&unit_id));

        let faction_id: ID = self.get_unit_faction (&unit_id);
        let terrain_id: ID = self.tiles[location.0][location.1].get_terrain_id ();
        let terrain: Terrain = self.lists.get_terrain (&terrain_id).clone ();

        if self.is_placeable (&location) {
            self.unit_locations.insert ((unit_id, location));
            self.faction_locations.replace (location, faction_id);
            self.notify (Message::UnitChangeModifierTerrain (unit_id, terrain.get_modifier_id ()));

            true
        } else {
            false
        }
    }

    pub fn move_unit (&mut self, unit_id: ID, movements: Vec<Direction>) -> bool {
        let mut locations: Vec<Location> = Vec::new ();
        let faction_id: ID = self.get_unit_faction (&unit_id);
        let start: Location = self.get_unit_location (&unit_id)
                .expect (&format! ("Location not found for unit {}", unit_id)).clone ();
        let mut end: Location = start.clone ();

        // Temporarily remove unit
        self.unit_locations.remove_first (&unit_id);

        for direction in movements {
            end = match self.try_move (&end, direction) {
                Some (e) => e.0.clone (),
                None => {
                    // Restore unit
                    self.unit_locations.insert ((unit_id, start));

                    return false
                }
            };
            locations.push (end);
        }

        for location in locations {
            self.faction_locations.replace (location, faction_id);
        }

        let terrain_id: ID = self.tiles[end.0][end.1].get_terrain_id ();
        let terrain: Terrain = self.lists.get_terrain (&terrain_id).clone ();
        // TODO: These can't be in here
        // let leader_id: Vec<Response> = self.notify (Message::FactionGetLeader (faction_id, unit_id));
        // let leader_id: ID = if let Response::FactionGetLeader (l) = Handler::reduce_responses (&leader_id) {
        //     *l
        // } else {
        //     panic! ("Invalid response");
        // };

        self.notify (Message::UnitChangeModifierTerrain (unit_id, terrain.get_modifier_id ()));
        self.unit_locations.insert ((unit_id, end));
        self.try_spawn_recruit (&end, &unit_id);
        // self.notify (Message::UnitSendPassive (unit_id));

        true
    }

    fn find_unit_movable (&self, unit_id: &ID) -> Vec<ID> {
        todo! ()
    }

    fn find_unit_cities (&self, unit_id: &ID, faction_id: &ID) -> Vec<ID> {
        assert! (is_rectangular (&self.tiles));

        let location: Location = self.get_unit_location (unit_id)
                .expect (&format! ("Location not found for unit {}", unit_id)).clone ();
        let mut locations: VecDeque<Location> = VecDeque::new ();
        let mut is_visited: Vec<Vec<bool>> = vec![vec![false; self.tiles[0].len ()]; self.tiles.len ()];
        let mut city_ids: Vec<ID> = Vec::new ();

        locations.push_back (location);
        is_visited[location.0][location.1] = true;

        while locations.len () > 0 {
            let location: Location = locations.pop_front ()
                    .expect ("Location not found");

            if let Some (c) = self.tiles[location.0][location.1].get_city_id () {
                city_ids.push (c);
            }

            for direction in DIRECTIONS {
                if let Some (n) = self.try_connect (&location, direction) {
                    let controller_id: &ID = self.faction_locations.get_second (&n)
                            .expect (&format! ("Faction not found for location {:?}", n));

                    if !is_visited[n.0][n.1] && controller_id == faction_id {
                        locations.push_back (n);
                        is_visited[n.0][n.1] = true;
                    }
                }
            }
        }

        city_ids
    }

    fn find_locations (&self, location: &Location, search: Search) -> Vec<Location> {
        assert! (is_rectangular (&self.tiles));
        assert! (is_in_bounds (&self.tiles, location));

        let mut locations: HashSet<Location> = HashSet::new ();
        let location: Location = *location;

        match search {
            Search::Single => { locations.insert (location); }
            Search::Radial (r) => {
                let range: usize = r as usize;

                for i in location.0.checked_sub (range).unwrap_or (0) ..= (location.0 + range) {
                    for j in location.1.checked_sub (range).unwrap_or (0) ..= (location.1 + range) {
                        let distance: usize = location.0.abs_diff (i) + location.1.abs_diff (j);

                        if distance <= range {
                            locations.insert ((i, j));
                        }
                    }
                }
            }
            Search::Path (w, r, d) => {
                let mut starts: HashSet<Location> = HashSet::new ();

                for i in 0 ..= w {
                    let (i, j): (usize, usize) = (i as usize, i as usize);

                    match d {
                        Direction::Up => {
                            let left: Location = (location.0.checked_sub (1).unwrap_or (0), location.1 + j);
                            let right: Location = (location.0.checked_sub (1).unwrap_or (0), location.1.checked_sub (j).unwrap_or (0));

                            if left != location {
                                starts.insert (left);
                            }

                            if right != location {
                                starts.insert (right);
                            }
                        }
                        Direction::Right => {
                            let up: Location = (location.0.checked_sub (i).unwrap_or (0), location.1 + 1);
                            let down: Location = (location.0 + i, location.1 + 1);

                            if up != location {
                                starts.insert (up);
                            }

                            if down != location {
                                starts.insert (down);
                            }
                        }
                        Direction::Left => {
                            let up: Location = (location.0.checked_sub (i).unwrap_or (0), location.1.checked_sub (1).unwrap_or (0));
                            let down: Location = (location.0 + i, location.1.checked_sub (1).unwrap_or (0));

                            if up != location {
                                starts.insert (up);
                            }

                            if down != location {
                                starts.insert (down);
                            }
                        }
                        Direction::Down => {
                            let left: Location = (location.0.checked_sub (1).unwrap_or (0), location.1.checked_sub (j).unwrap_or (0));
                            let right: Location = (location.0 + 1, location.1 + j);

                            if left != location {
                                starts.insert (left);
                            }

                            if right != location {
                                starts.insert (right);
                            }
                        }
                        _ => panic! ("Invalid direction {:?}", d),
                    }
                }

                for i in 0 .. r {
                    let (i, j): (usize, usize) = (i as usize, i as usize);

                    match d {
                        Direction::Up => {
                            locations.extend (starts.iter ().map (|l: &Location|
                                (l.0.checked_sub (i).unwrap_or (0), l.1)
                            ));
                        }
                        Direction::Right => {
                            locations.extend (starts.iter ().map (|l: &Location|
                                (l.0, l.1 + j)
                            ));
                        }
                        Direction::Left => {
                            locations.extend (starts.iter ().map (|l: &Location|
                                (l.0, l.1.checked_sub (j).unwrap_or (0))
                            ));
                        }
                        Direction::Down => {
                            locations.extend (starts.iter ().map (|l: &Location|
                                (l.0 + i, l.1)
                            ));
                        }
                        _ => panic! ("Invalid direction {:?}", d),
                    }
                }
            }
        }

        locations.retain (|l: &Location| is_in_bounds (&self.tiles, l));

        locations.into_iter ().collect::<Vec<Location>> ()
    }

    fn find_units (&self, location: &Location, search: Search) -> Vec<ID> {
        assert! (is_rectangular (&self.tiles));
        assert! (is_in_bounds (&self.tiles, location));

        let locations: Vec<Location> = self.find_locations (location, search);

        locations.iter ().filter_map (|l: &Location|
            self.unit_locations.get_second (l)
            .map (|u: &ID| u.clone ())
        ).collect::<Vec<ID>> ()
    }

    fn add_status (&self, location: &Location, status: Status) -> bool {
        assert! (is_rectangular (&self.tiles));
        assert! (is_in_bounds (&self.tiles, location));

        let is_added: bool = self.tiles[location.0][location.1].add_status (status);

        if let Trigger::None = status.get_trigger () {
            self.update_adjacency (location);
        }

        is_added
    }

    fn try_yield_appliable (&self, location: &Location) -> Option<Box<dyn Appliable>> {
        assert! (is_rectangular (&self.tiles));
        assert! (is_in_bounds (&self.tiles, location));

        self.tiles[location.0][location.1].try_yield_appliable (Rc::clone (&self.lists))
    }

    pub fn get_unit_location (&self, unit_id: &ID) -> Option<&Location> {
        self.unit_locations.get_first (unit_id)
    }

    pub fn get_location_unit (&self, location: &Location) -> Option<&ID> {
        assert! (is_in_bounds (&self.tiles, location));

        self.unit_locations.get_second (location)
    }

    pub fn get_faction_locations (&self, faction_id: &ID) -> Option<&HashSet<Location>> {
        self.faction_locations.get_first (faction_id)
    }

    pub fn get_location_faction (&self, location: &Location) -> Option<&ID> {
        assert! (is_in_bounds (&self.tiles, location));

        self.faction_locations.get_second (location)
    }
}

impl Observer for Grid {
    fn respond (&self, message: Message) -> Option<Response> {
        match message {
            Message::GridFindUnits (u, s) => {
                let location: &Location = self.get_unit_location (&u)
                        .expect (&format! ("Location not found for unit {}", u));
                let unit_ids: Vec<ID> = self.find_units (location, s);

                Some (Response::GridFindUnits (unit_ids))
            }
            Message::GridFindLocations (l, s) => {
                let locations: Vec<Location> = self.find_locations (&l, s);

                Some (Response::GridFindLocations (locations))
            }
            Message::GridGetUnitLocation (u) => {
                let location: &Location = self.get_unit_location (&u)
                        .expect (&format! ("Location not found for unit {}", u));

                Some (Response::GridGetUnitLocation (*location))
            }
            Message::GridIsUnitOnImpassable (u) => {
                let location: &Location = self.get_unit_location (&u)
                        .expect (&format! ("Location not found for unit {}", u));
                let is_on_impassable: bool = self.is_impassable (location);

                Some (Response::GridIsUnitOnImpassable (is_on_impassable))
            }
            Message::GridFindUnitCities (u, f) => {
                let city_ids: Vec<ID> = self.find_unit_cities (&u, &f);

                Some (Response::GridFindUnitCities (city_ids))
            }
            Message::GridAddStatus (l, s) => {
                let status: Status = self.lists.get_status (&s).clone ();

                self.add_status (&l, status);

                Some (Response::GridAddStatus)
            }
            Message::GridTryYieldAppliable (u) => {
                let location: &Location = self.get_unit_location (&u)
                        .expect (&format! ("Location not found for unit {}", u));
                let change: Option<Change> = self.try_yield_appliable (location).map (|a: Box<dyn Appliable>|
                    a.change ()
                );

                Some (Response::GridTryYieldAppliable (change))
            }
            Message::GridFindDistanceBetween (u0, u1) => {
                let distance: usize = self.find_distance_between (&u0, &u1);

                Some (Response::GridFindDistanceBetween (distance))
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

impl Subject for Grid {
    fn notify (&self, message: Message) -> Vec<Response> {
        self.handler.upgrade ()
                .expect (&format! ("Pointer upgrade failed for {:?}", self.handler))
                .borrow ()
                .notify (message)
    }
}

impl fmt::Display for Grid {
    fn fmt (&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut display: String = String::from ("");

        for i in 0 .. self.tiles.len () {
            for j in 0 .. self.tiles[i].len () {
                let tile: &Tile = &self.tiles[i][j];

                if self.is_occupied (&(i, j)) {
                    display.push_str (&format! ("{}o{} ",
                            self.unit_locations.get_second (&(i, j))
                                    .expect (&format! ("Missing unit on ({}, {})", i, j)),
                            tile.get_height ()));
                } else {
                    display.push_str (&format! ("{}_{} ", self.lists.get_terrain (&tile.get_terrain_id ()), tile.get_height ()));
                }
            }

            display.push_str ("\n");
        }

        write! (f, "{}", display)
    }
}    

#[cfg (test)]
pub mod tests {
    use super::*;
    use crate::character::unit::tests::{generate_factions, generate_units};
    use crate::event::{EVENT_FACTION_ADD_FOLLOWER, EVENT_FACTION_ADD_MEMBER, EVENT_FACTION_GET_FOLLOWERS, EVENT_FACTION_GET_LEADER, EVENT_FACTION_IS_MEMBER, EVENT_GRID_FIND_DISTANCE_BETWEEN, EVENT_UNIT_GET_FACTION_ID, EVENT_UNIT_TRY_ADD_PASSIVE};
    use crate::map::TileBuilder;
    use crate::event::handler::tests::generate_handler;
    use crate::tests::generate_lists;

    fn generate_tiles () -> Vec<Vec<Tile>> {
        let lists = generate_lists ();
        let tile_builder = TileBuilder::new (Rc::clone (&lists));

        vec![
            vec![tile_builder.build (0, 0, Some (0)), tile_builder.build (0, 1, None), tile_builder.build (0, 0, Some (1))],
            vec![tile_builder.build (1, 2, Some (2)), tile_builder.build (1, 1, None), tile_builder.build (2, 0, None)]
        ]
    }

    fn generate_faction_units () -> OuterJoinMap<ID, ID> {
        let mut faction_units: OuterJoinMap<ID, ID> = OuterJoinMap::new ();

        faction_units.insert ((0, 0));
        faction_units.insert ((0, 1));
        faction_units.insert ((1, 2));
        faction_units.insert ((0, 3));
        faction_units.insert ((2, 4));

        faction_units
    }

    pub fn generate_grid (handler: Weak<RefCell<Handler>>) -> Rc<RefCell<Grid>> {
        let lists = generate_lists ();
        let tiles: Vec<Vec<Tile>> = generate_tiles ();
        let grid = Grid::new (Rc::clone (&lists), tiles, handler);
        let grid = RefCell::new (grid);
        let grid = Rc::new (grid);

        grid
    }

    #[test]
    fn grid_get_cost () {
        let handler = generate_handler ();
        let grid = generate_grid (Rc::downgrade (&handler));

        assert_eq! (grid.borrow ().get_cost (&(0, 0), Direction::Up), 0);
        assert_eq! (grid.borrow ().get_cost (&(0, 0), Direction::Right), 2);
        assert_eq! (grid.borrow ().get_cost (&(0, 0), Direction::Left), 0);
        assert_eq! (grid.borrow ().get_cost (&(0, 0), Direction::Down), 0);
        assert_eq! (grid.borrow ().get_cost (&(0, 1), Direction::Up), 0);
        assert_eq! (grid.borrow ().get_cost (&(0, 1), Direction::Right), 2);
        assert_eq! (grid.borrow ().get_cost (&(0, 1), Direction::Left), 2);
        assert_eq! (grid.borrow ().get_cost (&(0, 1), Direction::Down), 2);
        assert_eq! (grid.borrow ().get_cost (&(0, 2), Direction::Up), 0);
        assert_eq! (grid.borrow ().get_cost (&(0, 2), Direction::Right), 0);
        assert_eq! (grid.borrow ().get_cost (&(0, 2), Direction::Left), 2);
        assert_eq! (grid.borrow ().get_cost (&(0, 2), Direction::Down), 0);

        assert_eq! (grid.borrow ().get_cost (&(1, 0), Direction::Up), 0);
        assert_eq! (grid.borrow ().get_cost (&(1, 0), Direction::Right), 3);
        assert_eq! (grid.borrow ().get_cost (&(1, 0), Direction::Left), 0);
        assert_eq! (grid.borrow ().get_cost (&(1, 0), Direction::Down), 0);
        assert_eq! (grid.borrow ().get_cost (&(1, 1), Direction::Up), 1);
        assert_eq! (grid.borrow ().get_cost (&(1, 1), Direction::Right), 0);
        assert_eq! (grid.borrow ().get_cost (&(1, 1), Direction::Left), 3);
        assert_eq! (grid.borrow ().get_cost (&(1, 1), Direction::Down), 0);
        assert_eq! (grid.borrow ().get_cost (&(1, 2), Direction::Up), 0);
        assert_eq! (grid.borrow ().get_cost (&(1, 2), Direction::Right), 0);
        assert_eq! (grid.borrow ().get_cost (&(1, 2), Direction::Left), 0);
        assert_eq! (grid.borrow ().get_cost (&(1, 2), Direction::Down), 0);
    }

    #[test]
    fn grid_is_impassable () {
        let handler = generate_handler ();
        let grid = generate_grid (Rc::downgrade (&handler));

        // Test passable
        assert_eq! (grid.borrow ().is_impassable (&(0, 0)), false);
        // Test impassable
        assert_eq! (grid.borrow ().is_impassable (&(1, 2)), true);
    }

    #[test]
    fn grid_is_occupied () {
        let handler = generate_handler ();
        let grid = generate_grid (Rc::downgrade (&handler));

        // Test empty
        assert_eq! (grid.borrow ().is_occupied (&(0, 0)), false);
        // Test occupied
        grid.borrow_mut ().unit_locations.insert ((0, (0, 0)));
        assert_eq! (grid.borrow ().is_occupied (&(0, 0)), true);
    }

    #[test]
    fn grid_is_placeable () {
        let handler = generate_handler ();
        let grid = generate_grid (Rc::downgrade (&handler));

        // Test passable
        assert_eq! (grid.borrow ().is_placeable (&(0, 0)), true);
        assert_eq! (grid.borrow ().is_placeable (&(0, 1)), true);
        assert_eq! (grid.borrow ().is_placeable (&(0, 2)), true);
        assert_eq! (grid.borrow ().is_placeable (&(1, 0)), true);
        assert_eq! (grid.borrow ().is_placeable (&(1, 1)), true);
        // Test impassable
        assert_eq! (grid.borrow ().is_placeable (&(1, 2)), false);
        // Test occupied
        grid.borrow_mut ().unit_locations.insert ((0, (0, 0)));
        assert_eq! (grid.borrow ().is_placeable (&(0, 0)), false);
    }

    #[test]
    fn grid_find_nearest_placeable () {
        let handler = generate_handler ();
        let grid = generate_grid (Rc::downgrade (&handler));

        grid.borrow_mut ().faction_units = generate_faction_units ();
        // Test empty find
        assert_eq! (grid.borrow ().find_nearest_placeable (&(0, 0)), (0, 0));
        // Test non-empty find
        grid.borrow_mut ().place_unit (0, (0, 0));
        assert! (grid.borrow ().find_nearest_placeable (&(0, 0)) == (0, 1)
            || grid.borrow ().find_nearest_placeable (&(0, 0)) == (1, 0)
        );
    }

    #[test]
    fn grid_find_distance_between () {
        let handler = generate_handler ();
        let grid = generate_grid (Rc::downgrade (&handler));

        grid.borrow_mut ().faction_units = generate_faction_units ();
        grid.borrow_mut ().place_unit (0, (0, 0));
        grid.borrow_mut ().place_unit (1, (0, 1));
        grid.borrow_mut ().place_unit (2, (1, 1));
        assert_eq! (grid.borrow ().find_distance_between (&0, &1), 1);
        assert_eq! (grid.borrow ().find_distance_between (&1, &0), 1);
        assert_eq! (grid.borrow ().find_distance_between (&0, &2), 2);
        assert_eq! (grid.borrow ().find_distance_between (&2, &0), 2);
        assert_eq! (grid.borrow ().find_distance_between (&1, &2), 1);
        assert_eq! (grid.borrow ().find_distance_between (&2, &1), 1);
    }

    #[test]
    fn grid_try_spawn_recruit () {
        let handler = generate_handler ();
        let (unit_0, unit_1, _) = generate_units (Rc::clone (&handler));
        let grid = generate_grid (Rc::downgrade (&handler));
        let (faction_0, _) = generate_factions (Rc::clone (&handler));
        let grid_id = handler.borrow_mut ().register (Rc::clone (&grid) as Rc<RefCell<dyn Observer>>);
        let unit_0_id = handler.borrow_mut ().register (Rc::clone (&unit_0) as Rc<RefCell<dyn Observer>>);
        let unit_1_id = handler.borrow_mut ().register (Rc::clone (&unit_1) as Rc<RefCell<dyn Observer>>);
        let faction_0_id = handler.borrow_mut ().register (Rc::clone (&faction_0) as Rc<RefCell<dyn Observer>>);

        handler.borrow_mut ().subscribe (grid_id, EVENT_GRID_FIND_DISTANCE_BETWEEN);
        handler.borrow_mut ().subscribe (unit_0_id, EVENT_UNIT_GET_FACTION_ID);
        handler.borrow_mut ().subscribe (unit_1_id, EVENT_UNIT_GET_FACTION_ID);
        handler.borrow_mut ().subscribe (unit_1_id, EVENT_UNIT_TRY_ADD_PASSIVE);
        handler.borrow_mut ().subscribe (faction_0_id, EVENT_FACTION_IS_MEMBER);
        handler.borrow_mut ().subscribe (faction_0_id, EVENT_FACTION_ADD_MEMBER);
        handler.borrow_mut ().subscribe (faction_0_id, EVENT_FACTION_ADD_FOLLOWER);
        handler.borrow_mut ().subscribe (faction_0_id, EVENT_FACTION_GET_LEADER);
        handler.borrow_mut ().subscribe (faction_0_id, EVENT_FACTION_GET_FOLLOWERS);
        grid.borrow_mut ().place_unit (0, (0, 1));
        faction_0.borrow ().add_follower (0, 0);

        grid.borrow_mut ().move_unit (0, vec![Direction::Left]);
        assert! (grid.borrow ().get_unit_location (&1).unwrap () == &(0, 1)
            || grid.borrow ().get_unit_location (&1).unwrap () == &(1, 0)
        );
        assert_eq! (faction_0.borrow ().is_member (&1), true);
        assert_eq! (faction_0.borrow ().get_leader (&1), 0);
        assert_eq! (faction_0.borrow ().get_followers (&0).len (), 2);
        assert_eq! (faction_0.borrow ().get_followers (&0).contains (&0), true);
        assert_eq! (faction_0.borrow ().get_followers (&0).contains (&1), true);
    }

    #[test]
    fn grid_try_connect () {
        let handler = generate_handler ();
        let grid = generate_grid (Rc::downgrade (&handler));

        assert_eq! (grid.borrow ().try_connect (&(0, 0), Direction::Up), None);
        assert_eq! (grid.borrow ().try_connect (&(0, 0), Direction::Right).unwrap (), (0, 1));
        assert_eq! (grid.borrow ().try_connect (&(0, 0), Direction::Left), None);
        assert_eq! (grid.borrow ().try_connect (&(0, 0), Direction::Down).unwrap (), (1, 0));
        assert_eq! (grid.borrow ().try_connect (&(0, 1), Direction::Up), None);
        assert_eq! (grid.borrow ().try_connect (&(0, 1), Direction::Right).unwrap (), (0, 2));
        assert_eq! (grid.borrow ().try_connect (&(0, 1), Direction::Left).unwrap (), (0, 0));
        assert_eq! (grid.borrow ().try_connect (&(0, 1), Direction::Down).unwrap (), (1, 1));
        assert_eq! (grid.borrow ().try_connect (&(0, 2), Direction::Up), None);
        assert_eq! (grid.borrow ().try_connect (&(0, 2), Direction::Right), None);
        assert_eq! (grid.borrow ().try_connect (&(0, 2), Direction::Left).unwrap (), (0, 1));
        assert_eq! (grid.borrow ().try_connect (&(0, 2), Direction::Down).unwrap (), (1, 2));

        assert_eq! (grid.borrow ().try_connect (&(1, 0), Direction::Up).unwrap (), (0, 0));
        assert_eq! (grid.borrow ().try_connect (&(1, 0), Direction::Right).unwrap (), (1, 1));
        assert_eq! (grid.borrow ().try_connect (&(1, 0), Direction::Left), None);
        assert_eq! (grid.borrow ().try_connect (&(1, 0), Direction::Down), None);
        assert_eq! (grid.borrow ().try_connect (&(1, 1), Direction::Up).unwrap (), (0, 1));
        assert_eq! (grid.borrow ().try_connect (&(1, 1), Direction::Right).unwrap (), (1, 2));
        assert_eq! (grid.borrow ().try_connect (&(1, 1), Direction::Left).unwrap (), (1, 0));
        assert_eq! (grid.borrow ().try_connect (&(1, 1), Direction::Down), None);
        assert_eq! (grid.borrow ().try_connect (&(1, 2), Direction::Up).unwrap (), (0, 2));
        assert_eq! (grid.borrow ().try_connect (&(1, 2), Direction::Right), None);
        assert_eq! (grid.borrow ().try_connect (&(1, 2), Direction::Left).unwrap (), (1, 1));
        assert_eq! (grid.borrow ().try_connect (&(1, 2), Direction::Down), None);
    }

    #[test]
    fn grid_try_move () {
        let handler = generate_handler ();
        let grid = generate_grid (Rc::downgrade (&handler));

        // Test empty move
        assert_eq! (grid.borrow ().try_move (&(0, 0), Direction::Up), None);
        assert_eq! (grid.borrow ().try_move (&(0, 0), Direction::Right).unwrap (), ((0, 1), 2));
        assert_eq! (grid.borrow ().try_move (&(0, 0), Direction::Left), None);
        assert_eq! (grid.borrow ().try_move (&(0, 0), Direction::Down), None);
        assert_eq! (grid.borrow ().try_move (&(0, 1), Direction::Up), None);
        assert_eq! (grid.borrow ().try_move (&(0, 1), Direction::Right).unwrap (), ((0, 2), 2));
        assert_eq! (grid.borrow ().try_move (&(0, 1), Direction::Left).unwrap (), ((0, 0), 2));
        assert_eq! (grid.borrow ().try_move (&(0, 1), Direction::Down).unwrap (), ((1, 1), 2));
        assert_eq! (grid.borrow ().try_move (&(0, 2), Direction::Up), None);
        assert_eq! (grid.borrow ().try_move (&(0, 2), Direction::Right), None);
        assert_eq! (grid.borrow ().try_move (&(0, 2), Direction::Left).unwrap (), ((0, 1), 2));
        assert_eq! (grid.borrow ().try_move (&(0, 2), Direction::Down), None);

        assert_eq! (grid.borrow ().try_move (&(1, 0), Direction::Up), None);
        assert_eq! (grid.borrow ().try_move (&(1, 0), Direction::Right).unwrap (), ((1, 1), 3));
        assert_eq! (grid.borrow ().try_move (&(1, 0), Direction::Left), None);
        assert_eq! (grid.borrow ().try_move (&(1, 0), Direction::Down), None);
        assert_eq! (grid.borrow ().try_move (&(1, 1), Direction::Up).unwrap (), ((0, 1), 1));
        assert_eq! (grid.borrow ().try_move (&(1, 1), Direction::Right), None);
        assert_eq! (grid.borrow ().try_move (&(1, 1), Direction::Left).unwrap (), ((1, 0), 3));
        assert_eq! (grid.borrow ().try_move (&(1, 1), Direction::Down), None);
        assert_eq! (grid.borrow ().try_move (&(1, 2), Direction::Up), None);
        assert_eq! (grid.borrow ().try_move (&(1, 2), Direction::Right), None);
        assert_eq! (grid.borrow ().try_move (&(1, 2), Direction::Left), None);
        assert_eq! (grid.borrow ().try_move (&(1, 2), Direction::Down), None);

        // Test non-empty move
        grid.borrow_mut ().unit_locations.insert ((0, (0, 1)));
        assert_eq! (grid.borrow ().try_move (&(0, 0), Direction::Right), None);
        assert_eq! (grid.borrow ().try_move (&(0, 2), Direction::Left), None);
        assert_eq! (grid.borrow ().try_move (&(1, 1), Direction::Up), None);
    }

    #[test]
    fn grid_update_adjacency () {
        let lists = generate_lists ();
        let tile_builder: TileBuilder = TileBuilder::new (Rc::clone (&lists));
        let tiles_updated: Vec<Vec<Tile>> = vec![
            vec![tile_builder.build (0, 10, Some (0)) /* changed */, tile_builder.build (0, 1, None), tile_builder.build (0, 0, Some (1))],
            vec![tile_builder.build (1, 2, None), tile_builder.build (1, 1, None), tile_builder.build (0, 0, None) /* changed */]
        ];
        let handler = generate_handler ();
        let grid = generate_grid (Rc::downgrade (&handler));

        grid.borrow_mut ().tiles = tiles_updated;
        // Test impassable update
        grid.borrow ().update_adjacency (&(0, 0));
        assert_eq! (grid.borrow ().try_move (&(0, 1), Direction::Left), None);
        assert_eq! (grid.borrow ().try_move (&(1, 0), Direction::Up), None);
        // Test passable update
        grid.borrow ().update_adjacency (&(1, 2));
        assert_eq! (grid.borrow ().try_move (&(0, 2), Direction::Down).unwrap (), ((1, 2), 1));
        assert_eq! (grid.borrow ().try_move (&(1, 1), Direction::Right).unwrap (), ((1, 2), 2));
    }

    #[test]
    fn grid_place_unit () {
        let handler = generate_handler ();
        let grid = generate_grid (Rc::downgrade (&handler));

        grid.borrow_mut ().faction_units = generate_faction_units ();
        // Test empty place
        assert_eq! (grid.borrow_mut ().place_unit (0, (0, 0)), true);
        assert_eq! (grid.borrow ().faction_locations.get_second (&(0, 0)).unwrap (), &0);
        // Test impassable place
        assert_eq! (grid.borrow_mut ().place_unit (1, (1, 2)), false);
        // Test non-empty place
        assert_eq! (grid.borrow_mut ().place_unit (2, (0, 0)), false);
    }

    #[test]
    fn grid_move_unit () {
        let handler = generate_handler ();
        let grid = generate_grid (Rc::downgrade (&handler));

        grid.borrow_mut ().faction_units = generate_faction_units ();
        grid.borrow_mut ().place_unit (0, (0, 0));
        assert_eq! (grid.borrow_mut ().move_unit (0, vec![Direction::Up]), false); // Test out-of-bounnds
        assert_eq! (grid.borrow_mut ().move_unit (0, vec![Direction::Down]), false); // Test not climbable
        assert_eq! (grid.borrow_mut ().move_unit (0, vec![Direction::Left]), false); // Test out-of-bounds
        // Test normal move
        assert_eq! (grid.borrow ().faction_locations.get_second (&(0, 1)).unwrap (), &ID_UNINITIALISED);
        assert_eq! (grid.borrow_mut ().move_unit (0, vec![Direction::Right]), true);
        assert_eq! (grid.borrow ().get_unit_location (&0).unwrap (), &(0, 1));
        assert_eq! (grid.borrow ().faction_locations.get_second (&(0, 0)).unwrap (), &0);
        assert_eq! (grid.borrow ().faction_locations.get_second (&(0, 1)).unwrap (), &0);
        // Test sequential move
        assert_eq! (grid.borrow ().faction_locations.get_second (&(0, 2)).unwrap (), &ID_UNINITIALISED);
        assert_eq! (grid.borrow ().faction_locations.get_second (&(1, 1)).unwrap (), &ID_UNINITIALISED);
        assert_eq! (grid.borrow_mut ().move_unit (0, vec![Direction::Right, Direction::Left, Direction::Down]), true); // Test overlap
        assert_eq! (grid.borrow ().get_unit_location (&0).unwrap (), &(1, 1));
        assert_eq! (grid.borrow ().faction_locations.get_second (&(0, 1)).unwrap (), &0);
        assert_eq! (grid.borrow ().faction_locations.get_second (&(0, 2)).unwrap (), &0);
        assert_eq! (grid.borrow ().faction_locations.get_second (&(1, 1)).unwrap (), &0);
        // Test atomic move
        assert_eq! (grid.borrow_mut ().move_unit (0, vec![Direction::Left, Direction::Right, Direction::Right]), false); // Test impassable
        assert_eq! (grid.borrow ().get_unit_location (&0).unwrap (), &(1, 1));
    }

    #[test]
    fn grid_find_unit_cities () {
        let handler = generate_handler ();
        let grid = generate_grid (Rc::downgrade (&handler));

        grid.borrow_mut ().faction_units = generate_faction_units ();
        // Test no supply
        grid.borrow_mut ().place_unit (0, (0, 1));
        assert_eq! (grid.borrow ().find_unit_cities (&0, &0).len (), 0);
        grid.borrow_mut ().move_unit (0, vec![Direction::Down]);
        assert_eq! (grid.borrow ().find_unit_cities (&0, &0).len (), 0);
        // Test contested supply
        grid.borrow_mut ().place_unit (2, (0, 0));
        assert_eq! (grid.borrow ().find_unit_cities (&0, &0).len (), 0);
        assert_eq! (grid.borrow ().find_unit_cities (&2, &1).len (), 1);
        // Test normal supply
        grid.borrow_mut ().place_unit (1, (0, 2));
        assert_eq! (grid.borrow ().find_unit_cities (&0, &0).len (), 1);
        assert_eq! (grid.borrow ().find_unit_cities (&1, &0).len (), 1);
        assert_eq! (grid.borrow ().find_unit_cities (&2, &1).len (), 1);
        // Test multiple supply
        grid.borrow_mut ().place_unit (3, (1, 0));
        assert_eq! (grid.borrow ().find_unit_cities (&0, &0).len (), 2);
        assert_eq! (grid.borrow ().find_unit_cities (&1, &0).len (), 2);
        assert_eq! (grid.borrow ().find_unit_cities (&2, &1).len (), 1);
    }

    #[test]
    fn grid_find_locations () {
        let handler = generate_handler ();
        let grid = generate_grid (Rc::downgrade (&handler));

        assert_eq! (grid.borrow ().find_locations (&(0, 0), Search::Single).len (), 1);
        assert_eq! (grid.borrow ().find_locations (&(0, 1), Search::Single).len (), 1);
        assert_eq! (grid.borrow ().find_locations (&(0, 2), Search::Single).len (), 1);
        assert_eq! (grid.borrow ().find_locations (&(1, 0), Search::Single).len (), 1);
        assert_eq! (grid.borrow ().find_locations (&(1, 1), Search::Single).len (), 1);
        assert_eq! (grid.borrow ().find_locations (&(1, 2), Search::Single).len (), 1);

        assert_eq! (grid.borrow ().find_locations (&(0, 0), Search::Path (1, 1, Direction::Right)).len (), 2);
        assert_eq! (grid.borrow ().find_locations (&(0, 1), Search::Path (1, 1, Direction::Down)).len (), 3);
        assert_eq! (grid.borrow ().find_locations (&(0, 2), Search::Path (1, 1, Direction::Left)).len (), 2);
        assert_eq! (grid.borrow ().find_locations (&(1, 0), Search::Path (1, 1, Direction::Right)).len (), 2);
        assert_eq! (grid.borrow ().find_locations (&(1, 1), Search::Path (1, 1, Direction::Up)).len (), 3);
        assert_eq! (grid.borrow ().find_locations (&(1, 2), Search::Path (1, 1, Direction::Left)).len (), 2);

        assert_eq! (grid.borrow ().find_locations (&(0, 0), Search::Path (0, 2, Direction::Right)).len (), 2);
        assert_eq! (grid.borrow ().find_locations (&(0, 1), Search::Path (0, 2, Direction::Down)).len (), 1);
        assert_eq! (grid.borrow ().find_locations (&(0, 2), Search::Path (0, 2, Direction::Left)).len (), 2);
        assert_eq! (grid.borrow ().find_locations (&(1, 0), Search::Path (0, 2, Direction::Right)).len (), 2);
        assert_eq! (grid.borrow ().find_locations (&(1, 1), Search::Path (0, 2, Direction::Up)).len (), 1);
        assert_eq! (grid.borrow ().find_locations (&(1, 2), Search::Path (0, 2, Direction::Left)).len (), 2);

        assert_eq! (grid.borrow ().find_locations (&(0, 0), Search::Radial (1)).len (), 3);
        assert_eq! (grid.borrow ().find_locations (&(0, 1), Search::Radial (1)).len (), 4);
        assert_eq! (grid.borrow ().find_locations (&(0, 2), Search::Radial (1)).len (), 3);
        assert_eq! (grid.borrow ().find_locations (&(1, 0), Search::Radial (1)).len (), 3);
        assert_eq! (grid.borrow ().find_locations (&(1, 1), Search::Radial (1)).len (), 4);
        assert_eq! (grid.borrow ().find_locations (&(1, 2), Search::Radial (1)).len (), 3);

        assert_eq! (grid.borrow ().find_locations (&(0, 0), Search::Radial (2)).len (), 5);
        assert_eq! (grid.borrow ().find_locations (&(0, 1), Search::Radial (2)).len (), 6);
        assert_eq! (grid.borrow ().find_locations (&(0, 2), Search::Radial (2)).len (), 5);
        assert_eq! (grid.borrow ().find_locations (&(1, 0), Search::Radial (2)).len (), 5);
        assert_eq! (grid.borrow ().find_locations (&(1, 1), Search::Radial (2)).len (), 6);
        assert_eq! (grid.borrow ().find_locations (&(1, 2), Search::Radial (2)).len (), 5);
    }

    #[test]
    fn grid_find_units () {
        let handler = generate_handler ();
        let grid = generate_grid (Rc::downgrade (&handler));

        // Test empty find
        assert_eq! (grid.borrow ().find_units (&(0, 0), Search::Single).len (), 0);
        assert_eq! (grid.borrow ().find_units (&(0, 1), Search::Single).len (), 0);
        assert_eq! (grid.borrow ().find_units (&(0, 2), Search::Single).len (), 0);
        assert_eq! (grid.borrow ().find_units (&(1, 0), Search::Single).len (), 0);
        assert_eq! (grid.borrow ().find_units (&(1, 1), Search::Single).len (), 0);
        assert_eq! (grid.borrow ().find_units (&(1, 2), Search::Single).len (), 0);

        assert_eq! (grid.borrow ().find_units (&(0, 0), Search::Path (1, 1, Direction::Right)).len (), 0);
        assert_eq! (grid.borrow ().find_units (&(0, 1), Search::Path (1, 1, Direction::Down)).len (), 0);
        assert_eq! (grid.borrow ().find_units (&(0, 2), Search::Path (1, 1, Direction::Left)).len (), 0);
        assert_eq! (grid.borrow ().find_units (&(1, 0), Search::Path (1, 1, Direction::Right)).len (), 0);
        assert_eq! (grid.borrow ().find_units (&(1, 1), Search::Path (1, 1, Direction::Up)).len (), 0);
        assert_eq! (grid.borrow ().find_units (&(1, 2), Search::Path (1, 1, Direction::Left)).len (), 0);

        assert_eq! (grid.borrow ().find_units (&(0, 0), Search::Path (0, 2, Direction::Right)).len (), 0);
        assert_eq! (grid.borrow ().find_units (&(0, 1), Search::Path (0, 2, Direction::Down)).len (), 0);
        assert_eq! (grid.borrow ().find_units (&(0, 2), Search::Path (0, 2, Direction::Left)).len (), 0);
        assert_eq! (grid.borrow ().find_units (&(1, 0), Search::Path (0, 2, Direction::Right)).len (), 0);
        assert_eq! (grid.borrow ().find_units (&(1, 1), Search::Path (0, 2, Direction::Up)).len (), 0);
        assert_eq! (grid.borrow ().find_units (&(1, 2), Search::Path (0, 2, Direction::Left)).len (), 0);

        assert_eq! (grid.borrow ().find_units (&(0, 0), Search::Radial (1)).len (), 0);
        assert_eq! (grid.borrow ().find_units (&(0, 1), Search::Radial (1)).len (), 0);
        assert_eq! (grid.borrow ().find_units (&(0, 2), Search::Radial (1)).len (), 0);
        assert_eq! (grid.borrow ().find_units (&(1, 0), Search::Radial (1)).len (), 0);
        assert_eq! (grid.borrow ().find_units (&(1, 1), Search::Radial (1)).len (), 0);
        assert_eq! (grid.borrow ().find_units (&(1, 2), Search::Radial (1)).len (), 0);

        assert_eq! (grid.borrow ().find_units (&(0, 0), Search::Radial (2)).len (), 0);
        assert_eq! (grid.borrow ().find_units (&(0, 1), Search::Radial (2)).len (), 0);
        assert_eq! (grid.borrow ().find_units (&(0, 2), Search::Radial (2)).len (), 0);
        assert_eq! (grid.borrow ().find_units (&(1, 0), Search::Radial (2)).len (), 0);
        assert_eq! (grid.borrow ().find_units (&(1, 1), Search::Radial (2)).len (), 0);
        assert_eq! (grid.borrow ().find_units (&(1, 2), Search::Radial (2)).len (), 0);
        // Test non-empty find
        grid.borrow_mut ().faction_units = generate_faction_units ();
        grid.borrow_mut ().place_unit (0, (0, 0));
        grid.borrow_mut ().place_unit (1, (0, 1));
        grid.borrow_mut ().place_unit (2, (0, 2));
        grid.borrow_mut ().place_unit (3, (1, 0));
        grid.borrow_mut ().place_unit (4, (1, 1));
        assert_eq! (grid.borrow ().find_units (&(0, 0), Search::Single).len (), 1);
        assert_eq! (grid.borrow ().find_units (&(0, 1), Search::Single).len (), 1);
        assert_eq! (grid.borrow ().find_units (&(0, 2), Search::Single).len (), 1);
        assert_eq! (grid.borrow ().find_units (&(1, 0), Search::Single).len (), 1);
        assert_eq! (grid.borrow ().find_units (&(1, 1), Search::Single).len (), 1);
        assert_eq! (grid.borrow ().find_units (&(1, 2), Search::Single).len (), 0);

        assert_eq! (grid.borrow ().find_units (&(0, 0), Search::Path (1, 1, Direction::Right)).len (), 2);
        assert_eq! (grid.borrow ().find_units (&(0, 1), Search::Path (1, 1, Direction::Down)).len (), 2);
        assert_eq! (grid.borrow ().find_units (&(0, 2), Search::Path (1, 1, Direction::Left)).len (), 2);
        assert_eq! (grid.borrow ().find_units (&(1, 0), Search::Path (1, 1, Direction::Right)).len (), 2);
        assert_eq! (grid.borrow ().find_units (&(1, 1), Search::Path (1, 1, Direction::Up)).len (), 3);
        assert_eq! (grid.borrow ().find_units (&(1, 2), Search::Path (1, 1, Direction::Left)).len (), 2);

        assert_eq! (grid.borrow ().find_units (&(0, 0), Search::Path (0, 2, Direction::Right)).len (), 2);
        assert_eq! (grid.borrow ().find_units (&(0, 1), Search::Path (0, 2, Direction::Down)).len (), 1);
        assert_eq! (grid.borrow ().find_units (&(0, 2), Search::Path (0, 2, Direction::Left)).len (), 2);
        assert_eq! (grid.borrow ().find_units (&(1, 0), Search::Path (0, 2, Direction::Right)).len (), 1);
        assert_eq! (grid.borrow ().find_units (&(1, 1), Search::Path (0, 2, Direction::Up)).len (), 1);
        assert_eq! (grid.borrow ().find_units (&(1, 2), Search::Path (0, 2, Direction::Left)).len (), 2);

        assert_eq! (grid.borrow ().find_units (&(0, 0), Search::Radial (1)).len (), 3);
        assert_eq! (grid.borrow ().find_units (&(0, 1), Search::Radial (1)).len (), 4);
        assert_eq! (grid.borrow ().find_units (&(0, 2), Search::Radial (1)).len (), 2);
        assert_eq! (grid.borrow ().find_units (&(1, 0), Search::Radial (1)).len (), 3);
        assert_eq! (grid.borrow ().find_units (&(1, 1), Search::Radial (1)).len (), 3);
        assert_eq! (grid.borrow ().find_units (&(1, 2), Search::Radial (1)).len (), 2);

        assert_eq! (grid.borrow ().find_units (&(0, 0), Search::Radial (2)).len (), 5);
        assert_eq! (grid.borrow ().find_units (&(0, 1), Search::Radial (2)).len (), 5);
        assert_eq! (grid.borrow ().find_units (&(0, 2), Search::Radial (2)).len (), 4);
        assert_eq! (grid.borrow ().find_units (&(1, 0), Search::Radial (2)).len (), 4);
        assert_eq! (grid.borrow ().find_units (&(1, 1), Search::Radial (2)).len (), 5);
        assert_eq! (grid.borrow ().find_units (&(1, 2), Search::Radial (2)).len (), 4);
    }

    #[test]
    fn grid_add_status () {
        let lists = generate_lists ();
        let handler = generate_handler ();
        let grid = generate_grid (Rc::downgrade (&handler));
        let status_0 = lists.get_status (&0).clone ();
        let status_2 = lists.get_status (&2).clone ();
        let status_3 = lists.get_status (&3).clone ();

        assert_eq! (grid.borrow ().add_status (&(0, 0), status_0), false);
        assert_eq! (grid.borrow ().add_status (&(0, 0), status_2), true);
        let cost_down_0: u8 = grid.borrow ().adjacencies.borrow ()[0][0][Direction::Down as usize];
        let cost_left_0: u8 = grid.borrow ().adjacencies.borrow ()[1][1][Direction::Left as usize];
        assert_eq! (grid.borrow ().add_status (&(1, 0), status_3), true);
        let cost_down_1: u8 = grid.borrow ().adjacencies.borrow ()[0][0][Direction::Down as usize];
        let cost_left_1: u8 = grid.borrow ().adjacencies.borrow ()[1][1][Direction::Left as usize];
        assert_eq! (cost_down_0, 0);
        assert_eq! (cost_down_1, 0);
        assert! (cost_left_0 > cost_left_1);
    }
}
