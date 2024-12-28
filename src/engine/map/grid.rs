use std::{cell::RefCell, collections::{HashMap, HashSet, VecDeque}, fmt, rc::Rc};
use crate::engine::Lists;
use crate::engine::common::{Area, FACTION_NONE, ID, ID_UNINITIALISED, Target};
use crate::engine::duplicate_map::{DuplicateInnerMap, DuplicateOuterMap};
use crate::engine::event::{Event, Subject, Observer, Response, RESPONSE_NOTIFICATION};
use crate::engine::dynamic::{Changeable, Modifier};
use super::{COST_IMPASSABLE, Tile};

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
    adjacencies: Vec<Vec<Adjacency>>,
    unit_locations: DuplicateInnerMap<ID, Location>,
    faction_locations: DuplicateOuterMap<ID, Location>,
    // TODO: This is definitely getting replaced whenever factions are done
    faction_units: DuplicateOuterMap<ID, ID>,
    observer_id: ID,
}

impl Grid {
    pub fn new (lists: Rc<Lists>, tiles: Vec<Vec<Tile>>, unit_factions: HashMap<ID, ID>) -> Self {
        assert! (is_rectangular (&tiles));

        let mut faction_locations: DuplicateOuterMap<ID, Location> = DuplicateOuterMap::new ();
        let adjacencies: Vec<Vec<Adjacency>> = Grid::build_adjacencies (&tiles);
        let unit_locations: DuplicateInnerMap<ID, Location> = DuplicateInnerMap::new ();
        let mut faction_units: DuplicateOuterMap<ID, ID> = DuplicateOuterMap::new ();
        let observer_id: ID = ID_UNINITIALISED;

        for i in 0 .. tiles.len () {
            for j in 0 .. tiles[i].len () {
                faction_locations.insert ((FACTION_NONE, (i, j)));
            }
        }

        for (unit_id, faction_id) in unit_factions {
            faction_units.insert ((faction_id, unit_id));
        }

        Self { lists, tiles, adjacencies, unit_locations, faction_locations, faction_units, observer_id }
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

    pub fn get_cost (&self, location: &Location, direction: Direction) -> u8 {
        assert! (is_rectangular (&self.adjacencies));
        assert! (is_in_bounds (&self.adjacencies, location));

        self.adjacencies[location.0][location.1][direction as usize]
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

    pub fn try_connect (&self, start: &Location, direction: Direction) -> Option<Location> {
        assert! (is_rectangular (&self.adjacencies));
        assert! (is_in_bounds (&self.adjacencies, start));

        let mut end: Location = start.clone ();

        match direction {
            Direction::Up => end.0 = start.0.checked_sub (1)?,
            Direction::Right => end.1 = start.1.checked_add (1)?,
            Direction::Left => end.1 = start.1.checked_sub (1)?,
            Direction::Down => end.0 = start.0.checked_add (1)?,
            _ => panic! ("Invalid direction {:?}", direction)
        }

        if is_in_bounds (&self.adjacencies, &end) {
            Some (end)
        } else {
            None
        }
    }

    pub fn try_move (&self, start: &Location, direction: Direction) -> Option<(Location, u8)> {
        assert! (is_rectangular (&self.adjacencies));
        assert! (is_in_bounds (&self.adjacencies, start));

        let cost: u8 = self.adjacencies[start.0][start.1][direction as usize];

        if cost > COST_IMPASSABLE {
            let mut end: Location = start.clone ();

            match direction {
                Direction::Up => end.0 = start.0.checked_sub (1)?,
                Direction::Right => end.1 = start.1.checked_add (1)?,
                Direction::Left => end.1 = start.1.checked_sub (1)?,
                Direction::Down => end.0 = start.0.checked_add (1)?,
                _ => panic! ("Invalid direction {:?}", direction)
            }

            if is_in_bounds (&self.adjacencies, &end) {
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

    pub fn update_adjacency (&mut self, location: &Location) -> () {
        assert! (is_rectangular (&self.tiles));
        assert! (is_in_bounds (&self.tiles, location));
        assert! (is_rectangular (&self.adjacencies));
        assert! (is_in_bounds (&self.adjacencies, location));

        let tile: &Tile = &self.tiles[location.0][location.1];

        for direction in DIRECTIONS {
            match self.try_connect (&location, direction) {
                Some (n) => {
                    let neighbour: &Tile = &self.tiles[n.0][n.1];
                    let cost: u8 = neighbour.find_cost (tile);

                    self.adjacencies[n.0][n.1][switch_direction (direction) as usize] = cost;
                }
                None => ()
            }
        }
    }

    pub fn place_unit (&mut self, unit_id: ID, location: Location) -> bool {
        assert! (is_in_bounds (&self.tiles, &location));
        assert! (!self.unit_locations.contains_key_first (&unit_id));

        let faction_id: &ID = self.faction_units.get_second (&unit_id).expect (&format! ("Faction not found for unit {}", unit_id));

        if self.is_placeable (&location) {
            self.unit_locations.insert ((unit_id, location));
            self.faction_locations.replace (location, *faction_id);

            true
        } else {
            false
        }
    }

    pub fn move_unit (&mut self, unit_id: ID, movements: Vec<Direction>) -> bool {
        let mut locations: Vec<Location> = Vec::new ();
        let faction_id: &ID = self.faction_units.get_second (&unit_id).expect (&format! ("Faction not found for unit {}", unit_id));
        let start: Location = self.get_unit_location (&unit_id).expect (&format! ("Location not found for unit {}", unit_id)).clone ();
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
            self.faction_locations.replace (location, *faction_id);
        }

        self.unit_locations.insert ((unit_id, end));

        true
    }

    pub fn find_unit_cities (&self, unit_id: &ID) -> Vec<ID> {
        assert! (is_rectangular (&self.tiles));

        let faction_id: &ID = self.faction_units.get_second (unit_id).expect (&format! ("Faction not found for unit {}", unit_id));
        let location: Location = self.get_unit_location (unit_id).expect (&format! ("Location not found for unit {}", unit_id)).clone ();
        let mut locations: VecDeque<Location> = VecDeque::new ();
        let mut is_visited: Vec<Vec<bool>> = vec![vec![false; self.tiles[0].len ()]; self.tiles.len ()];
        let mut city_ids: Vec<ID> = Vec::new ();

        locations.push_back (location);
        is_visited[location.0][location.1] = true;

        while locations.len () > 0 {
            let location: Location = locations.pop_front ().expect ("Location not found");

            if let Some (c) = self.tiles[location.0][location.1].get_city_id () {
                city_ids.push (c);
            }

            for direction in DIRECTIONS {
                match self.try_connect (&location, direction) {
                    Some (n) => {
                        let controller_id: &ID = self.faction_locations.get_second (&n).expect (&format! ("Faction not found for location {:?}", n));

                        if !is_visited[n.0][n.1] && controller_id == faction_id {
                            locations.push_back (n);
                            is_visited[n.0][n.1] = true;
                        }
                    }
                    None => ()
                }
            }
        }

        city_ids
    }

    pub fn find_nearby_locations (&self, location: Location, direction: Option<Direction>, area: Area, range: u8) -> Vec<Location> {
        assert! (is_rectangular (&self.tiles));
        assert! (is_in_bounds (&self.tiles, &location));

        let mut locations: HashSet<Location> = HashSet::new ();

        match area {
            Area::Single => { locations.insert (location); }
            Area::Path (w) => {
                let direction: Direction = direction.expect (&format! ("Direction not found for area {:?}", area));
                let mut starts: HashSet<Location> = HashSet::new ();

                for i in 0 ..= w {
                    let i: usize = i as usize;
                    let j: usize = i;

                    match direction {
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
                        _ => panic! ("Invalid direction {:?}", direction)
                    }
                }

                for i in 0 .. range {
                    let i: usize = i as usize;
                    let j: usize = i;

                    match direction {
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
                        _ => panic! ("Invalid direction {:?}", direction)
                    }
                }
            }
            Area::Radial (r) => {
                let r: usize = r as usize;

                for i in location.0.checked_sub (r).unwrap_or (0) ..= (location.0 + r) {
                    for j in location.1.checked_sub (r).unwrap_or (0) ..= (location.1 + r) {
                        let distance: usize = location.0.abs_diff (i) + location.1.abs_diff (j);

                        if distance <= r {
                            locations.insert ((i, j));
                        }
                    }
                }
            }
        }

        locations.retain (|l: &Location| is_in_bounds (&self.tiles, l));

        locations.into_iter ().collect::<Vec<Location>> ()
    }

    pub fn find_nearby_units (&self, location: Location, direction: Option<Direction>, area: Area, range: u8) -> Vec<ID> {
        assert! (is_rectangular (&self.tiles));
        assert! (is_in_bounds (&self.tiles, &location));

        let locations: Vec<Location> = self.find_nearby_locations (location, direction, area, range);

        locations.iter ().filter_map (|l: &Location|
            self.unit_locations.get_second (l)
            .map (|u: &ID| u.clone ())
        ).collect::<Vec<ID>> ()
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

impl Subject for Grid {
    async fn notify (&self, event: Event) -> Response {
        RESPONSE_NOTIFICATION
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
    use super::{*, super::TileBuilder};
    use crate::engine::tests::generate_lists;

    fn generate_tiles () -> Vec<Vec<Tile>> {
        let lists: Rc<Lists> = generate_lists ();
        let tile_builder: TileBuilder = TileBuilder::new (Rc::clone (&lists));

        vec![
            vec![tile_builder.build (0, 0, Some (0)), tile_builder.build (0, 1, None), tile_builder.build (0, 0, Some (1))],
            vec![tile_builder.build (1, 2, None), tile_builder.build (1, 1, None), tile_builder.build (2, 0, None)]
        ]
    }

    fn generate_unit_factions () -> HashMap<ID, ID> {
        let mut unit_factions: HashMap<ID, ID> = HashMap::new ();

        unit_factions.insert (0, 1);
        unit_factions.insert (1, 1);
        unit_factions.insert (2, 2);
        unit_factions.insert (3, 3);
        unit_factions.insert (4, 4);

        unit_factions
    }

    pub fn generate_grid () -> Grid {
        let lists: Rc<Lists> = generate_lists ();
        let tiles: Vec<Vec<Tile>> = generate_tiles ();
        let unit_factions: HashMap<ID, ID> = generate_unit_factions ();

        Grid::new (Rc::clone (&lists), tiles, unit_factions)
    }

    #[test]
    fn grid_get_cost () {
        let grid: Grid = generate_grid ();

        assert_eq! (grid.get_cost (&(0, 0), Direction::Up), 0);
        assert_eq! (grid.get_cost (&(0, 0), Direction::Right), 2);
        assert_eq! (grid.get_cost (&(0, 0), Direction::Left), 0);
        assert_eq! (grid.get_cost (&(0, 0), Direction::Down), 0);
        assert_eq! (grid.get_cost (&(0, 1), Direction::Up), 0);
        assert_eq! (grid.get_cost (&(0, 1), Direction::Right), 2);
        assert_eq! (grid.get_cost (&(0, 1), Direction::Left), 2);
        assert_eq! (grid.get_cost (&(0, 1), Direction::Down), 2);
        assert_eq! (grid.get_cost (&(0, 2), Direction::Up), 0);
        assert_eq! (grid.get_cost (&(0, 2), Direction::Right), 0);
        assert_eq! (grid.get_cost (&(0, 2), Direction::Left), 2);
        assert_eq! (grid.get_cost (&(0, 2), Direction::Down), 0);

        assert_eq! (grid.get_cost (&(1, 0), Direction::Up), 0);
        assert_eq! (grid.get_cost (&(1, 0), Direction::Right), 3);
        assert_eq! (grid.get_cost (&(1, 0), Direction::Left), 0);
        assert_eq! (grid.get_cost (&(1, 0), Direction::Down), 0);
        assert_eq! (grid.get_cost (&(1, 1), Direction::Up), 1);
        assert_eq! (grid.get_cost (&(1, 1), Direction::Right), 0);
        assert_eq! (grid.get_cost (&(1, 1), Direction::Left), 3);
        assert_eq! (grid.get_cost (&(1, 1), Direction::Down), 0);
        assert_eq! (grid.get_cost (&(1, 2), Direction::Up), 0);
        assert_eq! (grid.get_cost (&(1, 2), Direction::Right), 0);
        assert_eq! (grid.get_cost (&(1, 2), Direction::Left), 0);
        assert_eq! (grid.get_cost (&(1, 2), Direction::Down), 0);
    }

    #[test]
    fn grid_is_impassable () {
        let grid: Grid = generate_grid ();

        // Test passable
        assert_eq! (grid.is_impassable (&(0, 0)), false);
        // Test impassable
        assert_eq! (grid.is_impassable (&(1, 2)), true);
    }

    #[test]
    fn grid_is_occupied () {
        let mut grid: Grid = generate_grid ();

        // Test empty
        assert_eq! (grid.is_occupied (&(0, 0)), false);
        // Test occupied
        grid.unit_locations.insert ((0, (0, 0)));
        assert_eq! (grid.is_occupied (&(0, 0)), true);
    }

    #[test]
    fn grid_is_placeable () {
        let mut grid: Grid = generate_grid ();

        // Test passable
        assert_eq! (grid.is_placeable (&(0, 0)), true);
        assert_eq! (grid.is_placeable (&(0, 1)), true);
        assert_eq! (grid.is_placeable (&(0, 2)), true);
        assert_eq! (grid.is_placeable (&(1, 0)), true);
        assert_eq! (grid.is_placeable (&(1, 1)), true);
        // Test impassable
        assert_eq! (grid.is_placeable (&(1, 2)), false);
        // Test occupied
        grid.unit_locations.insert ((0, (0, 0)));
        assert_eq! (grid.is_placeable (&(0, 0)), false);
    }

    #[test]
    fn grid_is_movable () {
        let grid: Grid = generate_grid ();

        assert_eq! (grid.try_move (&(0, 0), Direction::Up), None);
        assert_eq! (grid.try_move (&(0, 0), Direction::Right).unwrap (), ((0, 1), 2));
        assert_eq! (grid.try_move (&(0, 0), Direction::Left), None);
        assert_eq! (grid.try_move (&(0, 0), Direction::Down), None); // Test not climbable
        assert_eq! (grid.try_move (&(0, 1), Direction::Up), None);
        assert_eq! (grid.try_move (&(0, 1), Direction::Right).unwrap (), ((0, 2), 2));
        assert_eq! (grid.try_move (&(0, 1), Direction::Left).unwrap (), ((0, 0), 2));
        assert_eq! (grid.try_move (&(0, 1), Direction::Down).unwrap (), ((1, 1), 2));
        assert_eq! (grid.try_move (&(0, 2), Direction::Up), None);
        assert_eq! (grid.try_move (&(0, 2), Direction::Right), None);
        assert_eq! (grid.try_move (&(0, 2), Direction::Left).unwrap (), ((0, 1), 2));
        assert_eq! (grid.try_move (&(0, 2), Direction::Down), None); // Test impassable

        assert_eq! (grid.try_move (&(1, 0), Direction::Up), None); // Test not climbable
        assert_eq! (grid.try_move (&(1, 0), Direction::Right).unwrap (), ((1, 1), 3));
        assert_eq! (grid.try_move (&(1, 0), Direction::Left), None);
        assert_eq! (grid.try_move (&(1, 0), Direction::Down), None);
        assert_eq! (grid.try_move (&(1, 1), Direction::Up).unwrap (), ((0, 1), 1));
        assert_eq! (grid.try_move (&(1, 1), Direction::Right), None); // Test impassable
        assert_eq! (grid.try_move (&(1, 1), Direction::Left).unwrap (), ((1, 0), 3));
        assert_eq! (grid.try_move (&(1, 1), Direction::Down), None);
        // Test impassable
        assert_eq! (grid.try_move (&(1, 2), Direction::Up), None);
        assert_eq! (grid.try_move (&(1, 2), Direction::Right), None);
        assert_eq! (grid.try_move (&(1, 2), Direction::Left), None);
        assert_eq! (grid.try_move (&(1, 2), Direction::Down), None);
    }

    #[test]
    fn grid_try_connect () {
        let grid: Grid = generate_grid ();

        assert_eq! (grid.try_connect (&(0, 0), Direction::Up), None);
        assert_eq! (grid.try_connect (&(0, 0), Direction::Right).unwrap (), (0, 1));
        assert_eq! (grid.try_connect (&(0, 0), Direction::Left), None);
        assert_eq! (grid.try_connect (&(0, 0), Direction::Down).unwrap (), (1, 0));
        assert_eq! (grid.try_connect (&(0, 1), Direction::Up), None);
        assert_eq! (grid.try_connect (&(0, 1), Direction::Right).unwrap (), (0, 2));
        assert_eq! (grid.try_connect (&(0, 1), Direction::Left).unwrap (), (0, 0));
        assert_eq! (grid.try_connect (&(0, 1), Direction::Down).unwrap (), (1, 1));
        assert_eq! (grid.try_connect (&(0, 2), Direction::Up), None);
        assert_eq! (grid.try_connect (&(0, 2), Direction::Right), None);
        assert_eq! (grid.try_connect (&(0, 2), Direction::Left).unwrap (), (0, 1));
        assert_eq! (grid.try_connect (&(0, 2), Direction::Down).unwrap (), (1, 2));

        assert_eq! (grid.try_connect (&(1, 0), Direction::Up).unwrap (), (0, 0));
        assert_eq! (grid.try_connect (&(1, 0), Direction::Right).unwrap (), (1, 1));
        assert_eq! (grid.try_connect (&(1, 0), Direction::Left), None);
        assert_eq! (grid.try_connect (&(1, 0), Direction::Down), None);
        assert_eq! (grid.try_connect (&(1, 1), Direction::Up).unwrap (), (0, 1));
        assert_eq! (grid.try_connect (&(1, 1), Direction::Right).unwrap (), (1, 2));
        assert_eq! (grid.try_connect (&(1, 1), Direction::Left).unwrap (), (1, 0));
        assert_eq! (grid.try_connect (&(1, 1), Direction::Down), None);
        assert_eq! (grid.try_connect (&(1, 2), Direction::Up).unwrap (), (0, 2));
        assert_eq! (grid.try_connect (&(1, 2), Direction::Right), None);
        assert_eq! (grid.try_connect (&(1, 2), Direction::Left).unwrap (), (1, 1));
        assert_eq! (grid.try_connect (&(1, 2), Direction::Down), None);
    }

    #[test]
    fn grid_try_move () {
        let mut grid: Grid = generate_grid ();

        // Test empty move
        assert_eq! (grid.try_move (&(0, 0), Direction::Up), None);
        assert_eq! (grid.try_move (&(0, 0), Direction::Right).unwrap (), ((0, 1), 2));
        assert_eq! (grid.try_move (&(0, 0), Direction::Left), None);
        assert_eq! (grid.try_move (&(0, 0), Direction::Down), None);
        assert_eq! (grid.try_move (&(0, 1), Direction::Up), None);
        assert_eq! (grid.try_move (&(0, 1), Direction::Right).unwrap (), ((0, 2), 2));
        assert_eq! (grid.try_move (&(0, 1), Direction::Left).unwrap (), ((0, 0), 2));
        assert_eq! (grid.try_move (&(0, 1), Direction::Down).unwrap (), ((1, 1), 2));
        assert_eq! (grid.try_move (&(0, 2), Direction::Up), None);
        assert_eq! (grid.try_move (&(0, 2), Direction::Right), None);
        assert_eq! (grid.try_move (&(0, 2), Direction::Left).unwrap (), ((0, 1), 2));
        assert_eq! (grid.try_move (&(0, 2), Direction::Down), None);

        assert_eq! (grid.try_move (&(1, 0), Direction::Up), None);
        assert_eq! (grid.try_move (&(1, 0), Direction::Right).unwrap (), ((1, 1), 3));
        assert_eq! (grid.try_move (&(1, 0), Direction::Left), None);
        assert_eq! (grid.try_move (&(1, 0), Direction::Down), None);
        assert_eq! (grid.try_move (&(1, 1), Direction::Up).unwrap (), ((0, 1), 1));
        assert_eq! (grid.try_move (&(1, 1), Direction::Right), None);
        assert_eq! (grid.try_move (&(1, 1), Direction::Left).unwrap (), ((1, 0), 3));
        assert_eq! (grid.try_move (&(1, 1), Direction::Down), None);
        assert_eq! (grid.try_move (&(1, 2), Direction::Up), None);
        assert_eq! (grid.try_move (&(1, 2), Direction::Right), None);
        assert_eq! (grid.try_move (&(1, 2), Direction::Left), None);
        assert_eq! (grid.try_move (&(1, 2), Direction::Down), None);

        // Test non-empty move
        grid.unit_locations.insert ((0, (0, 1)));
        assert_eq! (grid.try_move (&(0, 0), Direction::Right), None);
        assert_eq! (grid.try_move (&(0, 2), Direction::Left), None);
        assert_eq! (grid.try_move (&(1, 1), Direction::Up), None);
    }

    #[test]
    fn grid_update_adjacency () {
        let lists: Rc<Lists> = generate_lists ();
        let tile_builder: TileBuilder = TileBuilder::new (Rc::clone (&lists));
        let tiles_updated: Vec<Vec<Tile>> = vec![
            vec![tile_builder.build (0, 10, Some (0)) /* changed */, tile_builder.build (0, 1, None), tile_builder.build (0, 0, Some (1))],
            vec![tile_builder.build (1, 2, None), tile_builder.build (1, 1, None), tile_builder.build (0, 0, None) /* changed */]
        ];
        let mut grid: Grid = generate_grid ();

        grid.tiles = tiles_updated;
        // Test impassable update
        grid.update_adjacency (&(0, 0));
        assert_eq! (grid.try_move (&(0, 1), Direction::Left), None);
        assert_eq! (grid.try_move (&(1, 0), Direction::Up), None);
        // Test passable update
        grid.update_adjacency (&(1, 2));
        assert_eq! (grid.try_move (&(0, 2), Direction::Down).unwrap (), ((1, 2), 1));
        assert_eq! (grid.try_move (&(1, 1), Direction::Right).unwrap (), ((1, 2), 2));
    }

    #[test]
    fn grid_place_unit () {
        let mut grid: Grid = generate_grid ();

        // Test empty place
        assert_eq! (grid.place_unit (0, (0, 0)), true);
        assert_eq! (grid.faction_locations.get_second (&(0, 0)).unwrap (), &1);
        // Test impassable place
        assert_eq! (grid.place_unit (1, (1, 2)), false);
        // Test non-empty place
        assert_eq! (grid.place_unit (2, (0, 0)), false);
    }

    #[test]
    fn grid_move_unit () {
        let mut grid: Grid = generate_grid ();

        grid.place_unit (0, (0, 0));
        assert_eq! (grid.move_unit (0, vec![Direction::Up]), false); // Test out-of-bounnds
        assert_eq! (grid.move_unit (0, vec![Direction::Down]), false); // Test not climbable
        assert_eq! (grid.move_unit (0, vec![Direction::Left]), false); // Test out-of-bounds
        // Test normal move
        assert_eq! (grid.faction_locations.get_second (&(0, 1)).unwrap (), &FACTION_NONE);
        assert_eq! (grid.move_unit (0, vec![Direction::Right]), true);
        assert_eq! (grid.get_unit_location (&0).unwrap (), &(0, 1));
        assert_eq! (grid.faction_locations.get_second (&(0, 0)).unwrap (), &1);
        assert_eq! (grid.faction_locations.get_second (&(0, 1)).unwrap (), &1);
        // Test sequential move
        assert_eq! (grid.faction_locations.get_second (&(0, 2)).unwrap (), &FACTION_NONE);
        assert_eq! (grid.faction_locations.get_second (&(1, 1)).unwrap (), &FACTION_NONE);
        assert_eq! (grid.move_unit (0, vec![Direction::Right, Direction::Left, Direction::Down]), true); // Test overlap
        assert_eq! (grid.get_unit_location (&0).unwrap (), &(1, 1));
        assert_eq! (grid.faction_locations.get_second (&(0, 1)).unwrap (), &1);
        assert_eq! (grid.faction_locations.get_second (&(0, 2)).unwrap (), &1);
        assert_eq! (grid.faction_locations.get_second (&(1, 1)).unwrap (), &1);
        // Test atomic move
        assert_eq! (grid.move_unit (0, vec![Direction::Left, Direction::Right, Direction::Right]), false); // Test impassable
        assert_eq! (grid.get_unit_location (&0).unwrap (), &(1, 1));
    }

    #[test]
    fn grid_find_unit_cities () {
        let mut grid: Grid = generate_grid ();

        // Test no supply
        grid.place_unit (0, (0, 1));
        assert_eq! (grid.find_unit_cities (&0).len (), 0);
        grid.move_unit (0, vec![Direction::Down, Direction::Left]);
        assert_eq! (grid.find_unit_cities (&0).len (), 0);
        // Test contested supply
        grid.place_unit (2, (0, 0));
        assert_eq! (grid.find_unit_cities (&0).len (), 0);
        assert_eq! (grid.find_unit_cities (&2).len (), 1);
        // Test normal supply
        grid.place_unit (1, (0, 2));
        assert_eq! (grid.find_unit_cities (&0).len (), 1);
        assert_eq! (grid.find_unit_cities (&1).len (), 1);
        assert_eq! (grid.find_unit_cities (&2).len (), 1);
        // Test multiple supply
        grid.faction_locations.replace ((0, 0), 1);
        assert_eq! (grid.find_unit_cities (&0).len (), 2);
        assert_eq! (grid.find_unit_cities (&1).len (), 2);
        assert_eq! (grid.find_unit_cities (&2).len (), 1);
    }

    #[test]
    fn grid_find_nearby_locations () {
        let mut grid: Grid = generate_grid ();

        assert_eq! (grid.find_nearby_locations ((0, 0), None, Area::Single, 0).len (), 1);
        assert_eq! (grid.find_nearby_locations ((0, 1), None, Area::Single, 0).len (), 1);
        assert_eq! (grid.find_nearby_locations ((0, 2), None, Area::Single, 0).len (), 1);
        assert_eq! (grid.find_nearby_locations ((1, 0), None, Area::Single, 0).len (), 1);
        assert_eq! (grid.find_nearby_locations ((1, 1), None, Area::Single, 0).len (), 1);
        assert_eq! (grid.find_nearby_locations ((1, 2), None, Area::Single, 0).len (), 1);

        assert_eq! (grid.find_nearby_locations ((0, 0), Some (Direction::Right), Area::Path (1), 1).len (), 2);
        assert_eq! (grid.find_nearby_locations ((0, 1), Some (Direction::Down), Area::Path (1), 1).len (), 3);
        assert_eq! (grid.find_nearby_locations ((0, 2), Some (Direction::Left), Area::Path (1), 1).len (), 2);
        assert_eq! (grid.find_nearby_locations ((1, 0), Some (Direction::Right), Area::Path (1), 1).len (), 2);
        assert_eq! (grid.find_nearby_locations ((1, 1), Some (Direction::Up), Area::Path (1), 1).len (), 3);
        assert_eq! (grid.find_nearby_locations ((1, 2), Some (Direction::Left), Area::Path (1), 1).len (), 2);

        assert_eq! (grid.find_nearby_locations ((0, 0), Some (Direction::Right), Area::Path (0), 2).len (), 2);
        assert_eq! (grid.find_nearby_locations ((0, 1), Some (Direction::Down), Area::Path (0), 2).len (), 1);
        assert_eq! (grid.find_nearby_locations ((0, 2), Some (Direction::Left), Area::Path (0), 2).len (), 2);
        assert_eq! (grid.find_nearby_locations ((1, 0), Some (Direction::Right), Area::Path (0), 2).len (), 2);
        assert_eq! (grid.find_nearby_locations ((1, 1), Some (Direction::Up), Area::Path (0), 2).len (), 1);
        assert_eq! (grid.find_nearby_locations ((1, 2), Some (Direction::Left), Area::Path (0), 2).len (), 2);

        assert_eq! (grid.find_nearby_locations ((0, 0), None, Area::Radial (1), 0).len (), 3);
        assert_eq! (grid.find_nearby_locations ((0, 1), None, Area::Radial (1), 0).len (), 4);
        assert_eq! (grid.find_nearby_locations ((0, 2), None, Area::Radial (1), 0).len (), 3);
        assert_eq! (grid.find_nearby_locations ((1, 0), None, Area::Radial (1), 0).len (), 3);
        assert_eq! (grid.find_nearby_locations ((1, 1), None, Area::Radial (1), 0).len (), 4);
        assert_eq! (grid.find_nearby_locations ((1, 2), None, Area::Radial (1), 0).len (), 3);

        assert_eq! (grid.find_nearby_locations ((0, 0), None, Area::Radial (2), 0).len (), 5);
        assert_eq! (grid.find_nearby_locations ((0, 1), None, Area::Radial (2), 0).len (), 6);
        assert_eq! (grid.find_nearby_locations ((0, 2), None, Area::Radial (2), 0).len (), 5);
        assert_eq! (grid.find_nearby_locations ((1, 0), None, Area::Radial (2), 0).len (), 5);
        assert_eq! (grid.find_nearby_locations ((1, 1), None, Area::Radial (2), 0).len (), 6);
        assert_eq! (grid.find_nearby_locations ((1, 2), None, Area::Radial (2), 0).len (), 5);
    }

    #[test]
    fn grid_find_nearby_units () {
        let mut grid: Grid = generate_grid ();

        // Test empty find
        assert_eq! (grid.find_nearby_units ((0, 0), None, Area::Single, 0).len (), 0);
        assert_eq! (grid.find_nearby_units ((0, 1), None, Area::Single, 0).len (), 0);
        assert_eq! (grid.find_nearby_units ((0, 2), None, Area::Single, 0).len (), 0);
        assert_eq! (grid.find_nearby_units ((1, 0), None, Area::Single, 0).len (), 0);
        assert_eq! (grid.find_nearby_units ((1, 1), None, Area::Single, 0).len (), 0);
        assert_eq! (grid.find_nearby_units ((1, 2), None, Area::Single, 0).len (), 0);

        assert_eq! (grid.find_nearby_units ((0, 0), Some (Direction::Right), Area::Path (1), 1).len (), 0);
        assert_eq! (grid.find_nearby_units ((0, 1), Some (Direction::Down), Area::Path (1), 1).len (), 0);
        assert_eq! (grid.find_nearby_units ((0, 2), Some (Direction::Left), Area::Path (1), 1).len (), 0);
        assert_eq! (grid.find_nearby_units ((1, 0), Some (Direction::Right), Area::Path (1), 1).len (), 0);
        assert_eq! (grid.find_nearby_units ((1, 1), Some (Direction::Up), Area::Path (1), 1).len (), 0);
        assert_eq! (grid.find_nearby_units ((1, 2), Some (Direction::Left), Area::Path (1), 1).len (), 0);

        assert_eq! (grid.find_nearby_units ((0, 0), Some (Direction::Right), Area::Path (0), 2).len (), 0);
        assert_eq! (grid.find_nearby_units ((0, 1), Some (Direction::Down), Area::Path (0), 2).len (), 0);
        assert_eq! (grid.find_nearby_units ((0, 2), Some (Direction::Left), Area::Path (0), 2).len (), 0);
        assert_eq! (grid.find_nearby_units ((1, 0), Some (Direction::Right), Area::Path (0), 2).len (), 0);
        assert_eq! (grid.find_nearby_units ((1, 1), Some (Direction::Up), Area::Path (0), 2).len (), 0);
        assert_eq! (grid.find_nearby_units ((1, 2), Some (Direction::Left), Area::Path (0), 2).len (), 0);

        assert_eq! (grid.find_nearby_units ((0, 0), None, Area::Radial (1), 0).len (), 0);
        assert_eq! (grid.find_nearby_units ((0, 1), None, Area::Radial (1), 0).len (), 0);
        assert_eq! (grid.find_nearby_units ((0, 2), None, Area::Radial (1), 0).len (), 0);
        assert_eq! (grid.find_nearby_units ((1, 0), None, Area::Radial (1), 0).len (), 0);
        assert_eq! (grid.find_nearby_units ((1, 1), None, Area::Radial (1), 0).len (), 0);
        assert_eq! (grid.find_nearby_units ((1, 2), None, Area::Radial (1), 0).len (), 0);

        assert_eq! (grid.find_nearby_units ((0, 0), None, Area::Radial (2), 0).len (), 0);
        assert_eq! (grid.find_nearby_units ((0, 1), None, Area::Radial (2), 0).len (), 0);
        assert_eq! (grid.find_nearby_units ((0, 2), None, Area::Radial (2), 0).len (), 0);
        assert_eq! (grid.find_nearby_units ((1, 0), None, Area::Radial (2), 0).len (), 0);
        assert_eq! (grid.find_nearby_units ((1, 1), None, Area::Radial (2), 0).len (), 0);
        assert_eq! (grid.find_nearby_units ((1, 2), None, Area::Radial (2), 0).len (), 0);
        // Test non-empty find
        grid.place_unit (0, (0, 0));
        grid.place_unit (1, (0, 1));
        grid.place_unit (2, (0, 2));
        grid.place_unit (3, (1, 0));
        grid.place_unit (4, (1, 1));
        assert_eq! (grid.find_nearby_units ((0, 0), None, Area::Single, 0).len (), 1);
        assert_eq! (grid.find_nearby_units ((0, 1), None, Area::Single, 0).len (), 1);
        assert_eq! (grid.find_nearby_units ((0, 2), None, Area::Single, 0).len (), 1);
        assert_eq! (grid.find_nearby_units ((1, 0), None, Area::Single, 0).len (), 1);
        assert_eq! (grid.find_nearby_units ((1, 1), None, Area::Single, 0).len (), 1);
        assert_eq! (grid.find_nearby_units ((1, 2), None, Area::Single, 0).len (), 0);

        assert_eq! (grid.find_nearby_units ((0, 0), Some (Direction::Right), Area::Path (1), 1).len (), 2);
        assert_eq! (grid.find_nearby_units ((0, 1), Some (Direction::Down), Area::Path (1), 1).len (), 2);
        assert_eq! (grid.find_nearby_units ((0, 2), Some (Direction::Left), Area::Path (1), 1).len (), 2);
        assert_eq! (grid.find_nearby_units ((1, 0), Some (Direction::Right), Area::Path (1), 1).len (), 2);
        assert_eq! (grid.find_nearby_units ((1, 1), Some (Direction::Up), Area::Path (1), 1).len (), 3);
        assert_eq! (grid.find_nearby_units ((1, 2), Some (Direction::Left), Area::Path (1), 1).len (), 2);

        assert_eq! (grid.find_nearby_units ((0, 0), Some (Direction::Right), Area::Path (0), 2).len (), 2);
        assert_eq! (grid.find_nearby_units ((0, 1), Some (Direction::Down), Area::Path (0), 2).len (), 1);
        assert_eq! (grid.find_nearby_units ((0, 2), Some (Direction::Left), Area::Path (0), 2).len (), 2);
        assert_eq! (grid.find_nearby_units ((1, 0), Some (Direction::Right), Area::Path (0), 2).len (), 1);
        assert_eq! (grid.find_nearby_units ((1, 1), Some (Direction::Up), Area::Path (0), 2).len (), 1);
        assert_eq! (grid.find_nearby_units ((1, 2), Some (Direction::Left), Area::Path (0), 2).len (), 2);

        assert_eq! (grid.find_nearby_units ((0, 0), None, Area::Radial (1), 0).len (), 3);
        assert_eq! (grid.find_nearby_units ((0, 1), None, Area::Radial (1), 0).len (), 4);
        assert_eq! (grid.find_nearby_units ((0, 2), None, Area::Radial (1), 0).len (), 2);
        assert_eq! (grid.find_nearby_units ((1, 0), None, Area::Radial (1), 0).len (), 3);
        assert_eq! (grid.find_nearby_units ((1, 1), None, Area::Radial (1), 0).len (), 3);
        assert_eq! (grid.find_nearby_units ((1, 2), None, Area::Radial (1), 0).len (), 2);

        assert_eq! (grid.find_nearby_units ((0, 0), None, Area::Radial (2), 0).len (), 5);
        assert_eq! (grid.find_nearby_units ((0, 1), None, Area::Radial (2), 0).len (), 5);
        assert_eq! (grid.find_nearby_units ((0, 2), None, Area::Radial (2), 0).len (), 4);
        assert_eq! (grid.find_nearby_units ((1, 0), None, Area::Radial (2), 0).len (), 4);
        assert_eq! (grid.find_nearby_units ((1, 1), None, Area::Radial (2), 0).len (), 5);
        assert_eq! (grid.find_nearby_units ((1, 2), None, Area::Radial (2), 0).len (), 4);
    }
}
