use std::{collections::{HashMap, HashSet, VecDeque}, fmt, rc::Rc};
use crate::engine::common::{Direction, DuplicateCollectionMap, DuplicateMap, Location, Modifier, ID};
use crate::engine::event::{SET_ENCIRCLED_EVENT, UNIT_TYPE};

const CLIMB_MAX: u8 = 2;
const FACTION_UNCONTROLLED: ID = 0;

type Adjacencies = [u8; Direction::Length as usize];

#[derive (Debug)]
pub struct Terrain {
    modifiers: Vec<Modifier>,
    cost: u8
}

#[derive (Debug)]
struct Tile {
    terrains: Rc<HashMap<ID, Terrain>>,
    modifiers: Vec<Modifier>,
    terrain_id: ID,
    height: u8,
    city_id: Option<ID>
}

#[derive (Debug)]
pub struct TileBuilder {
    terrain_id: ID,
    height: u8,
    city_id: Option<ID>
}

pub struct TileMap {
    tiles: Vec<Vec<Tile>>
}

#[derive (Debug)]
struct AdjacencyMatrix {
    matrix: Vec<Vec<Adjacencies>>
}

#[derive (Debug)]
pub struct Map {
    terrains: Rc<HashMap<ID, Terrain>>,
    tiles: Vec<Vec<Tile>>,
    adjacency_matrix: AdjacencyMatrix,
    unit_locations: DuplicateMap<ID, Location>,
    faction_locations: DuplicateCollectionMap<ID, Location>,
    faction_units: DuplicateCollectionMap<ID, ID>
}

#[derive (Debug)]
pub struct City {
    population: u8, // (thousands)
    factories: u8,
    farms: u8
}

impl Terrain {
    pub fn new (modifiers: Vec<Modifier>, cost: u8 ) -> Self {
        Self { modifiers, cost }
    }

    pub fn get_modifiers (&self) -> &Vec<Modifier> {
        &self.modifiers
    }

    pub fn get_cost (&self) -> u8 {
        self.cost
    }
}

impl Tile {
    pub fn get_terrain (&self) -> &Terrain {
        self.terrains.get (&self.terrain_id)
                .expect (&format! ("Terrain {} not found", self.terrain_id))
    }

    pub fn get_cost (&self) -> u8 {
        self.get_terrain ().get_cost ()
    }

    pub fn is_impassable (&self) -> bool {
        self.get_cost () == 0
    }

    fn try_climb (&self, tile: &Tile) -> Option<u8> {
        let climb: u8 = self.height.abs_diff (tile.height);

        if climb < CLIMB_MAX {
            Some (climb)
        } else {
            None
        }
    }

    pub fn find_cost (&self, tile: &Tile) -> u8 {
        if self.is_impassable () || tile.is_impassable () {
            0
        } else {
            self.try_climb (tile).map_or (0, |c| tile.get_cost () + c)
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

impl TileBuilder {
    pub fn new (terrain_id: ID, height: u8, city_id: Option<ID>) -> Self {
        Self { terrain_id, height, city_id }
    }

    pub fn build (&self, terrains: Rc<HashMap<ID, Terrain>>) -> Tile {
        let modifiers: Vec<Modifier> = Vec::new ();

        Tile { terrains, modifiers, terrain_id: self.terrain_id, height: self.height, city_id: self.city_id }
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

    pub fn try_move (&self, location: &Location, direction: Direction) -> Option<(Location, u8)> {
        assert! (rectangular_map::is_in_bounds (&self.matrix, location));
        assert! ((direction as usize) < (Direction::Length as usize));

        let cost: u8 = self.matrix[location.0][location.1][direction as usize];

        if cost > 0 {
            let mut destination: Location = location.clone ();

            match direction {
                Direction::Up => destination.0 = location.0.checked_sub (1)?,
                Direction::Right => destination.1 = location.1.checked_add (1)?,
                Direction::Down => destination.0 = location.0.checked_add (1)?,
                Direction::Left => destination.1 = location.1.checked_sub (1)?,
                _ => panic! ("Unknown direction {:?}", direction)
            }

            if rectangular_map::is_in_bounds (&self.matrix, &destination) {
                Some ((destination, cost))
            } else {
                None
            }
        } else {
            None
        }
    }
}

impl Map {
    pub fn new (terrains: HashMap<ID, Terrain>, tile_builders: Vec<Vec<TileBuilder>>, unit_factions: HashMap<ID, ID>) -> Self {
        let (_, mut factions): (Vec<ID>, Vec<ID>) = unit_factions.iter ().unzip ();
        let terrains: Rc<HashMap<ID, Terrain>> = Rc::new (terrains);
        let mut tiles: Vec<Vec<Tile>> = Vec::new ();

        factions.push (FACTION_UNCONTROLLED);

        let mut faction_locations: DuplicateCollectionMap<ID, Location> = DuplicateCollectionMap::new (factions.clone ());

        for i in 0 .. tile_builders.len () {
            tiles.push (Vec::new ());

            for j in 0 .. tile_builders[i].len () {
                let terrains: Rc<HashMap<ID, Terrain>> = Rc::clone (&terrains);

                tiles[i].push (tile_builders[i][j].build (terrains));
                faction_locations.insert ((FACTION_UNCONTROLLED, (i, j)));
            }
        }

        let adjacency_matrix: AdjacencyMatrix = AdjacencyMatrix::new (&tiles);
        let unit_locations: DuplicateMap<ID, Location> = DuplicateMap::new ();
        let mut faction_units: DuplicateCollectionMap<ID, ID> = DuplicateCollectionMap::new (factions);

        let _ = unit_factions.iter ().map (|(u, f)| faction_units.insert ((*f, *u))).collect::<Vec<_>> ();

        Self { terrains, tiles, adjacency_matrix, unit_locations, faction_locations, faction_units }
    }

    

    pub fn is_occupied (&self, location: &Location) -> bool {
        assert! (rectangular_map::is_in_bounds (&self.tiles, location));

        self.get_location_occupant (location).is_some ()
    }

    fn is_placeable (&self, location: &Location) -> bool {
        assert! (rectangular_map::is_in_bounds (&self.tiles, location));

        !self.is_occupied (location) && !self.tiles[location.0][location.1].is_impassable ()
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
        let location_old: Location = self.get_unit_location (&unit_id).expect (&format! ("Location not found for unit {}", unit_id)).clone ();
        let mut location_new: Location = location_old.clone ();

        // Temporarily remove unit
        self.unit_locations.remove_first (&unit_id);

        for direction in movements {
            location_new = match self.try_move (&location_new, direction) {
                Some (d) => d.0.clone (),
                None => {
                    // Restore unit
                    self.unit_locations.insert ((unit_id, location_old));

                    return false
                }
            };
            locations.push (location_new);
        }

        let _ = locations.iter ().map (|d| self.faction_locations.replace (*d, *faction_id)).collect::<Vec<_>> ();
        self.unit_locations.insert ((unit_id, location_new));

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
        let mut cities: Vec<ID> = Vec::new ();

        locations.push_back (location);
        is_visited[location.0][location.1] = true;

        while locations.len () > 0 {
            let location: Location = locations.pop_front ().expect ("Location not found");

            if let Some (c) = self.tiles[location.0][location.1].get_city_id () {
                cities.push (c);
            }

            let _ = directions.iter ().map (|d| {
                match self.adjacency_matrix.try_move (&location, *d) {
                    Some ((d, _)) => {
                        let controller_id: &ID = self.faction_locations.get_second (&d).expect (&format! ("Faction not found for location {:?}", d));

                        if !is_visited[d.0][d.1] && controller_id == faction_id {
                            locations.push_back (d);
                            is_visited[d.0][d.1] = true;
                        }
                    }
                    None => ()
                }
            }).collect::<Vec<_>> ();
        };

        // TODO: Signal units about encirclement
        cities
    }

    pub fn get_location_occupant (&self, location: &Location) -> Option<&ID> {
        assert! (rectangular_map::is_in_bounds (&self.tiles, location));

        self.unit_locations.get_second (location)
    }

    pub fn get_location_controller (&self, location: &Location) -> Option<&ID> {
        assert! (rectangular_map::is_in_bounds (&self.tiles, location));

        self.faction_locations.get_second (location)
    }

    pub fn get_unit_location (&self, unit_id: &ID) -> Option<&Location> {
        self.unit_locations.get_first (unit_id)
    }

    pub fn get_faction_locations (&self, faction_id: &ID) -> Option<&HashSet<Location>> {
        self.faction_locations.get_first (faction_id)
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
                            self.get_location_occupant (&(i, j))
                                    .expect (&format! ("Missing unit on ({}, {})", i, j)),
                            tile.height));
                } else {
                    display.push_str (&format! ("{}_{} ",
                            self.terrains.get (&tile.terrain_id)
                                    .expect (&format! ("Unknown terrain ID {}", tile.terrain_id)),
                            tile.height));
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
mod tests {
    use super::*;

    fn generate_terrains () -> HashMap<ID, Terrain> {
        let passable_1: Terrain = Terrain::new (Vec::new (), 1);
        let passable_2: Terrain = Terrain::new (Vec::new (), 2);
        let impassable: Terrain = Terrain::new (Vec::new (), 0);
        let mut terrains: HashMap<ID, Terrain> = HashMap::new ();

        terrains.insert (0, passable_1);
        terrains.insert (1, passable_2);
        terrains.insert (2, impassable);

        terrains
    }

    fn generate_tile_builders () -> Vec<Vec<TileBuilder>> {
        vec![
            vec![TileBuilder::new (0, 0, Some (0)), TileBuilder::new (0, 1, None), TileBuilder::new (0, 0, Some (1))],
            vec![TileBuilder::new (1, 2, None), TileBuilder::new (1, 1, None), TileBuilder::new (2, 0, None)]
        ]
    }

    fn generate_tiles () -> Vec<Vec<Tile>> {
        let terrains: Rc<HashMap<ID, Terrain>> = Rc::new (generate_terrains ());
        let tile_builders: Vec<Vec<TileBuilder>> = generate_tile_builders ();
        let mut tiles: Vec<Vec<Tile>> = Vec::new ();

        for i in 0 .. tile_builders.len () {
            tiles.push (Vec::new ());

            for j in 0 .. tile_builders[i].len () {
                tiles[i].push (tile_builders[i][j].build (Rc::clone (&terrains)));
            }
        }

        tiles
    }

    fn generate_unit_factions () -> HashMap<ID, ID> {
        let mut unit_factions: HashMap<ID, ID> = HashMap::new ();

        unit_factions.insert (0, 1);
        unit_factions.insert (1, 1);
        unit_factions.insert (2, 2);
        unit_factions.insert (3, 3);

        unit_factions
    }

    fn generate_map () -> Map {
        let terrains: HashMap<ID, Terrain> = generate_terrains ();
        let tile_map_builder: Vec<Vec<TileBuilder>> = generate_tile_builders ();
        let unit_factions: HashMap<ID, ID> = generate_unit_factions ();

        Map::new (terrains, tile_map_builder, unit_factions)
    }

    fn generate_tile (terrains: Rc<HashMap<ID, Terrain>>, terrain_id: ID, height: u8) -> Tile {
        let tile_builder: TileBuilder = TileBuilder::new (terrain_id, height, None);

        tile_builder.build (terrains)
    }

    #[test]
    fn terrain_data () {
        let terrains: HashMap<ID, Terrain> = generate_terrains ();

        assert_eq! (terrains.get (&0).unwrap ().get_modifiers ().len (), 0);
        assert_eq! (terrains.get (&0).unwrap ().get_cost (), 1);
        assert_eq! (terrains.get (&1).unwrap ().get_modifiers ().len (), 0);
        assert_eq! (terrains.get (&1).unwrap ().get_cost (), 2);
        assert_eq! (terrains.get (&2).unwrap ().get_modifiers ().len (), 0);
        assert_eq! (terrains.get (&2).unwrap ().get_cost (), 0);
    }

    #[test]
    fn tile_builder_data () {
        let tile_builder: TileBuilder = TileBuilder::new (0, 0, None);

        assert_eq! (tile_builder.terrain_id, 0);
        assert_eq! (tile_builder.height, 0);
    }

    #[test]
    fn tile_builder_build () {
        let tile_builder: TileBuilder = TileBuilder::new (0, 0, None);
        let terrains: Rc<HashMap<u8, Terrain>> = Rc::new (generate_terrains ());
        let tile: Tile = tile_builder.build (terrains);

        assert_eq! (Rc::strong_count (&tile.terrains), 1);
        assert_eq! (tile.get_modifiers ().len (), 0);
        assert_eq! (tile.get_terrain_id (), 0);
        assert_eq! (tile.get_height (), 0);
    }

    #[test]
    fn tile_get_terrain () {
        let terrains: Rc<HashMap<u8, Terrain>> = Rc::new (generate_terrains ());

        let tile: Tile = generate_tile (Rc::clone (&terrains), 0, 0);
        assert_eq! (tile.get_terrain ().cost, 1);
        let tile: Tile = generate_tile (Rc::clone (&terrains), 1, 0);
        assert_eq! (tile.get_terrain ().cost, 2);
        let tile: Tile = generate_tile (Rc::clone (&terrains), 2, 0);
        assert_eq! (tile.get_terrain ().cost, 0);
    }

    #[test]
    fn tile_get_cost () {
        let terrains: Rc<HashMap<u8, Terrain>> = Rc::new (generate_terrains ());

        let tile: Tile = generate_tile (Rc::clone (&terrains), 0, 0);
        assert_eq! (tile.get_cost (), 1);
        let tile: Tile = generate_tile (Rc::clone (&terrains), 1, 0);
        assert_eq! (tile.get_cost (), 2);
        let tile: Tile = generate_tile (Rc::clone (&terrains), 2, 0);
        assert_eq! (tile.get_cost (), 0);
    }

    #[test]
    fn tile_is_impassable () {
        let terrains: Rc<HashMap<u8, Terrain>> = Rc::new (generate_terrains ());

        // Test passable tile
        let tile: Tile = generate_tile (Rc::clone (&terrains), 0, 0);
        assert! (!tile.is_impassable ());
        // Test impassable tile
        let tile: Tile = generate_tile (Rc::clone (&terrains), 2, 0);
        assert! (tile.is_impassable ());
    }

    fn tile_try_climb () {
        let terrains: Rc<HashMap<u8, Terrain>> = Rc::new (generate_terrains ());

        // Test impassable climb
        let tile_1: Tile = generate_tile (Rc::clone (&terrains), 0, 0);
        let tile_2: Tile = generate_tile (Rc::clone (&terrains), 2, 0);
        assert_eq! (tile_1.try_climb (&tile_2), None);
        assert_eq! (tile_2.try_climb (&tile_1), None);
        // Test passable climb
        let tile_1: Tile = generate_tile (Rc::clone (&terrains), 0, 0);
        let tile_2: Tile = generate_tile (Rc::clone (&terrains), 1, 0);
        assert_eq! (tile_1.try_climb (&tile_2).unwrap (), 0);
        assert_eq! (tile_2.try_climb (&tile_1).unwrap (), 0);
        let tile_1: Tile = generate_tile (Rc::clone (&terrains), 0, 0);
        let tile_2: Tile = generate_tile (Rc::clone (&terrains), 1, 1);
        assert_eq! (tile_1.try_climb (&tile_2).unwrap (), 1);
        assert_eq! (tile_2.try_climb (&tile_1).unwrap (), 1);
    }

    #[test]
    fn tile_find_cost () {
        let terrains: Rc<HashMap<u8, Terrain>> = Rc::new (generate_terrains ());

        // Test impassable cost
        let tile_1: Tile = generate_tile (Rc::clone (&terrains), 0, 0);
        let tile_2: Tile = generate_tile (Rc::clone (&terrains), 2, 0);
        assert_eq! (tile_1.find_cost (&tile_2), 0);
        assert_eq! (tile_2.find_cost (&tile_1), 0);
        // Test passable cost
        let tile_1: Tile = generate_tile (Rc::clone (&terrains), 0, 0);
        let tile_2: Tile = generate_tile (Rc::clone (&terrains), 1, 0);
        assert_eq! (tile_1.find_cost (&tile_2), 2);
        assert_eq! (tile_2.find_cost (&tile_1), 1);
        let tile_1: Tile = generate_tile (Rc::clone (&terrains), 0, 0);
        let tile_2: Tile = generate_tile (Rc::clone (&terrains), 1, 1);
        assert_eq! (tile_1.find_cost (&tile_2), 3);
        assert_eq! (tile_2.find_cost (&tile_1), 2);
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

    // vec![
    //         vec![TileBuilder::new (0, 0, Some (0)), TileBuilder::new (0, 1, None), TileBuilder::new (0, 0, Some (1))],
    //         vec![TileBuilder::new (1, 2, None), TileBuilder::new (1, 1, None), TileBuilder::new (2, 0, None)]
    //     ]
    //     unit_factions.insert (0, 1);
    //     unit_factions.insert (1, 1);
    //     unit_factions.insert (2, 2);
    //     unit_factions.insert (3, 3);

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
        assert_eq! (map.get_unit_supply_cities (&2).len (), 1);
    }
}
