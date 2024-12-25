use std::{cell::RefCell, collections::{HashMap, HashSet, VecDeque}, fmt, rc::Rc};
use crate::engine::Lists;
use crate::engine::common::{Area, Direction, DuplicateInnerMap, DuplicateNaturalMap, ID, ID_UNINITIALISED, Modifiable, Modifier, Target};
use crate::engine::event::{Event, Subject, Observer, Response, RESPONSE_NOTIFICATION};
use super::{Location, Tile};

type Adjacencies = [u8; Direction::Length as usize];

const FACTION_UNCONTROLLED: ID = 0;

fn is_rectangular<T> (grid: &Vec<Vec<T>>) -> bool {
    assert! (grid.len () > 0);
    assert! (grid[0].len () > 0);

    grid.iter ().all (|r| r.len () == grid[0].len ())
}

fn is_in_bounds<T> (grid: &Vec<Vec<T>>, location: &Location) -> bool {
    assert! (grid.len () > 0);
    assert! (grid[0].len () > 0);

    location.0 < grid.len () && location.1 < grid[0].len ()
}

#[derive (Debug)]
struct AdjacencyMatrix {
    matrix: Vec<Vec<Adjacencies>>
}

impl AdjacencyMatrix {
    pub fn new (tiles: &Vec<Vec<Tile>>) -> Self {
        assert! (is_rectangular (tiles));

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
        assert! (is_in_bounds (&self.matrix, location));

        self.matrix[location.0][location.1][direction as usize]
    }

    pub fn try_connect (&self, start: &Location, direction: Direction) -> Option<Location> {
        assert! (is_in_bounds (&self.matrix, start));

        let mut end: Location = start.clone ();

        match direction {
            Direction::Up => end.0 = start.0.checked_sub (1)?,
            Direction::Right => end.1 = start.1.checked_add (1)?,
            Direction::Down => end.0 = start.0.checked_add (1)?,
            Direction::Left => end.1 = start.1.checked_sub (1)?,
            _ => panic! ("Invalid direction {:?}", direction)
        }

        if is_in_bounds (&self.matrix, &end) {
            Some (end)
        } else {
            None
        }
    }

    pub fn try_move (&self, start: &Location, direction: Direction) -> Option<(Location, u8)> {
        assert! (is_in_bounds (&self.matrix, start));

        let cost: u8 = self.matrix[start.0][start.1][direction as usize];

        if cost > 0 {
            let mut end: Location = start.clone ();

            match direction {
                Direction::Up => end.0 = start.0.checked_sub (1)?,
                Direction::Right => end.1 = start.1.checked_add (1)?,
                Direction::Down => end.0 = start.0.checked_add (1)?,
                Direction::Left => end.1 = start.1.checked_sub (1)?,
                _ => panic! ("Invalid direction {:?}", direction)
            }

            if is_in_bounds (&self.matrix, &end) {
                Some ((end, cost))
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn update_adjacency (&mut self, location: &Location, cost: u8) -> () {

    }
}

#[derive (Debug)]
pub struct Grid {
    lists: Rc<Lists>,
    tiles: Vec<Vec<Tile>>,
    adjacency_matrix: AdjacencyMatrix,
    unit_locations: DuplicateNaturalMap<ID, Location>,
    faction_locations: DuplicateInnerMap<ID, Location>,
    // TODO: This is definitely getting replaced whenever factions are done
    faction_units: DuplicateInnerMap<ID, ID>,
    observer_id: ID
}

impl Grid {
    pub fn new (lists: Rc<Lists>, tiles: Vec<Vec<Tile>>, unit_factions: HashMap<ID, ID>) -> Self {
        let lists: Rc<Lists> = Rc::clone (&lists);
        let mut faction_locations: DuplicateInnerMap<ID, Location> = DuplicateInnerMap::new ();
        let adjacency_matrix: AdjacencyMatrix = AdjacencyMatrix::new (&tiles);
        let unit_locations: DuplicateNaturalMap<ID, Location> = DuplicateNaturalMap::new ();
        let mut faction_units: DuplicateInnerMap<ID, ID> = DuplicateInnerMap::new ();
        let observer_id: ID = ID_UNINITIALISED;

        for i in 0 .. tiles.len () {
            for j in 0 .. tiles[i].len () {
                faction_locations.insert ((FACTION_UNCONTROLLED, (i, j)));
            }
        }

        for (unit_id, faction_id) in unit_factions {
            faction_units.insert ((faction_id, unit_id));
        }

        Self { lists, tiles, adjacency_matrix, unit_locations, faction_locations, faction_units, observer_id }
    }

    pub fn is_impassable (&self, location: &Location) -> bool {
        assert! (is_in_bounds (&self.tiles, location));

        self.tiles[location.0][location.1].is_impassable ()
    }

    pub fn is_occupied (&self, location: &Location) -> bool {
        assert! (is_in_bounds (&self.tiles, location));

        self.unit_locations.get_second (location).is_some ()
    }

    fn is_placeable (&self, location: &Location) -> bool {
        assert! (is_in_bounds (&self.tiles, location));

        !self.is_occupied (location) && !self.is_impassable(location)
    }

    pub fn try_move (&self, location: &Location, direction: Direction) -> Option<(Location, u8)> {
        assert! (is_in_bounds (&self.tiles, location));

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

    pub fn get_unit_supply_cities (&self, unit_id: &ID) -> Vec<ID> {
        assert! (self.tiles.len () > 0);
        assert! (self.tiles[0].len () > 0);

        const DIRECTIONS: [Direction; Direction::Length as usize] = [Direction::Up, Direction::Right, Direction::Down, Direction::Left];
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

        city_ids
    }

    pub fn find_nearby_allies (&self, unit_id: &ID, target: Target, area: Area, range: u8) -> Vec<ID> {
        assert! (self.tiles.len () > 0);
        assert! (self.tiles[0].len () > 0);

        let location_self: Location = self.get_unit_location (unit_id).expect (&format! ("Location not found for unit {}", unit_id)).clone ();

        self.faction_units.get_collection_second (unit_id)
                .expect (&format! ("Faction not found for unit {}", unit_id))
                .iter ().filter_map (|u| {
                    let location_other: Location = self.get_unit_location (u).expect (&format! ("Location not found for unit {}", u)).clone ();
                    let distance: usize = location_other.0.abs_diff (location_self.0) + location_other.1.abs_diff (location_self.1);

                    if distance > (range as usize) {
                        None
                    } else {
                        Some (*u)
                    }
                }).collect ()
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
    fn subscribe (&mut self, event_id: ID) -> ID {
        todo! ()
    }

    fn unsubscribe (&mut self, event_id: ID) -> ID {
        todo! ()   
    }

    fn update (&mut self, event_id: ID) -> () {
        todo! ()
    }

    fn get_observer_id (&self) -> Option<ID> {
        if self.observer_id == ID_UNINITIALISED {
            None
        } else {
            Some (self.observer_id)
        }
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

        unit_factions
    }

    pub fn generate_grid () -> Grid {
        let lists: Rc<Lists> = generate_lists ();
        let tiles: Vec<Vec<Tile>> = generate_tiles ();
        let unit_factions: HashMap<ID, ID> = generate_unit_factions ();

        Grid::new (Rc::clone (&lists), tiles, unit_factions)
    }

    #[test]
    fn adjacency_matrix_get_connection () {
        let tile_grid: Vec<Vec<Tile>> = generate_tiles ();
        let adjacency_matrix: AdjacencyMatrix = AdjacencyMatrix::new (&tile_grid);

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
        grid.place_unit (0, (0, 0));
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
        grid.place_unit (0, (0, 0));
        assert_eq! (grid.is_placeable (&(0, 0)), false);
    }

    #[test]
    fn grid_is_movable () {
        let grid: Grid = generate_grid ();

        assert_eq! (grid.try_move (&(0, 0), Direction::Up), None);
        assert_eq! (grid.try_move (&(0, 0), Direction::Right).unwrap (), ((0, 1), 2));
        assert_eq! (grid.try_move (&(0, 0), Direction::Down), None); // Test not climbable
        assert_eq! (grid.try_move (&(0, 0), Direction::Left), None);
        assert_eq! (grid.try_move (&(0, 1), Direction::Up), None);
        assert_eq! (grid.try_move (&(0, 1), Direction::Right).unwrap (), ((0, 2), 2));
        assert_eq! (grid.try_move (&(0, 1), Direction::Down).unwrap (), ((1, 1), 2));
        assert_eq! (grid.try_move (&(0, 1), Direction::Left).unwrap (), ((0, 0), 2));
        assert_eq! (grid.try_move (&(0, 2), Direction::Up), None);
        assert_eq! (grid.try_move (&(0, 2), Direction::Right), None);
        assert_eq! (grid.try_move (&(0, 2), Direction::Down), None); // Test impassable
        assert_eq! (grid.try_move (&(0, 2), Direction::Left).unwrap (), ((0, 1), 2));

        assert_eq! (grid.try_move (&(1, 0), Direction::Up), None); // Test not climbable
        assert_eq! (grid.try_move (&(1, 0), Direction::Right).unwrap (), ((1, 1), 3));
        assert_eq! (grid.try_move (&(1, 0), Direction::Down), None);
        assert_eq! (grid.try_move (&(1, 0), Direction::Left), None);
        assert_eq! (grid.try_move (&(1, 1), Direction::Up).unwrap (), ((0, 1), 1));
        assert_eq! (grid.try_move (&(1, 1), Direction::Right), None); // Test impassable
        assert_eq! (grid.try_move (&(1, 1), Direction::Down), None);
        assert_eq! (grid.try_move (&(1, 1), Direction::Left).unwrap (), ((1, 0), 3));
        // Test impassable
        assert_eq! (grid.try_move (&(1, 2), Direction::Up), None);
        assert_eq! (grid.try_move (&(1, 2), Direction::Right), None);
        assert_eq! (grid.try_move (&(1, 2), Direction::Down), None);
        assert_eq! (grid.try_move (&(1, 2), Direction::Left), None);
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
        assert_eq! (grid.faction_locations.get_second (&(0, 1)).unwrap (), &FACTION_UNCONTROLLED);
        assert_eq! (grid.move_unit (0, vec![Direction::Right]), true);
        assert_eq! (grid.get_unit_location (&0).unwrap (), &(0, 1));
        assert_eq! (grid.faction_locations.get_second (&(0, 0)).unwrap (), &1);
        assert_eq! (grid.faction_locations.get_second (&(0, 1)).unwrap (), &1);
        // Test sequential move
        assert_eq! (grid.faction_locations.get_second (&(0, 2)).unwrap (), &FACTION_UNCONTROLLED);
        assert_eq! (grid.faction_locations.get_second (&(1, 1)).unwrap (), &FACTION_UNCONTROLLED);
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
    fn grid_get_unit_supply_cities () {
        let mut grid: Grid = generate_grid ();

        // Test no supply
        grid.place_unit (0, (0, 1));
        assert_eq! (grid.get_unit_supply_cities (&0).len (), 0);
        grid.move_unit (0, vec![Direction::Down, Direction::Left]);
        assert_eq! (grid.get_unit_supply_cities (&0).len (), 0);
        // Test contested supply
        grid.place_unit (2, (0, 0));
        assert_eq! (grid.get_unit_supply_cities (&0).len (), 0);
        assert_eq! (grid.get_unit_supply_cities (&2).len (), 1);
        // Test normal supply
        grid.place_unit (1, (0, 2));
        assert_eq! (grid.get_unit_supply_cities (&0).len (), 1);
        assert_eq! (grid.get_unit_supply_cities (&1).len (), 1);
        assert_eq! (grid.get_unit_supply_cities (&2).len (), 1);
        // Test multiple supply
        grid.faction_locations.replace ((0, 0), 1);
        assert_eq! (grid.get_unit_supply_cities (&0).len (), 2);
        assert_eq! (grid.get_unit_supply_cities (&1).len (), 2);
        assert_eq! (grid.get_unit_supply_cities (&2).len (), 1);
    }
}
