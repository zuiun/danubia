use super::{City, COST_IMPASSABLE, Search, Tile, TileBuilder};
use crate::collections::{InnerJoinMap, OuterJoinMap};
use crate::common::{ID, ID_UNINITIALISED, Scene};
use crate::dynamic::{Appliable, Applier, Dynamic, Modifier, AppliableKind};
use std::collections::{HashSet, VecDeque};
use std::fmt::{self, Display, Formatter};
use std::rc::Rc;

pub type Location = (usize, usize); // row, column
type Rectangle<T> = Vec<Vec<T>>;
type Row<T> = Vec<T>;
type Adjacency = [u8; Direction::Length as usize]; // cost, climb

const DIRECTIONS: [Direction; Direction::Length as usize] = [Direction::Up, Direction::Right, Direction::Left, Direction::Down];
const FACTION_UNCONTROLLED: ID = ID_UNINITIALISED;

const fn switch_direction (direction: Direction) -> Direction {
    DIRECTIONS[(Direction::Length as usize) - (direction as usize) - 1]
}

fn is_rectangular<T> (grid: &Rectangle<T>) -> bool {
    assert! (!grid.is_empty ());
    assert! (!grid[0].is_empty ());

    grid.iter ().all (|r: &Row<T>| r.len () == grid[0].len ())
}

fn is_in_bounds<T> (grid: &Rectangle<T>, location: &Location) -> bool {
    assert! (!grid.is_empty ());
    assert! (!grid[0].is_empty ());

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
    scene: Rc<Scene>,
    tiles: Rectangle<Tile>,
    adjacencies: Rectangle<Adjacency>,
    unit_locations: InnerJoinMap<ID, Location>,
    faction_locations: OuterJoinMap<ID, Location>,
    unit_id_passable: Option<ID>,
}

impl Grid {
    pub fn new (scene: Rc<Scene>) -> Self {
        let tile_builders: &[&[TileBuilder]] = scene.get_tile_builders ();
        let mut tiles: Rectangle<Tile> = Rectangle::new ();

        for (i, row) in tile_builders.iter ().enumerate () {
            tiles.push (Row::new ());

            for tile_builder in row.iter () {
                let tile: Tile = tile_builder.build (Rc::clone (&scene));

                tiles[i].push (tile);
            }
        }

        let adjacencies: Rectangle<Adjacency> = Grid::build_adjacencies (&tiles);
        let mut faction_locations: OuterJoinMap<ID, Location> = OuterJoinMap::new ();
        let unit_locations: InnerJoinMap<ID, Location> = InnerJoinMap::new ();
        let unit_id_passable: Option<ID> = None;

        for i in 0 .. tiles.len () {
            for j in 0 .. tiles[0].len () {
                faction_locations.insert ((FACTION_UNCONTROLLED, (i, j)));
            }
        }

        assert! (is_rectangular (&tiles));
        assert! (is_rectangular (&adjacencies));

        Self { scene, tiles, adjacencies, unit_locations, faction_locations, unit_id_passable }
    }

    fn build_adjacencies (tiles: &Rectangle<Tile>) -> Rectangle<Adjacency> {
        assert! (is_rectangular (tiles));

        let mut adjacencies: Rectangle<Adjacency> = Vec::new ();

        for (i, row) in tiles.iter ().enumerate () {
            adjacencies.push (Row::new ());

            for (j, tile) in row.iter ().enumerate () {
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

        match self.get_location_unit (location) {
            Some (u) => if let Some (p) = self.unit_id_passable {
                *u != p
            } else {
                true
            }
            None => false
        }
    }

    fn is_placeable (&self, location: &Location) -> bool {
        assert! (is_in_bounds (&self.tiles, location));

        !self.is_impassable (location) && !self.is_occupied (location)
    }

    pub fn find_nearest_placeable (&self, location: &Location) -> Location {
        assert! (is_rectangular (&self.tiles));
        assert! (is_in_bounds (&self.tiles, location));

        let mut is_visited: Rectangle<bool> = vec![vec![false; self.tiles[0].len ()]; self.tiles.len ()];
        let mut locations: VecDeque<Location> = VecDeque::new ();

        locations.push_back (*location);
        is_visited[location.0][location.1] = true;

        while let Some (location) = locations.pop_front () {
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

    pub fn find_distance_between (&self, unit_id_first: &ID, unit_id_second: &ID) -> usize {
        let location_first: &Location = self.get_unit_location (unit_id_first);
        let location_second: &Location = self.get_unit_location (unit_id_second);

        location_first.0.abs_diff (location_second.0) + location_first.1.abs_diff (location_second.1)
    }

    pub fn try_connect (&self, start: &Location, direction: Direction) -> Option<Location> {
        assert! (is_rectangular (&self.adjacencies));
        assert! (is_in_bounds (&self.adjacencies, start));

        let mut end: Location = *start;

        match direction {
            Direction::Up => end.0 = start.0.checked_sub (1)?,
            Direction::Right => end.1 = start.1.checked_add (1)?,
            Direction::Left => end.1 = start.1.checked_sub (1)?,
            Direction::Down => end.0 = start.0.checked_add (1)?,
            _ => panic! ("Invalid direction {:?}", direction),
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
            let mut end: Location = *start;

            match direction {
                Direction::Up => end.0 = start.0.checked_sub (1)?,
                Direction::Right => end.1 = start.1.checked_add (1)?,
                Direction::Left => end.1 = start.1.checked_sub (1)?,
                Direction::Down => end.0 = start.0.checked_add (1)?,
                _ => panic! ("Invalid direction {:?}", direction),
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

    pub fn update_adjacency (&mut self, location: &Location) {
        assert! (is_rectangular (&self.tiles));
        assert! (is_in_bounds (&self.tiles, location));
        assert! (is_rectangular (&self.adjacencies));
        assert! (is_in_bounds (&self.adjacencies, location));

        let tile: &Tile = &self.tiles[location.0][location.1];

        for direction in DIRECTIONS {
            if let Some (n) = self.try_connect (location, direction) {
                let neighbour: &Tile = &self.tiles[n.0][n.1];
                let cost: u8 = neighbour.find_cost (tile);

                self.adjacencies[n.0][n.1][switch_direction (direction) as usize] = cost;
            }
        }
    }

    pub fn place_unit (&mut self, unit_id: ID, location: Location) -> Option<ID> {
        assert! (is_in_bounds (&self.tiles, &location));
        assert! (!self.unit_locations.contains_key_first (&unit_id));

        let faction_id: ID = self.get_unit_faction (&unit_id);

        if self.is_placeable (&location) {
            self.unit_locations.insert ((unit_id, location));
            self.faction_locations.replace (location, faction_id);

            Some (self.tiles[location.0][location.1].get_terrain_id ())
        } else {
            None
        }
    }

    pub fn remove_unit (&mut self, unit_id: &ID) {
        self.unit_locations.remove_first (unit_id);
    }

    pub fn move_unit (&mut self, unit_id: ID, movements: &[Direction]) -> Option<(Location, ID)> {
        let mut locations: Vec<Location> = Vec::new ();
        let faction_id: ID = self.get_unit_faction (&unit_id);
        let start: Location = *self.get_unit_location (&unit_id);
        let mut end: Location = start;

        // Temporarily remove unit
        // self.unit_locations.remove_first (&unit_id);
        self.unit_id_passable = Some (unit_id);

        for direction in movements.iter () {
            end = match self.try_move (&end, *direction) {
                Some (e) => e.0,
                None => {
                    // TODO: This is probably worth a panic
                    // self.unit_locations.insert ((unit_id, end));
                    self.unit_locations.replace_first (unit_id, end);
                    self.unit_id_passable = None;

                    return None
                }
            };
            locations.push (end);
        }

        for location in locations {
            self.faction_locations.replace (location, faction_id);
        }

        let terrain_id: ID = self.tiles[end.0][end.1].get_terrain_id ();

        // self.unit_locations.insert ((unit_id, end));
        self.unit_locations.replace_first (unit_id, end);
        self.unit_id_passable = None;

        Some ((end, terrain_id))
    }

    fn find_unit_movable_helper (&self, is_visited: &mut Rectangle<bool>, location: &Location, mov: u16) {
        is_visited[location.0][location.1] = true;

        for direction in DIRECTIONS {
            if let Some ((location, cost)) = self.try_move (location, direction) {
                if let Some (mov) = mov.checked_sub (cost as u16) {
                    self.find_unit_movable_helper (is_visited, &location, mov);
                }
            }
        }
    }

    pub fn find_unit_movable (&self, unit_id: &ID, mov: u16) -> Vec<Location> {
        let location: Location = *self.get_unit_location (unit_id);
        let mut is_visited: Rectangle<bool> = vec![vec![false; self.tiles[0].len ()]; self.tiles.len ()];
        let mut locations: Vec<Location> = Vec::new ();

        self.find_unit_movable_helper (&mut is_visited, &location, mov);

        for (i, row) in is_visited.iter ().enumerate () {
            for (j, is_visited) in row.iter ().enumerate () {
                if *is_visited {
                    locations.push ((i, j))
                }
            }
        }

        locations
        // is_visited.iter ().enumerate ().map (|(i, row)| {
        //     row.iter ().enumerate ().filter_map (move |(j, is_visited)|
        //         is_visited.then_some ((i, j))
        //     )
        // }).flatten ().collect ()
    }

    pub fn find_unit_cities (&self, unit_id: &ID) -> Vec<ID> {
        assert! (is_rectangular (&self.tiles));

        let location: Location = *self.get_unit_location (unit_id);
        let faction_id: ID = self.get_unit_faction (unit_id);
        let mut locations: VecDeque<Location> = VecDeque::new ();
        let mut is_visited: Rectangle<bool> = vec![vec![false; self.tiles[0].len ()]; self.tiles.len ()];
        let mut city_ids: Vec<ID> = Vec::new ();

        locations.push_back (location);
        is_visited[location.0][location.1] = true;

        while let Some (location) = locations.pop_front () {
            if let Some (c) = self.tiles[location.0][location.1].get_city_id () {
                city_ids.push (c);
            }

            for direction in DIRECTIONS {
                if let Some (n) = self.try_connect (&location, direction) {
                    let controller_id: &ID = self.get_location_faction (&n);

                    if !is_visited[n.0][n.1] && *controller_id == faction_id {
                        locations.push_back (n);
                        is_visited[n.0][n.1] = true;
                    }
                }
            }
        }

        city_ids
    }

    pub fn find_locations (&self, location: &Location, search: Search) -> Vec<Location> {
        assert! (is_rectangular (&self.tiles));
        assert! (is_in_bounds (&self.tiles, location));

        let mut locations: HashSet<Location> = HashSet::new ();
        let location: Location = *location;

        match search {
            Search::Single => { locations.insert (location); }
            Search::Radial (r) => {
                let range: usize = r as usize;

                for i in location.0.saturating_sub (range) ..= (location.0 + range) {
                    for j in location.1.saturating_sub (range) ..= (location.1 + range) {
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
                        Direction::Up => if let Some (i) = location.0.checked_sub (1) {
                            let left: Location = (i, location.1.saturating_sub (j));
                            let right: Location = (i, location.1 + j);

                            starts.insert (left);
                            starts.insert (right);
                        } else {
                            break
                        }
                        Direction::Right => {
                            let j: usize = location.1 + 1;
                            let up: Location = (location.0.saturating_sub (i), j);
                            let down: Location = (location.0 + i, j);

                            starts.insert (up);
                            starts.insert (down);
                        }
                        Direction::Left => if let Some (j) = location.1.checked_sub (1) {
                            let up: Location = (location.0.saturating_sub (i), j);
                            let down: Location = (location.0 + i, j);

                            starts.insert (up);
                            starts.insert (down);
                        } else {
                            break
                        }
                        Direction::Down => {
                            let i: usize = location.0 + 1;
                            let left: Location = (i, location.1.saturating_sub (j));
                            let right: Location = (i, location.1 + j);

                            starts.insert (left);
                            starts.insert (right);
                        }
                        _ => panic! ("Invalid direction {:?}", d),
                    }
                }

                for i in 0 .. r {
                    let (i, j): (usize, usize) = (i as usize, i as usize);

                    match d {
                        Direction::Up => locations.extend (
                            starts.iter ().map (|l: &Location| (l.0.saturating_sub (i), l.1))
                        ),
                        Direction::Right => locations.extend (
                            starts.iter ().map (|l: &Location| (l.0, l.1 + j))
                        ),
                        Direction::Left => locations.extend (
                            starts.iter ().map (|l: &Location| (l.0, l.1.saturating_sub (j)))
                        ),
                        Direction::Down => locations.extend (
                            starts.iter ().map (|l: &Location| (l.0 + i, l.1))
                        ),
                        _ => panic! ("Invalid direction {:?}", d),
                    }
                }
            }
        }

        locations.retain (|l: &Location| is_in_bounds (&self.tiles, l));

        locations.into_iter ().collect::<Vec<Location>> ()
    }

    pub fn find_units (&self, location: &Location, search: Search) -> Vec<ID> {
        assert! (is_rectangular (&self.tiles));
        assert! (is_in_bounds (&self.tiles, location));

        let locations: Vec<Location> = self.find_locations (location, search);

        locations.iter ().filter_map (|l: &Location|
            self.get_location_unit (l).copied ()
        ).collect::<Vec<ID>> ()
    }

    pub fn add_appliable (&mut self, location: &Location, appliable: Box<dyn Appliable>) -> bool {
        assert! (is_rectangular (&self.tiles));
        assert! (is_in_bounds (&self.tiles, location));

        let kind: AppliableKind = appliable.kind ();
        let is_added: bool = self.tiles[location.0][location.1].add_appliable (appliable);

        if let AppliableKind::Modifier ( .. ) = kind {
            self.update_adjacency (location);
        }

        is_added
    }

    pub fn try_spawn_recruit (&mut self, location: Location, faction_id: &ID) -> Option<(ID, ID)> {
        assert! (is_rectangular (&self.tiles));
        assert! (is_in_bounds (&self.tiles, &location));

        let tile: &Tile = &self.tiles[location.0][location.1];

        // TODO: This whole chain of None looks goofy
        if let Some (city_id) = tile.get_city_id () {
            if !tile.is_recruited () {
                let city: &City = self.scene.get_city (&city_id);

                if let Some (recruit_id) = city.get_recruit_id () {
                    let faction_id_recruit: ID = self.get_unit_faction (&recruit_id);

                    if faction_id_recruit == *faction_id {
                        let spawn: Location = self.find_nearest_placeable (&location);
                        let terrain_id: ID = self.place_unit (recruit_id, spawn)?;

                        self.tiles[location.0][location.1].set_recruited (true);

                        Some ((recruit_id, terrain_id))
                    } else {
                        None
                    }
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

    pub fn find_locations_supplied (&self, unit_id: &ID) -> Vec<Location> {
        assert! (is_rectangular (&self.tiles));

        // Find all locations connected to a city controlled by faction_id
        let location: Location = *self.get_unit_location (unit_id);
        let faction_id: ID = self.get_unit_faction (unit_id);
        let mut locations: VecDeque<Location> = VecDeque::new ();
        let mut is_visited: Rectangle<bool> = vec![vec![false; self.tiles[0].len ()]; self.tiles.len ()];
        let mut controlled: Vec<Location> = Vec::new ();
        let mut has_city: bool = self.tiles[location.0][location.1].get_city_id ().is_some ();

        locations.push_back (location);
        controlled.push (location);
        is_visited[location.0][location.1] = true;

        while let Some (location) = locations.pop_front () {
            for direction in DIRECTIONS {
                if let Some (n) = self.try_connect (&location, direction) {
                    let controller_id: ID = *self.get_location_faction (&n);

                    if !is_visited[n.0][n.1] && controller_id == faction_id {
                        locations.push_back (n);
                        controlled.push (n);
                        is_visited[n.0][n.1] = true;

                        if self.tiles[n.0][n.1].get_city_id ().is_some () {
                            has_city = true;
                        }
                    }
                }
            }
        }

        if has_city {
            controlled
        } else {
            Vec::new ()
        }
    }

    pub fn expand_control (&mut self, unit_id: &ID) {
        assert! (is_rectangular (&self.tiles));

        let faction_id: ID = self.get_unit_faction (unit_id);
        let controlled: Vec<Location> = self.find_locations_supplied (unit_id);
        let mut is_visited: Rectangle<bool> = vec![vec![false; self.tiles[0].len ()]; self.tiles.len ()];
        let mut uncontrolled: Vec<Location> = Vec::new ();

        for location in controlled.iter () {
            is_visited[location.0][location.1] = true;
        }

        for location in controlled {
            for direction in DIRECTIONS {
                let occupation: Option<Location> = self.try_connect (&location, direction);

                if let Some (o) = occupation {
                    if !is_visited[o.0][o.1]
                            && self.tiles[o.0][o.1].get_city_id ().is_none ()
                            && self.get_location_unit (&o).is_none () {
                        uncontrolled.push (o);
                        is_visited[o.0][o.1] = true;
                    }
                }
            }
        }

        for occupation in uncontrolled {
            self.faction_locations.replace (occupation, faction_id);
        }
    }

    pub fn decrement_durations (&mut self, unit_id: &ID) {
        for row in self.tiles.iter_mut () {
            for tile in row.iter_mut () {
                if let Some (a) = tile.get_applier_id_modifier () {
                    if a == *unit_id {
                        tile.decrement_durations ();
                    }
                } else if let Some (a) = tile.get_applier_id_attribute () {
                    if a == *unit_id {
                        tile.decrement_durations ();
                    }
                }
            }
        }
    }

    pub fn get_city_id (&self, location: &Location) -> Option<ID> {
        assert! (is_rectangular (&self.tiles));
        assert! (is_in_bounds (&self.tiles, location));

        self.tiles[location.0][location.1].get_city_id ()
    }

    pub fn get_terrain_id (&self, location: &Location) -> ID {
        assert! (is_rectangular (&self.tiles));
        assert! (is_in_bounds (&self.tiles, location));

        self.tiles[location.0][location.1].get_terrain_id ()
    }

    pub fn get_modifier (&self, location: &Location) -> Option<Modifier> {
        assert! (is_rectangular (&self.tiles));
        assert! (is_in_bounds (&self.tiles, location));

        self.tiles[location.0][location.1].get_modifier ()
    }

    pub fn try_yield_appliable (&self, location: &Location) -> Option<Box<dyn Appliable>> {
        assert! (is_rectangular (&self.tiles));
        assert! (is_in_bounds (&self.tiles, location));

        self.tiles[location.0][location.1].try_yield_appliable (Rc::clone (&self.scene))
    }

    pub fn is_recruited (&self, location: &Location) -> bool {
        assert! (is_rectangular (&self.tiles));
        assert! (is_in_bounds (&self.tiles, location));

        self.tiles[location.0][location.1].is_recruited ()
    }

    pub fn get_unit_location (&self, unit_id: &ID) -> &Location {
        self.unit_locations.get_first (unit_id)
                .unwrap_or_else (|| panic! ("Location not found for unit {}", unit_id))
    }

    pub fn get_location_unit (&self, location: &Location) -> Option<&ID> {
        assert! (is_in_bounds (&self.tiles, location));

        self.unit_locations.get_second (location)
    }

    pub fn get_faction_locations (&self, faction_id: &ID) -> Option<&HashSet<Location>> {
        self.faction_locations.get_first (faction_id)
    }

    pub fn get_location_faction (&self, location: &Location) -> &ID {
        assert! (is_in_bounds (&self.tiles, location));

        self.faction_locations.get_second (location)
                .unwrap_or_else (|| panic! ("Faction not found for location {:?}", location))
    }

    fn get_unit_faction (&self, unit_id: &ID) -> ID {
        self.scene.get_unit_builder (unit_id).get_faction_id ()
    }

    pub fn set_unit_id_passable (&mut self, unit_id_passable: Option<ID>) {
        self.unit_id_passable = unit_id_passable;
    }
}

impl Display for Grid {
    fn fmt (&self, f: &mut Formatter) -> fmt::Result {
        let mut display: String = String::from ("");

        for (i, row) in self.tiles.iter ().enumerate () {
            for (j, tile) in row.iter ().enumerate () {
                if self.is_occupied (&(i, j)) {
                    display.push_str (&format! ("{}u{}h ",
                            self.get_location_unit (&(i, j))
                                    .unwrap_or_else (|| panic! ("Unit not found for location ({}, {})", i, j)),
                            tile.get_height ()));
                } else {
                    display.push_str (&format! ("{}_{}h ", tile.get_cost (), tile.get_height ()));
                }
            }

            display.push ('\n');
        }

        write! (f, "{}", display)
    }
}

#[cfg (test)]
mod tests {
    use super::*;
    use crate::tests::generate_scene;

    fn generate_grid () -> Grid {
        let scene = generate_scene ();

        Grid::new (Rc::clone (&scene))
    }

    #[test]
    fn grid_get_cost () {
        let grid = generate_grid ();

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
        let grid = generate_grid ();

        // Test passable
        assert! (!grid.is_impassable (&(0, 0)));
        // Test impassable
        assert! (grid.is_impassable (&(1, 2)));
    }

    #[test]
    fn grid_is_occupied () {
        let mut grid = generate_grid ();

        // Test empty
        assert! (!grid.is_occupied (&(0, 0)));
        // Test occupied
        grid.unit_locations.insert ((0, (0, 0)));
        assert! (grid.is_occupied (&(0, 0)));
    }

    #[test]
    fn grid_is_placeable () {
        let mut grid = generate_grid ();

        // Test passable
        assert! (grid.is_placeable (&(0, 0)));
        assert! (grid.is_placeable (&(0, 1)));
        assert! (grid.is_placeable (&(0, 2)));
        assert! (grid.is_placeable (&(1, 0)));
        assert! (grid.is_placeable (&(1, 1)));
        // Test impassable
        assert! (!grid.is_placeable (&(1, 2)));
        // Test occupied
        grid.unit_locations.insert ((0, (0, 0)));
        assert! (!grid.is_placeable (&(0, 0)));
    }

    #[test]
    fn grid_find_nearest_placeable () {
        let mut grid = generate_grid ();

        // Test empty find
        assert_eq! (grid.find_nearest_placeable (&(0, 0)), (0, 0));
        // Test non-empty find
        grid.place_unit (0, (0, 0));
        assert! (grid.find_nearest_placeable (&(0, 0)) == (0, 1)
            || grid.find_nearest_placeable (&(0, 0)) == (1, 0)
        );
    }

    #[test]
    fn grid_find_distance_between () {
        let mut grid = generate_grid ();

        grid.place_unit (0, (0, 0));
        grid.place_unit (1, (0, 1));
        grid.place_unit (2, (1, 1));
        assert_eq! (grid.find_distance_between (&0, &1), 1);
        assert_eq! (grid.find_distance_between (&1, &0), 1);
        assert_eq! (grid.find_distance_between (&0, &2), 2);
        assert_eq! (grid.find_distance_between (&2, &0), 2);
        assert_eq! (grid.find_distance_between (&1, &2), 1);
        assert_eq! (grid.find_distance_between (&2, &1), 1);
    }

    #[test]
    fn grid_try_connect () {
        let grid = generate_grid ();

        assert! (grid.try_connect (&(0, 0), Direction::Up).is_none ());
        assert_eq! (grid.try_connect (&(0, 0), Direction::Right).unwrap (), (0, 1));
        assert! (grid.try_connect (&(0, 0), Direction::Left).is_none ());
        assert_eq! (grid.try_connect (&(0, 0), Direction::Down).unwrap (), (1, 0));
        assert! (grid.try_connect (&(0, 1), Direction::Up).is_none ());
        assert_eq! (grid.try_connect (&(0, 1), Direction::Right).unwrap (), (0, 2));
        assert_eq! (grid.try_connect (&(0, 1), Direction::Left).unwrap (), (0, 0));
        assert_eq! (grid.try_connect (&(0, 1), Direction::Down).unwrap (), (1, 1));
        assert! (grid.try_connect (&(0, 2), Direction::Up).is_none ());
        assert! (grid.try_connect (&(0, 2), Direction::Right).is_none ());
        assert_eq! (grid.try_connect (&(0, 2), Direction::Left).unwrap (), (0, 1));
        assert_eq! (grid.try_connect (&(0, 2), Direction::Down).unwrap (), (1, 2));

        assert_eq! (grid.try_connect (&(1, 0), Direction::Up).unwrap (), (0, 0));
        assert_eq! (grid.try_connect (&(1, 0), Direction::Right).unwrap (), (1, 1));
        assert! (grid.try_connect (&(1, 0), Direction::Left).is_none ());
        assert! (grid.try_connect (&(1, 0), Direction::Down).is_none ());
        assert_eq! (grid.try_connect (&(1, 1), Direction::Up).unwrap (), (0, 1));
        assert_eq! (grid.try_connect (&(1, 1), Direction::Right).unwrap (), (1, 2));
        assert_eq! (grid.try_connect (&(1, 1), Direction::Left).unwrap (), (1, 0));
        assert! (grid.try_connect (&(1, 1), Direction::Down).is_none ());
        assert_eq! (grid.try_connect (&(1, 2), Direction::Up).unwrap (), (0, 2));
        assert! (grid.try_connect (&(1, 2), Direction::Right).is_none ());
        assert_eq! (grid.try_connect (&(1, 2), Direction::Left).unwrap (), (1, 1));
        assert! (grid.try_connect (&(1, 2), Direction::Down).is_none ());
    }

    #[test]
    fn grid_try_move () {
        let mut grid = generate_grid ();

        // Test empty move
        assert! (grid.try_move (&(0, 0), Direction::Up).is_none ());
        assert_eq! (grid.try_move (&(0, 0), Direction::Right).unwrap (), ((0, 1), 2));
        assert! (grid.try_move (&(0, 0), Direction::Left).is_none ());
        assert! (grid.try_move (&(0, 0), Direction::Down).is_none ());
        assert! (grid.try_move (&(0, 1), Direction::Up).is_none ());
        assert_eq! (grid.try_move (&(0, 1), Direction::Right).unwrap (), ((0, 2), 2));
        assert_eq! (grid.try_move (&(0, 1), Direction::Left).unwrap (), ((0, 0), 2));
        assert_eq! (grid.try_move (&(0, 1), Direction::Down).unwrap (), ((1, 1), 2));
        assert! (grid.try_move (&(0, 2), Direction::Up).is_none ());
        assert! (grid.try_move (&(0, 2), Direction::Right).is_none ());
        assert_eq! (grid.try_move (&(0, 2), Direction::Left).unwrap (), ((0, 1), 2));
        assert! (grid.try_move (&(0, 2), Direction::Down).is_none ());

        assert! (grid.try_move (&(1, 0), Direction::Up).is_none ());
        assert_eq! (grid.try_move (&(1, 0), Direction::Right).unwrap (), ((1, 1), 3));
        assert! (grid.try_move (&(1, 0), Direction::Left).is_none ());
        assert! (grid.try_move (&(1, 0), Direction::Down).is_none ());
        assert_eq! (grid.try_move (&(1, 1), Direction::Up).unwrap (), ((0, 1), 1));
        assert! (grid.try_move (&(1, 1), Direction::Right).is_none ());
        assert_eq! (grid.try_move (&(1, 1), Direction::Left).unwrap (), ((1, 0), 3));
        assert! (grid.try_move (&(1, 1), Direction::Down).is_none ());
        assert! (grid.try_move (&(1, 2), Direction::Up).is_none ());
        assert! (grid.try_move (&(1, 2), Direction::Right).is_none ());
        assert! (grid.try_move (&(1, 2), Direction::Left).is_none ());
        assert! (grid.try_move (&(1, 2), Direction::Down).is_none ());

        // Test non-empty move
        grid.unit_locations.insert ((0, (0, 1)));
        assert! (grid.try_move (&(0, 0), Direction::Right).is_none ());
        assert! (grid.try_move (&(0, 2), Direction::Left).is_none ());
        assert! (grid.try_move (&(1, 1), Direction::Up).is_none ());
    }

    #[test]
    fn grid_update_adjacency () {
        let scene = generate_scene ();
        let tiles_updated: Rectangle<Tile> = vec![
            vec![Tile::new (Rc::clone (&scene), 0, 10, Some (0)), Tile::new (Rc::clone (&scene), 0, 1, None), Tile::new (Rc::clone (&scene), 0, 0, Some (1))],
            vec![Tile::new (Rc::clone (&scene), 1, 2, Some (2)), Tile::new (Rc::clone (&scene), 1, 1, None), Tile::new (Rc::clone (&scene), 0, 0, None)],
        ];
        let mut grid = generate_grid ();

        grid.tiles = tiles_updated;

        // Test impassable update
        grid.update_adjacency (&(0, 0));
        assert! (grid.try_move (&(0, 1), Direction::Left).is_none ());
        assert! (grid.try_move (&(1, 0), Direction::Up).is_none ());
        // Test passable update
        grid.update_adjacency (&(1, 2));
        assert_eq! (grid.try_move (&(0, 2), Direction::Down).unwrap (), ((1, 2), 1));
        assert_eq! (grid.try_move (&(1, 1), Direction::Right).unwrap (), ((1, 2), 2));
    }

    #[test]
    fn grid_place_unit () {
        let mut grid = generate_grid ();

        // Test empty place
        assert_eq! (grid.place_unit (0, (0, 0)).unwrap (), 0);
        assert_eq! (grid.get_location_faction (&(0, 0)), &0);
        // Test impassable place
        assert! (grid.place_unit (1, (1, 2)).is_none ());
        // Test non-empty place
        assert! (grid.place_unit (2, (0, 0)).is_none ());
    }

    #[test]
    fn grid_move_unit () {
        let mut grid = generate_grid ();

        grid.place_unit (0, (0, 0));

        assert! (grid.move_unit (0, &[Direction::Up]).is_none ()); // Test out-of-bounds move
        assert! (grid.move_unit (0, &[Direction::Left]).is_none ()); // Test out-of-bounds move
        assert! (grid.move_unit (0, &[Direction::Down]).is_none ()); // Test not climbable move
        // Test normal move
        assert_eq! (grid.get_location_faction (&(0, 1)), &FACTION_UNCONTROLLED);
        let response = grid.move_unit (0, &[Direction::Right]).unwrap ();
        assert_eq! (response.0, (0, 1));
        assert_eq! (response.1, 0);
        assert_eq! (grid.get_unit_location (&0), &(0, 1));
        assert_eq! (grid.get_location_faction (&(0, 0)), &0);
        assert_eq! (grid.get_location_faction (&(0, 1)), &0);
        // Test sequential move
        assert_eq! (grid.get_location_faction (&(0, 2)), &FACTION_UNCONTROLLED);
        assert_eq! (grid.get_location_faction (&(1, 1)), &FACTION_UNCONTROLLED);
        let response = grid.move_unit (0, &[Direction::Right, Direction::Left, Direction::Down]).unwrap (); // Test overlapping move
        assert_eq! (response.0, (1, 1));
        assert_eq! (response.1, 1);
        assert_eq! (grid.get_unit_location (&0), &(1, 1));
        assert_eq! (grid.get_location_faction (&(0, 1)), &0);
        assert_eq! (grid.get_location_faction (&(0, 2)), &0);
        assert_eq! (grid.get_location_faction (&(1, 1)), &0);
        // Test atomic move
        assert! (grid.move_unit (0, &[Direction::Left, Direction::Right, Direction::Right]).is_none ()); // Test impassable move
        assert_eq! (grid.get_unit_location (&0), &(1, 1));
    }

    #[test]
    fn grid_find_unit_movable () {
        let mut grid = generate_grid ();

        grid.place_unit (0, (0, 0));

        // Test empty move
        let response = grid.find_unit_movable (&0, 1);
        assert_eq! (response.len (), 1);
        assert! (response.contains (&(0, 0)));
        // Test normal move
        let response = grid.find_unit_movable (&0, 2);
        assert_eq! (response.len (), 2);
        assert! (response.contains (&(0, 0)));
        assert! (response.contains (&(0, 1)));
        let response = grid.find_unit_movable (&0, 3);
        assert_eq! (response.len (), 2);
        assert! (response.contains (&(0, 0)));
        assert! (response.contains (&(0, 1)));
        let response = grid.find_unit_movable (&0, 4);
        assert_eq! (response.len (), 4);
        assert! (response.contains (&(0, 0)));
        assert! (response.contains (&(0, 1)));
        assert! (response.contains (&(0, 2)));
        assert! (response.contains (&(1, 1)));
        let response = grid.find_unit_movable (&0, 5);
        assert_eq! (response.len (), 4);
        assert! (response.contains (&(0, 0)));
        assert! (response.contains (&(0, 1)));
        assert! (response.contains (&(0, 2)));
        assert! (response.contains (&(1, 1)));
        let response = grid.find_unit_movable (&0, 6);
        assert_eq! (response.len (), 4);
        assert! (response.contains (&(0, 0)));
        assert! (response.contains (&(0, 1)));
        assert! (response.contains (&(0, 2)));
        assert! (response.contains (&(1, 1)));
        let response = grid.find_unit_movable (&0, 7);
        assert_eq! (response.len (), 5);
        assert! (response.contains (&(0, 0)));
        assert! (response.contains (&(0, 1)));
        assert! (response.contains (&(0, 2)));
        assert! (response.contains (&(1, 1)));
        assert! (response.contains (&(1, 0)));
    }

    #[test]
    fn grid_find_unit_cities () {
        let mut grid = generate_grid ();

        // Test no supply
        grid.place_unit (0, (0, 1));
        assert! (grid.find_unit_cities (&0).is_empty ());
        grid.move_unit (0, &[Direction::Down]);
        assert! (grid.find_unit_cities (&0).is_empty ());
        // Test contested supply
        grid.place_unit (2, (0, 0));
        assert! (grid.find_unit_cities (&0).is_empty ());
        assert_eq! (grid.find_unit_cities (&2).len (), 1);
        // Test normal supply
        grid.place_unit (1, (0, 2));
        assert_eq! (grid.find_unit_cities (&0).len (), 1);
        assert_eq! (grid.find_unit_cities (&1).len (), 1);
        assert_eq! (grid.find_unit_cities (&2).len (), 1);
        // Test multiple supply
        grid.place_unit (3, (1, 0));
        assert_eq! (grid.find_unit_cities (&0).len (), 2);
        assert_eq! (grid.find_unit_cities (&1).len (), 2);
        assert_eq! (grid.find_unit_cities (&2).len (), 1);
    }

    #[test]
    fn grid_find_locations () {
        let grid = generate_grid ();

        assert_eq! (grid.find_locations (&(0, 0), Search::Single).len (), 1);
        assert_eq! (grid.find_locations (&(0, 1), Search::Single).len (), 1);
        assert_eq! (grid.find_locations (&(0, 2), Search::Single).len (), 1);
        assert_eq! (grid.find_locations (&(1, 0), Search::Single).len (), 1);
        assert_eq! (grid.find_locations (&(1, 1), Search::Single).len (), 1);
        assert_eq! (grid.find_locations (&(1, 2), Search::Single).len (), 1);

        assert_eq! (grid.find_locations (&(0, 0), Search::Path (1, 1, Direction::Right)).len (), 2);
        assert_eq! (grid.find_locations (&(0, 1), Search::Path (1, 1, Direction::Down)).len (), 3);
        assert_eq! (grid.find_locations (&(0, 2), Search::Path (1, 1, Direction::Left)).len (), 2);
        assert_eq! (grid.find_locations (&(1, 0), Search::Path (1, 1, Direction::Right)).len (), 2);
        assert_eq! (grid.find_locations (&(1, 1), Search::Path (1, 1, Direction::Up)).len (), 3);
        assert_eq! (grid.find_locations (&(1, 2), Search::Path (1, 1, Direction::Left)).len (), 2);

        assert_eq! (grid.find_locations (&(0, 0), Search::Path (0, 2, Direction::Right)).len (), 2);
        assert_eq! (grid.find_locations (&(0, 1), Search::Path (0, 2, Direction::Down)).len (), 1);
        assert_eq! (grid.find_locations (&(0, 2), Search::Path (0, 2, Direction::Left)).len (), 2);
        assert_eq! (grid.find_locations (&(1, 0), Search::Path (0, 2, Direction::Right)).len (), 2);
        assert_eq! (grid.find_locations (&(1, 1), Search::Path (0, 2, Direction::Up)).len (), 1);
        assert_eq! (grid.find_locations (&(1, 2), Search::Path (0, 2, Direction::Left)).len (), 2);

        assert_eq! (grid.find_locations (&(0, 0), Search::Radial (1)).len (), 3);
        assert_eq! (grid.find_locations (&(0, 1), Search::Radial (1)).len (), 4);
        assert_eq! (grid.find_locations (&(0, 2), Search::Radial (1)).len (), 3);
        assert_eq! (grid.find_locations (&(1, 0), Search::Radial (1)).len (), 3);
        assert_eq! (grid.find_locations (&(1, 1), Search::Radial (1)).len (), 4);
        assert_eq! (grid.find_locations (&(1, 2), Search::Radial (1)).len (), 3);

        assert_eq! (grid.find_locations (&(0, 0), Search::Radial (2)).len (), 5);
        assert_eq! (grid.find_locations (&(0, 1), Search::Radial (2)).len (), 6);
        assert_eq! (grid.find_locations (&(0, 2), Search::Radial (2)).len (), 5);
        assert_eq! (grid.find_locations (&(1, 0), Search::Radial (2)).len (), 5);
        assert_eq! (grid.find_locations (&(1, 1), Search::Radial (2)).len (), 6);
        assert_eq! (grid.find_locations (&(1, 2), Search::Radial (2)).len (), 5);
    }

    #[test]
    fn grid_find_units () {
        let mut grid = generate_grid ();

        // Test empty find
        assert! (grid.find_units (&(0, 0), Search::Single).is_empty ());
        assert! (grid.find_units (&(0, 1), Search::Single).is_empty ());
        assert! (grid.find_units (&(0, 2), Search::Single).is_empty ());
        assert! (grid.find_units (&(1, 0), Search::Single).is_empty ());
        assert! (grid.find_units (&(1, 1), Search::Single).is_empty ());
        assert! (grid.find_units (&(1, 2), Search::Single).is_empty ());

        assert! (grid.find_units (&(0, 0), Search::Path (1, 1, Direction::Right)).is_empty ());
        assert! (grid.find_units (&(0, 1), Search::Path (1, 1, Direction::Down)).is_empty ());
        assert! (grid.find_units (&(0, 2), Search::Path (1, 1, Direction::Left)).is_empty ());
        assert! (grid.find_units (&(1, 0), Search::Path (1, 1, Direction::Right)).is_empty ());
        assert! (grid.find_units (&(1, 1), Search::Path (1, 1, Direction::Up)).is_empty ());
        assert! (grid.find_units (&(1, 2), Search::Path (1, 1, Direction::Left)).is_empty ());

        assert! (grid.find_units (&(0, 0), Search::Path (0, 2, Direction::Right)).is_empty ());
        assert! (grid.find_units (&(0, 1), Search::Path (0, 2, Direction::Down)).is_empty ());
        assert! (grid.find_units (&(0, 2), Search::Path (0, 2, Direction::Left)).is_empty ());
        assert! (grid.find_units (&(1, 0), Search::Path (0, 2, Direction::Right)).is_empty ());
        assert! (grid.find_units (&(1, 1), Search::Path (0, 2, Direction::Up)).is_empty ());
        assert! (grid.find_units (&(1, 2), Search::Path (0, 2, Direction::Left)).is_empty ());

        assert! (grid.find_units (&(0, 0), Search::Radial (1)).is_empty ());
        assert! (grid.find_units (&(0, 1), Search::Radial (1)).is_empty ());
        assert! (grid.find_units (&(0, 2), Search::Radial (1)).is_empty ());
        assert! (grid.find_units (&(1, 0), Search::Radial (1)).is_empty ());
        assert! (grid.find_units (&(1, 1), Search::Radial (1)).is_empty ());
        assert! (grid.find_units (&(1, 2), Search::Radial (1)).is_empty ());

        assert! (grid.find_units (&(0, 0), Search::Radial (2)).is_empty ());
        assert! (grid.find_units (&(0, 1), Search::Radial (2)).is_empty ());
        assert! (grid.find_units (&(0, 2), Search::Radial (2)).is_empty ());
        assert! (grid.find_units (&(1, 0), Search::Radial (2)).is_empty ());
        assert! (grid.find_units (&(1, 1), Search::Radial (2)).is_empty ());
        assert! (grid.find_units (&(1, 2), Search::Radial (2)).is_empty ());
        // Test non-empty find
        grid.place_unit (0, (0, 0));
        grid.place_unit (1, (0, 1));
        grid.place_unit (2, (0, 2));
        grid.place_unit (3, (1, 0));
        grid.place_unit (4, (1, 1));
        assert_eq! (grid.find_units (&(0, 0), Search::Single).len (), 1);
        assert_eq! (grid.find_units (&(0, 1), Search::Single).len (), 1);
        assert_eq! (grid.find_units (&(0, 2), Search::Single).len (), 1);
        assert_eq! (grid.find_units (&(1, 0), Search::Single).len (), 1);
        assert_eq! (grid.find_units (&(1, 1), Search::Single).len (), 1);
        assert! (grid.find_units (&(1, 2), Search::Single).is_empty ());

        assert_eq! (grid.find_units (&(0, 0), Search::Path (1, 1, Direction::Right)).len (), 2);
        assert_eq! (grid.find_units (&(0, 1), Search::Path (1, 1, Direction::Down)).len (), 2);
        assert_eq! (grid.find_units (&(0, 2), Search::Path (1, 1, Direction::Left)).len (), 2);
        assert_eq! (grid.find_units (&(1, 0), Search::Path (1, 1, Direction::Right)).len (), 2);
        assert_eq! (grid.find_units (&(1, 1), Search::Path (1, 1, Direction::Up)).len (), 3);
        assert_eq! (grid.find_units (&(1, 2), Search::Path (1, 1, Direction::Left)).len (), 2);

        assert_eq! (grid.find_units (&(0, 0), Search::Path (0, 2, Direction::Right)).len (), 2);
        assert_eq! (grid.find_units (&(0, 1), Search::Path (0, 2, Direction::Down)).len (), 1);
        assert_eq! (grid.find_units (&(0, 2), Search::Path (0, 2, Direction::Left)).len (), 2);
        assert_eq! (grid.find_units (&(1, 0), Search::Path (0, 2, Direction::Right)).len (), 1);
        assert_eq! (grid.find_units (&(1, 1), Search::Path (0, 2, Direction::Up)).len (), 1);
        assert_eq! (grid.find_units (&(1, 2), Search::Path (0, 2, Direction::Left)).len (), 2);

        assert_eq! (grid.find_units (&(0, 0), Search::Radial (1)).len (), 3);
        assert_eq! (grid.find_units (&(0, 1), Search::Radial (1)).len (), 4);
        assert_eq! (grid.find_units (&(0, 2), Search::Radial (1)).len (), 2);
        assert_eq! (grid.find_units (&(1, 0), Search::Radial (1)).len (), 3);
        assert_eq! (grid.find_units (&(1, 1), Search::Radial (1)).len (), 3);
        assert_eq! (grid.find_units (&(1, 2), Search::Radial (1)).len (), 2);

        assert_eq! (grid.find_units (&(0, 0), Search::Radial (2)).len (), 5);
        assert_eq! (grid.find_units (&(0, 1), Search::Radial (2)).len (), 5);
        assert_eq! (grid.find_units (&(0, 2), Search::Radial (2)).len (), 4);
        assert_eq! (grid.find_units (&(1, 0), Search::Radial (2)).len (), 4);
        assert_eq! (grid.find_units (&(1, 1), Search::Radial (2)).len (), 5);
        assert_eq! (grid.find_units (&(1, 2), Search::Radial (2)).len (), 4);
    }

    #[test]
    fn grid_add_appliable () {
        let scene = generate_scene ();
        let mut grid = generate_grid ();
        let attribute_2 = *scene.get_attribute (&2);
        let attribute_2 = Box::new (attribute_2);
        let modifier_1 = *scene.get_modifier (&1);
        let modifier_1 = Box::new (modifier_1);

        assert! (grid.add_appliable (&(0, 0), attribute_2));
        assert! (grid.tiles[0][0].try_yield_appliable (Rc::clone (&scene)).is_some ());
        let cost_down_0: u8 = grid.adjacencies[0][0][Direction::Down as usize];
        let cost_left_0: u8 = grid.adjacencies[1][1][Direction::Left as usize];
        assert! (grid.add_appliable (&(1, 0), modifier_1));
        let cost_down_1: u8 = grid.adjacencies[0][0][Direction::Down as usize];
        let cost_left_1: u8 = grid.adjacencies[1][1][Direction::Left as usize];
        assert_eq! (cost_down_0, 0);
        assert_eq! (cost_down_1, 0);
        assert! (cost_left_0 > cost_left_1);
    }

    #[test]
    fn grid_try_spawn_recruit () {
        let mut grid = generate_grid ();

        // Test empty spawn
        assert! (grid.try_spawn_recruit ((0, 1), &0).is_none ());
        // Test normal spawn
        assert_eq! (grid.try_spawn_recruit ((0, 0), &0).unwrap (), (1, 0));
        assert_eq! (grid.get_unit_location (&1), &(0, 0));
        // Test repeated spawn
        assert! (grid.try_spawn_recruit ((0, 0), &0).is_none ());
    }

    #[test]
    fn grid_find_locations_supplied () {
        let mut grid = generate_grid ();

        // Test normal find
        grid.place_unit (0, (0, 0));
        let response = grid.find_locations_supplied (&0);
        assert_eq! (response.len (), 1);
        assert! (response.contains (&(0, 0)));
        // Test disconnected find
        grid.place_unit (1, (0, 2));
        let response = grid.find_locations_supplied (&1);
        assert_eq! (response.len (), 1);
        assert! (response.contains (&(0, 2)));
        // Test connected find
        grid.move_unit (0, &[Direction::Right]);
        let response = grid.find_locations_supplied (&0);
        assert_eq! (response.len (), 3);
        assert! (response.contains (&(0, 0)));
        assert! (response.contains (&(0, 1)));
        assert! (response.contains (&(0, 2)));
    }

    #[test]
    fn grid_expand_control () {
        let mut grid = generate_grid ();

        // Test normal expand
        grid.place_unit (0, (0, 0));
        grid.expand_control (&0);
        let response = grid.get_faction_locations (&0).unwrap ();
        assert_eq! (response.len (), 2);
        assert! (response.contains (&(0, 0)));
        assert! (response.contains (&(0, 1)));
        // Test blocked expand
        grid.place_unit (2, (1, 1));
        grid.expand_control (&0);
        let response = grid.get_faction_locations (&0).unwrap ();
        assert_eq! (response.len (), 2);
        assert! (response.contains (&(0, 0)));
        assert! (response.contains (&(0, 1)));
        // Test encircled expand
        grid.expand_control (&2);
        let response = grid.get_faction_locations (&1).unwrap ();
        assert_eq! (response.len (), 1);
        assert! (response.contains (&(1, 1)));
    }

    #[test]
    fn grid_decrement_durations () {
        let scene = generate_scene ();
        let mut grid = generate_grid ();
        let attribute_2 = *scene.get_attribute (&2);
        let mut attribute_2 = Box::new (attribute_2);
        let modifier_2 = *scene.get_modifier (&2);
        let mut modifier_2 = Box::new (modifier_2);

        attribute_2.set_applier_id (0);
        modifier_2.set_applier_id (1);
        grid.add_appliable (&(0, 0), attribute_2);
        grid.add_appliable (&(1, 1), modifier_2);

        grid.decrement_durations (&0);
        assert! (grid.try_yield_appliable (&(0, 0)).is_some ());
        assert! (grid.get_modifier (&(1, 1)).is_some ());
        grid.decrement_durations (&0);
        assert! (grid.try_yield_appliable (&(0, 0)).is_some ());
        assert! (grid.get_modifier (&(1, 1)).is_some ());
        grid.decrement_durations (&0);
        assert! (grid.try_yield_appliable (&(0, 0)).is_none ());
        assert! (grid.get_modifier (&(1, 1)).is_some ());
        assert! (grid.get_modifier (&(1, 1)).unwrap ().get_next_id ().is_some ());
        grid.decrement_durations (&1);
        assert! (grid.get_modifier (&(1, 1)).is_some ());
        assert! (grid.get_modifier (&(1, 1)).unwrap ().get_next_id ().is_some ());
        grid.decrement_durations (&1);
        assert! (grid.get_modifier (&(1, 1)).is_some ());
        assert! (grid.get_modifier (&(1, 1)).unwrap ().get_next_id ().is_none ());
    }
}
