use std::{cell::RefCell, collections::{HashMap, HashSet, VecDeque}, fmt, rc::Rc};
use crate::engine::Lists;
use crate::engine::common::{Direction, DuplicateCollectionMap, DuplicateMap, Event, Location, Modifier, Observer, Subject, ID};
use crate::engine::event::{EVENT_MAP_GET_SUPPLY, EVENT_UNIT_SET_SUPPLY};

type Adjacencies = [u8; Direction::Length as usize];

const CLIMB_MAX: u8 = 2;
const FACTION_UNCONTROLLED: ID = 0;

#[derive (Debug)]
pub struct Terrain {
    modifiers: Vec<Modifier>,
    cost: u8
}

#[derive (Debug)]
pub struct City {
    population: u16, // (thousands)
    factories: u16,
    farms: u16
}

#[derive (Debug)]
pub struct Tile {
    lists: Rc<Lists>,
    modifiers: Vec<Modifier>,
    terrain_id: ID,
    height: u8,
    city_id: Option<ID>
}

#[derive (Debug)]
struct AdjacencyMatrix {
    matrix: Vec<Vec<Adjacencies>>
}

#[derive (Debug)]
pub struct Map {
    lists: Rc<Lists>,
    tiles: Vec<Vec<Tile>>,
    adjacency_matrix: AdjacencyMatrix,
    unit_locations: DuplicateMap<ID, Location>,
    faction_locations: DuplicateCollectionMap<ID, Location>,
    faction_units: DuplicateCollectionMap<ID, ID>,
    observers: Vec<Rc<RefCell<dyn Observer>>>
}

impl Terrain {
    pub const fn new (modifiers: Vec<Modifier>, cost: u8 ) -> Self {
        Self { modifiers, cost }
    }

    pub fn get_modifiers (&self) -> &Vec<Modifier> {
        &self.modifiers
    }

    pub fn get_cost (&self) -> u8 {
        self.cost
    }
}

impl City {
    pub const fn new (population: u16, factories: u16, farms: u16) -> Self {
        Self { population, factories, farms }
    }

    pub fn get_population (&self) -> u16 {
        self.population
    }

    pub fn get_factories (&self) -> u16 {
        self.factories
    }

    pub fn get_farms (&self) -> u16 {
        self.farms
    }
}

impl Tile {
    pub fn new (lists: Rc<Lists>, terrain_id: ID, height: u8, city_id: Option<ID>) -> Self {
        let lists: Rc<Lists> = Rc::clone (&lists);
        let modifiers: Vec<Modifier> = Vec::new ();

        Self { lists, modifiers, terrain_id, height, city_id }
    }

    pub fn get_cost (&self) -> u8 {
        self.lists.get_terrain (&self.terrain_id).get_cost ()
    }

    pub fn is_impassable (&self) -> bool {
        self.get_cost () == 0
    }

    fn try_climb (&self, other: &Tile) -> Option<u8> {
        let climb: u8 = self.height.abs_diff (other.height);

        if climb < CLIMB_MAX {
            Some (climb)
        } else {
            None
        }
    }

    pub fn find_cost (&self, other: &Tile) -> u8 {
        if self.is_impassable () || other.is_impassable () {
            0
        } else {
            self.try_climb (other).map_or (0, |c| other.get_cost () + c)
        }
    }

    pub fn get_terrain_id (&self) -> ID {
        self.terrain_id
    }

    pub fn get_modifiers (&self) -> &Vec<Modifier> {
        &self.modifiers
    }

    pub fn get_height (&self) -> u8 {
        self.height
    }

    pub fn get_city_id (&self) -> Option<ID> {
        self.city_id
    }
}

impl AdjacencyMatrix {
    pub fn new (tiles: &Vec<Vec<Tile>>) -> Self {
        assert! (rectangular_map::is_rectangular (tiles));

        let mut matrix: Vec<Vec<Adjacencies>> = Vec::new ();

        for i in 0 .. tiles.len () {
            matrix.push (Vec::new ());

            for j in 0 .. tiles[i].len () {
                let tile: &Tile = &tiles[i][j];
                let up: Option<&Tile> = i.checked_sub (1).map (|i| &tiles[i][j]);
                let right: Option<&Tile> = j.checked_add (1).map (|j|
                    if j < tiles[i].len () {
                        Some (&tiles[i][j])
                    } else {
                        None
                }).flatten ();
                let down: Option<&Tile> = i.checked_add (1).map (|i|
                    if i < tiles.len () {
                        Some (&tiles[i][j])
                    } else {
                        None
                }).flatten ();
                let left: Option<&Tile> = j.checked_sub (1).map (|j| &tiles[i][j]);
                let up: u8 = up.map_or (0, |t| tile.find_cost (t));
                let right: u8 = right.map_or (0, |t| tile.find_cost (t));
                let down: u8 = down.map_or (0, |t| tile.find_cost (t));
                let left: u8 = left.map_or (0, |t| tile.find_cost (t));
                let mut adjacencies: Adjacencies = [0; Direction::Length as usize];

                adjacencies[Direction::Up as usize] = up;
                adjacencies[Direction::Right as usize] = right;
                adjacencies[Direction::Down as usize] = down;
                adjacencies[Direction::Left as usize] = left;
                matrix[i].push (adjacencies);
            }
        }

        AdjacencyMatrix { matrix }
    }

    pub fn get_connection (&self, location: &Location, direction: Direction) -> u8 {
        assert! (rectangular_map::is_in_bounds (&self.matrix, location));
        assert! ((direction as usize) < (Direction::Length as usize));

        self.matrix[location.0][location.1][direction as usize]
    }

    pub fn try_move (&self, start: &Location, direction: Direction) -> Option<(Location, u8)> {
        assert! (rectangular_map::is_in_bounds (&self.matrix, start));
        assert! ((direction as usize) < (Direction::Length as usize));

        let cost: u8 = self.matrix[start.0][start.1][direction as usize];

        if cost > 0 {
            let mut end: Location = start.clone ();

            match direction {
                Direction::Up => end.0 = start.0.checked_sub (1)?,
                Direction::Right => end.1 = start.1.checked_add (1)?,
                Direction::Down => end.0 = start.0.checked_add (1)?,
                Direction::Left => end.1 = start.1.checked_sub (1)?,
                _ => panic! ("Unknown direction {:?}", direction)
            }

            if rectangular_map::is_in_bounds (&self.matrix, &end) {
                Some ((end, cost))
            } else {
                None
            }
        } else {
            None
        }
    }
}

impl Map {
    pub fn new (lists: Rc<Lists>, tiles: Vec<Vec<Tile>>, unit_factions: HashMap<ID, ID>) -> Self {
        let lists: Rc<Lists> = Rc::clone (&lists);
        let (_, mut factions): (Vec<ID>, Vec<ID>) = unit_factions.iter ().unzip ();

        factions.push (FACTION_UNCONTROLLED);

        let mut faction_locations: DuplicateCollectionMap<ID, Location> = DuplicateCollectionMap::new (factions.clone ());

        for i in 0 .. tiles.len () {
            for j in 0 .. tiles[i].len () {
                faction_locations.insert ((FACTION_UNCONTROLLED, (i, j)));
            }
        }

        let adjacency_matrix: AdjacencyMatrix = AdjacencyMatrix::new (&tiles);
        let unit_locations: DuplicateMap<ID, Location> = DuplicateMap::new ();
        let mut faction_units: DuplicateCollectionMap<ID, ID> = DuplicateCollectionMap::new (factions);

        for (unit_id, faction_id) in unit_factions {
            faction_units.insert ((faction_id, unit_id));
        }

        let observers: Vec<Rc<RefCell<dyn Observer>>> = Vec::new ();

        Self { lists, tiles, adjacency_matrix, unit_locations, faction_locations, faction_units, observers }
    }

    pub fn is_impassable (&self, location: &Location) -> bool {
        assert! (rectangular_map::is_in_bounds (&self.tiles, location));

        self.tiles[location.0][location.1].is_impassable ()
    }

    pub fn is_occupied (&self, location: &Location) -> bool {
        assert! (rectangular_map::is_in_bounds (&self.tiles, location));

        self.unit_locations.get_second (location).is_some ()
    }

    fn is_placeable (&self, location: &Location) -> bool {
        assert! (rectangular_map::is_in_bounds (&self.tiles, location));

        !self.is_occupied (location) && !self.is_impassable(location)
    }

    pub fn try_move (&self, location: &Location, direction: Direction) -> Option<(Location, u8)> {
        assert! (rectangular_map::is_in_bounds (&self.tiles, location));

        match self.adjacency_matrix.try_move (location, direction) {
            Some ((d, c)) => {
                if self.is_placeable (&d) {
                    Some ((d, c))
                } else {
                    None
                }
            }
            None => None
        }
    }

    pub fn place_unit (&mut self, unit_id: ID, location: Location) -> bool {
        assert! (rectangular_map::is_in_bounds (&self.tiles, &location));
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

    pub fn get_unit_supply_cities (&self, unit_id: &ID) -> Vec<ID> {
        assert! (self.tiles.len () > 0);
        assert! (self.tiles[0].len () > 0);

        let directions: [Direction; Direction::Length as usize] = [Direction::Up, Direction::Right, Direction::Down, Direction::Left];
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

            for direction in directions {
                match self.adjacency_matrix.try_move (&location, direction) {
                    Some ((e, _)) => {
                        let controller_id: &ID = self.faction_locations.get_second (&e).expect (&format! ("Faction not found for location {:?}", e));

                        if !is_visited[e.0][e.1] && controller_id == faction_id {
                            locations.push_back (e);
                            is_visited[e.0][e.1] = true;
                        }
                    }
                    None => ()
                }
            }
        };

        for city_id in city_ids.iter () {
            let mut value: u16 = *unit_id as u16;

            value <<= 8;
            value |= *city_id as u16;
            // Automatically assume encircled if no event received
            self.notify ((EVENT_UNIT_SET_SUPPLY, value));
        }

        city_ids
    }

    pub fn get_unit_location (&self, unit_id: &ID) -> Option<&Location> {
        self.unit_locations.get_first (unit_id)
    }

    pub fn get_location_unit (&self, location: &Location) -> Option<&ID> {
        assert! (rectangular_map::is_in_bounds (&self.tiles, location));

        self.unit_locations.get_second (location)
    }

    pub fn get_faction_locations (&self, faction_id: &ID) -> Option<&HashSet<Location>> {
        self.faction_locations.get_first (faction_id)
    }

    pub fn get_location_faction (&self, location: &Location) -> Option<&ID> {
        assert! (rectangular_map::is_in_bounds (&self.tiles, location));

        self.faction_locations.get_second (location)
    }
}

impl Subject for Map {
    fn add_observer (&mut self, observer: Rc<RefCell<dyn Observer>>) -> () {
        let observer: Rc<RefCell<dyn Observer>> = Rc::clone (&observer);

        self.observers.push (observer);
    }

    fn remove_observer (&mut self, observer: Rc<RefCell<dyn Observer>>) -> () {
        unimplemented! ()
    }

    fn notify (&self, event: Event) -> () {
        for observer in self.observers.iter () {
            observer.borrow_mut ().update (event);
        }
    }
}

impl Observer for Map {
    fn update (&mut self, event: Event) -> () {
        match event {
            (EVENT_MAP_GET_SUPPLY, u) => {
                let unit_id: ID = u as ID;

                self.get_unit_supply_cities (&unit_id);
            }
            _ => ()
        }
    }
}

impl fmt::Display for Terrain {
    fn fmt (&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write! (f, "{}", self.cost)
    }
}

impl fmt::Display for Tile {
    fn fmt (&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write! (f, "{}.{}", self.terrain_id, self.height)
    }
}

impl fmt::Display for Map {
    fn fmt (&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut display: String = String::from ("");

        for i in 0 .. self.tiles.len () {
            for j in 0 .. self.tiles[i].len () {
                let tile: &Tile = &self.tiles[i][j];

                if self.is_occupied (&(i, j)) {
                    display.push_str (&format! ("{}o{} ",
                            self.unit_locations.get_second (&(i, j))
                                    .expect (&format! ("Missing unit on ({}, {})", i, j)),
                            tile.height));
                } else {
                    display.push_str (&format! ("{}_{} ", self.lists.get_terrain (&tile.terrain_id), tile.height));
                }
            }

            display.push_str ("\n");
        }

        write! (f, "{}", display)
    }
}    

mod rectangular_map {
    use crate::engine::common::Location;

    pub fn is_rectangular<T> (map: &Vec<Vec<T>>) -> bool {
        assert! (map.len () > 0);
        assert! (map[0].len () > 0);

        map.iter ().all (|r| r.len () == map[0].len ())
    }

    pub fn is_in_bounds<T> (map: &Vec<Vec<T>>, location: &Location) -> bool {
        assert! (map.len () > 0);
        assert! (map[0].len () > 0);

        location.0 < map.len () && location.1 < map[0].len ()
    }
}

#[cfg (test)]
pub mod tests {
    use super::*;
    use crate::engine::tests::generate_lists;

    fn generate_tiles () -> Vec<Vec<Tile>> {
        let lists: Rc<Lists> = generate_lists ();

        vec![
            vec![Tile::new (Rc::clone (&lists), 0, 0, Some (0)), Tile::new (Rc::clone (&lists), 0, 1, None), Tile::new (Rc::clone (&lists), 0, 0, Some (1))],
            vec![Tile::new (Rc::clone (&lists), 1, 2, None), Tile::new (Rc::clone (&lists), 1, 1, None), Tile::new (Rc::clone (&lists), 2, 0, None)]
        ]
    }

    fn generate_unit_factions () -> HashMap<ID, ID> {
        let mut unit_factions: HashMap<ID, ID> = HashMap::new ();

        unit_factions.insert (0, 1);
        unit_factions.insert (1, 1);
        unit_factions.insert (2, 2);
        unit_factions.insert (3, 3);

        unit_factions
    }

    pub fn generate_map () -> Map {
        let lists: Rc<Lists> = generate_lists ();
        let tiles: Vec<Vec<Tile>> = generate_tiles ();
        let unit_factions: HashMap<ID, ID> = generate_unit_factions ();

        Map::new (Rc::clone (&lists), tiles, unit_factions)
    }

    #[test]
    fn terrain_data () {
        let lists: Rc<Lists> = generate_lists ();

        assert_eq! (lists.get_terrain (&0).get_modifiers ().len (), 0);
        assert_eq! (lists.get_terrain (&0).get_cost (), 1);
        assert_eq! (lists.get_terrain (&1).get_modifiers ().len (), 0);
        assert_eq! (lists.get_terrain (&1).get_cost (), 2);
        assert_eq! (lists.get_terrain (&2).get_modifiers ().len (), 0);
        assert_eq! (lists.get_terrain (&2).get_cost (), 0);
    }

    #[test]
    fn tile_get_cost () {
        let lists: Rc<Lists> = generate_lists ();
        let tile_0: Tile = Tile::new (Rc::clone (&lists), 0, 0, None);
        let tile_1: Tile = Tile::new (Rc::clone (&lists), 1, 0, None);
        let tile_2: Tile = Tile::new (Rc::clone (&lists), 2, 0, None);

        assert_eq! (tile_0.get_cost (), 1);
        assert_eq! (tile_1.get_cost (), 2);
        assert_eq! (tile_2.get_cost (), 0);
    }

    #[test]
    fn tile_is_impassable () {
        let lists: Rc<Lists> = generate_lists ();
        let tile_0: Tile = Tile::new (Rc::clone (&lists), 0, 0, None);
        let tile_2: Tile = Tile::new (Rc::clone (&lists), 2, 0, None);

        // Test passable tile
        assert! (!tile_0.is_impassable ());
        // Test impassable tile
        assert! (tile_2.is_impassable ());
    }

    #[test]
    fn tile_try_climb () {
        let lists: Rc<Lists> = generate_lists ();
        let tile_0: Tile = Tile::new (Rc::clone (&lists), 0, 0, None);
        let tile_1_0: Tile = Tile::new (Rc::clone (&lists), 1, 0, None);
        let tile_1_1: Tile = Tile::new (Rc::clone (&lists), 1, 1, None);
        let tile_1_2: Tile = Tile::new (Rc::clone (&lists), 1, 2, None);

        // Test impassable climb
        assert_eq! (tile_0.try_climb (&tile_1_2), None);
        assert_eq! (tile_1_2.try_climb (&tile_0), None);
        // Test passable climb
        assert_eq! (tile_0.try_climb (&tile_1_0).unwrap (), 0);
        assert_eq! (tile_1_0.try_climb (&tile_0).unwrap (), 0);
        assert_eq! (tile_1_0.try_climb (&tile_1_1).unwrap (), 1);
        assert_eq! (tile_1_1.try_climb (&tile_1_0).unwrap (), 1);
    }

    #[test]
    fn tile_find_cost () {
        let lists: Rc<Lists> = generate_lists ();
        let tile_0: Tile = Tile::new (Rc::clone (&lists), 0, 0, None);
        let tile_1_0: Tile = Tile::new (Rc::clone (&lists), 1, 0, None);
        let tile_1_1: Tile = Tile::new (Rc::clone (&lists), 1, 1, None);
        let tile_2: Tile = Tile::new (Rc::clone (&lists), 2, 0, None);

        // Test impassable cost
        assert_eq! (tile_0.find_cost (&tile_2), 0);
        assert_eq! (tile_2.find_cost (&tile_0), 0);
        // Test passable cost
        assert_eq! (tile_0.find_cost (&tile_1_0), 2);
        assert_eq! (tile_1_0.find_cost (&tile_0), 1);
        assert_eq! (tile_0.find_cost (&tile_1_1), 3);
        assert_eq! (tile_1_1.find_cost (&tile_0), 2);
    }

    #[test]
    fn adjacency_matrix_get_connection () {
        let tile_map: Vec<Vec<Tile>> = generate_tiles ();
        let adjacency_matrix: AdjacencyMatrix = AdjacencyMatrix::new (&tile_map);

        assert_eq! (adjacency_matrix.get_connection (&(0, 0), Direction::Up), 0);
        assert_eq! (adjacency_matrix.get_connection (&(0, 0), Direction::Right), 2);
        assert_eq! (adjacency_matrix.get_connection (&(0, 0), Direction::Down), 0);
        assert_eq! (adjacency_matrix.get_connection (&(0, 0), Direction::Left), 0);
        assert_eq! (adjacency_matrix.get_connection (&(0, 1), Direction::Up), 0);
        assert_eq! (adjacency_matrix.get_connection (&(0, 1), Direction::Right), 2);
        assert_eq! (adjacency_matrix.get_connection (&(0, 1), Direction::Down), 2);
        assert_eq! (adjacency_matrix.get_connection (&(0, 1), Direction::Left), 2);
        assert_eq! (adjacency_matrix.get_connection (&(0, 2), Direction::Up), 0);
        assert_eq! (adjacency_matrix.get_connection (&(0, 2), Direction::Right), 0);
        assert_eq! (adjacency_matrix.get_connection (&(0, 2), Direction::Down), 0);
        assert_eq! (adjacency_matrix.get_connection (&(0, 2), Direction::Left), 2);

        assert_eq! (adjacency_matrix.get_connection (&(1, 0), Direction::Up), 0);
        assert_eq! (adjacency_matrix.get_connection (&(1, 0), Direction::Right), 3);
        assert_eq! (adjacency_matrix.get_connection (&(1, 0), Direction::Down), 0);
        assert_eq! (adjacency_matrix.get_connection (&(1, 0), Direction::Left), 0);
        assert_eq! (adjacency_matrix.get_connection (&(1, 1), Direction::Up), 1);
        assert_eq! (adjacency_matrix.get_connection (&(1, 1), Direction::Right), 0);
        assert_eq! (adjacency_matrix.get_connection (&(1, 1), Direction::Down), 0);
        assert_eq! (adjacency_matrix.get_connection (&(1, 1), Direction::Left), 3);
        assert_eq! (adjacency_matrix.get_connection (&(1, 2), Direction::Up), 0);
        assert_eq! (adjacency_matrix.get_connection (&(1, 2), Direction::Right), 0);
        assert_eq! (adjacency_matrix.get_connection (&(1, 2), Direction::Down), 0);
        assert_eq! (adjacency_matrix.get_connection (&(1, 2), Direction::Left), 0);
    }

    
    #[test]
    fn map_is_impassable () {
        let map: Map = generate_map ();

        // Test passable
        assert_eq! (map.is_impassable (&(0, 0)), false);
        // Test impassable
        assert_eq! (map.is_impassable (&(1, 2)), true);
    }

    #[test]
    fn map_is_occupied () {
        let mut map: Map = generate_map ();

        // Test empty
        assert_eq! (map.is_occupied (&(0, 0)), false);
        // Test occupied
        map.place_unit (0, (0, 0));
        assert_eq! (map.is_occupied (&(0, 0)), true);
    }

    #[test]
    fn map_is_placeable () {
        let mut map: Map = generate_map ();

        // Test passable
        assert_eq! (map.is_placeable (&(0, 0)), true);
        assert_eq! (map.is_placeable (&(0, 1)), true);
        assert_eq! (map.is_placeable (&(0, 2)), true);
        assert_eq! (map.is_placeable (&(1, 0)), true);
        assert_eq! (map.is_placeable (&(1, 1)), true);
        // Test impassable
        assert_eq! (map.is_placeable (&(1, 2)), false);
        // Test occupied
        map.place_unit (0, (0, 0));
        assert_eq! (map.is_placeable (&(0, 0)), false);
    }

    #[test]
    fn map_is_movable () {
        let map: Map = generate_map ();

        assert_eq! (map.try_move (&(0, 0), Direction::Up), None);
        assert_eq! (map.try_move (&(0, 0), Direction::Right).unwrap (), ((0, 1), 2));
        assert_eq! (map.try_move (&(0, 0), Direction::Down), None); // Test not climbable
        assert_eq! (map.try_move (&(0, 0), Direction::Left), None);
        assert_eq! (map.try_move (&(0, 1), Direction::Up), None);
        assert_eq! (map.try_move (&(0, 1), Direction::Right).unwrap (), ((0, 2), 2));
        assert_eq! (map.try_move (&(0, 1), Direction::Down).unwrap (), ((1, 1), 2));
        assert_eq! (map.try_move (&(0, 1), Direction::Left).unwrap (), ((0, 0), 2));
        assert_eq! (map.try_move (&(0, 2), Direction::Up), None);
        assert_eq! (map.try_move (&(0, 2), Direction::Right), None);
        assert_eq! (map.try_move (&(0, 2), Direction::Down), None); // Test impassable
        assert_eq! (map.try_move (&(0, 2), Direction::Left).unwrap (), ((0, 1), 2));

        assert_eq! (map.try_move (&(1, 0), Direction::Up), None); // Test not climbable
        assert_eq! (map.try_move (&(1, 0), Direction::Right).unwrap (), ((1, 1), 3));
        assert_eq! (map.try_move (&(1, 0), Direction::Down), None);
        assert_eq! (map.try_move (&(1, 0), Direction::Left), None);
        assert_eq! (map.try_move (&(1, 1), Direction::Up).unwrap (), ((0, 1), 1));
        assert_eq! (map.try_move (&(1, 1), Direction::Right), None); // Test impassable
        assert_eq! (map.try_move (&(1, 1), Direction::Down), None);
        assert_eq! (map.try_move (&(1, 1), Direction::Left).unwrap (), ((1, 0), 3));
        // Test impassable
        assert_eq! (map.try_move (&(1, 2), Direction::Up), None);
        assert_eq! (map.try_move (&(1, 2), Direction::Right), None);
        assert_eq! (map.try_move (&(1, 2), Direction::Down), None);
        assert_eq! (map.try_move (&(1, 2), Direction::Left), None);
    }

    #[test]
    fn map_place_unit () {
        let mut map: Map = generate_map ();

        // Test empty place
        assert_eq! (map.place_unit (0, (0, 0)), true);
        assert_eq! (map.faction_locations.get_second (&(0, 0)).unwrap (), &1);
        // Test impassable place
        assert_eq! (map.place_unit (1, (1, 2)), false);
        // Test non-empty place
        assert_eq! (map.place_unit (2, (0, 0)), false);
    }

    #[test]
    fn map_move_unit () {
        let mut map: Map = generate_map ();

        map.place_unit (0, (0, 0));
        assert_eq! (map.move_unit (0, vec![Direction::Up]), false); // Test out-of-bounnds
        assert_eq! (map.move_unit (0, vec![Direction::Down]), false); // Test not climbable
        assert_eq! (map.move_unit (0, vec![Direction::Left]), false); // Test out-of-bounds
        // Test normal move
        assert_eq! (map.faction_locations.get_second (&(0, 1)).unwrap (), &FACTION_UNCONTROLLED);
        assert_eq! (map.move_unit (0, vec![Direction::Right]), true);
        assert_eq! (map.get_unit_location (&0).unwrap (), &(0, 1));
        assert_eq! (map.faction_locations.get_second (&(0, 0)).unwrap (), &1);
        assert_eq! (map.faction_locations.get_second (&(0, 1)).unwrap (), &1);
        // Test sequential move
        assert_eq! (map.faction_locations.get_second (&(0, 2)).unwrap (), &FACTION_UNCONTROLLED);
        assert_eq! (map.faction_locations.get_second (&(1, 1)).unwrap (), &FACTION_UNCONTROLLED);
        assert_eq! (map.move_unit (0, vec![Direction::Right, Direction::Left, Direction::Down]), true); // Test overlap
        assert_eq! (map.get_unit_location (&0).unwrap (), &(1, 1));
        assert_eq! (map.faction_locations.get_second (&(0, 1)).unwrap (), &1);
        assert_eq! (map.faction_locations.get_second (&(0, 2)).unwrap (), &1);
        assert_eq! (map.faction_locations.get_second (&(1, 1)).unwrap (), &1);
        // Test atomic move
        assert_eq! (map.move_unit (0, vec![Direction::Left, Direction::Right, Direction::Right]), false); // Test impassable
        assert_eq! (map.get_unit_location (&0).unwrap (), &(1, 1));
    }

    #[test]
    fn map_get_unit_supply_cities () {
        let mut map: Map = generate_map ();

        // Test no supply
        map.place_unit (0, (0, 1));
        assert_eq! (map.get_unit_supply_cities (&0).len (), 0);
        map.move_unit (0, vec![Direction::Down, Direction::Left]);
        assert_eq! (map.get_unit_supply_cities (&0).len (), 0);
        // Test contested supply
        map.place_unit (2, (0, 0));
        assert_eq! (map.get_unit_supply_cities (&0).len (), 0);
        assert_eq! (map.get_unit_supply_cities (&2).len (), 1);
        // Test normal supply
        map.place_unit (1, (0, 2));
        assert_eq! (map.get_unit_supply_cities (&0).len (), 1);
        assert_eq! (map.get_unit_supply_cities (&1).len (), 1);
        assert_eq! (map.get_unit_supply_cities (&2).len (), 1);
    }
}
