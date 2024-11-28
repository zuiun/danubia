use std::{collections::{HashMap, HashSet}, fmt, rc::Rc};
use crate::engine::common::{Direction, DuplicateCollectionMap, DuplicateMap, Information, Location, Modifier, ID};

const CLIMB_MAX: u8 = 2;

type Adjacencies = [u8; Direction::Length as usize];

#[derive (Debug)]
pub struct Terrain {
    information: Information,
    modifiers: Vec<Modifier>,
    cost: u8
}

#[derive (Debug)]
pub struct Tile {
    terrains: Rc<HashMap<ID, Terrain>>,
    modifiers: Vec<Modifier>,
    terrain_id: ID,
    height: u8
}

#[derive (Debug)]
pub struct TileBuilder {
    terrain_id: ID,
    height: u8
}

#[derive (Debug)]
pub struct TileMapBuilder {
    tiles: Vec<Vec<TileBuilder>>
}

#[derive (Debug)]
pub struct AdjacencyMatrix {
    matrix: Vec<Vec<Adjacencies>>
}

#[derive (Debug)]
pub struct Map {
    terrains: Rc<HashMap<ID, Terrain>>,
    map: Vec<Vec<Tile>>,
    adjacency_matrix: AdjacencyMatrix,
    character_locations: DuplicateMap<ID, Location>,
    controller_locations: DuplicateCollectionMap<ID, Location>,
}

impl Terrain {
    pub fn new (information: Information, modifiers: Vec<Modifier>, cost: u8 ) -> Self {
        Self { information, modifiers, cost }
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
}

impl TileBuilder {
    pub fn new (terrain_id: ID, height: u8) -> Self {
        Self { terrain_id, height }
    }

    pub fn build (&self, terrains: Rc<HashMap<ID, Terrain>>) -> Tile {
        let modifiers: Vec<Modifier> = Vec::new ();

        Tile { terrains, modifiers, terrain_id: self.terrain_id, height: self.height }
    }
}

impl TileMapBuilder {
    pub fn new (tiles: Vec<Vec<TileBuilder>>) -> Self {
        assert! (tiles.len () > 0);
        assert! (tiles[0].len () > 0);
        assert! (tiles.iter ().all (|r| r.len () == tiles[0].len ()));

        Self { tiles }
    }

    pub fn build (self, terrains: Rc<HashMap<ID, Terrain>>) -> Vec<Vec<Tile>> {
        let mut tile_map: Vec<Vec<Tile>> = Vec::new ();
        
        for i in 0 .. self.tiles.len () {
            tile_map.push (Vec::new ());

            for j in 0 .. self.tiles[i].len () {
                let terrains: Rc<HashMap<ID, Terrain>> = Rc::clone (&terrains);

                tile_map[i].push (self.tiles[i][j].build (terrains));
            }
        }

        tile_map
    }
}

impl AdjacencyMatrix {
    pub fn new (tile_map: &Vec<Vec<Tile>>) -> Self {
        assert! (tile_map.len () > 0);
        assert! (tile_map[0].len () > 0);
        assert! (tile_map.iter ().all (|r| r.len () == tile_map[0].len ()));

        let mut matrix: Vec<Vec<Adjacencies>> = Vec::new ();

        for i in 0 .. tile_map.len () {
            matrix.push (Vec::new ());

            for j in 0 .. tile_map[i].len () {
                let tile: &Tile = &tile_map[i][j];
                let up: Option<&Tile> = i.checked_sub (1).map (|i| &tile_map[i][j]);
                let right: Option<&Tile> = j.checked_add (1).map (|j|
                    if j < tile_map[i].len () {
                        Some (&tile_map[i][j])
                    } else {
                        None
                    })
                    .flatten ();
                let down: Option<&Tile> = i.checked_add (1).map (|i|
                    if i < tile_map.len () {
                        Some (&tile_map[i][j])
                    } else {
                        None
                    })
                    .flatten ();
                let left: Option<&Tile> = j.checked_sub (1).map (|j| &tile_map[i][j]);
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

        Self { matrix }
    }

    pub fn get_connection (&self, location: &Location, direction: Direction) -> u8 {
        assert! (location.0 < self.matrix.len ());
        assert! (self.matrix.len () > 0);
        assert! (location.1 < self.matrix[0].len ());

        self.matrix[location.0][location.1][direction as usize]
    }
}

impl Map {
    pub fn new (terrains: HashMap<ID, Terrain>, tile_map_builder: TileMapBuilder) -> Self {
        let factions: Vec<ID> = Vec::new (); // TODO: Import factions
        let terrains: Rc<HashMap<ID, Terrain>> = Rc::new (terrains);
        let map: Vec<Vec<Tile>> = tile_map_builder.build (Rc::clone (&terrains));
        let adjacency_matrix: AdjacencyMatrix = AdjacencyMatrix::new (&map);
        let character_locations: DuplicateMap<ID, Location> = DuplicateMap::new ();
        let controller_locations: DuplicateCollectionMap<ID, Location> = DuplicateCollectionMap::new (factions);

        Self { terrains, map, adjacency_matrix, character_locations, controller_locations }
    }

    pub fn is_in_bounds (&self, location: &Location) -> bool {
        assert! (self.map.len () > 0);

        location.0 < self.map.len () && location.1 < self.map[0].len ()
    }

    pub fn is_occupied (&self, location: &Location) -> bool {
        assert! (self.is_in_bounds (location));

        self.get_character (location).is_some ()
    }

    fn is_placeable (&self, location: &Location) -> bool {
        assert! (self.is_in_bounds (location));

        !self.is_occupied (location) && !self.map[location.0][location.1].is_impassable ()
    }

    pub fn try_move (&self, location: &Location, direction: Direction) -> Option<(Location, u8)> {
        assert! (self.is_in_bounds (location));

        let cost: u8 = self.adjacency_matrix.get_connection (location, direction);

        if cost > 0 {
            let mut destination: Location = location.clone ();

            match direction {
                Direction::Up => destination.0 = location.0.checked_sub (1)?,
                Direction::Right => destination.1 = location.1.checked_add (1)?,
                Direction::Down => destination.0 = location.0.checked_add (1)?,
                Direction::Left => destination.1 = location.1.checked_sub (1)?,
                _ => panic! ("Unknown direction {:?}", direction)
            }

            assert! (self.is_in_bounds (&destination));

            if self.is_placeable (&destination) {
                Some ((destination, cost))
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn place_character (&mut self, character_id: ID, location: Location) -> bool {
        assert! (self.is_in_bounds (&location));
        assert! (!self.character_locations.contains_key_first (&character_id));

        if self.is_placeable (&location) {
            self.character_locations.insert ((character_id, location));

            true
        } else {
            false
        }
    }

    pub fn move_character (&mut self, character_id: ID, movements: Vec<Direction>) -> bool {
        let location_old: Location = match self.get_location (&character_id) {
            Some (l) => l.clone (),
            None => return false
        };
        let mut location_new: Location = location_old.clone ();

        // TODO: Pass through faction members
        // Tiles need to know who occupies them, but characters need to be implemented first
        // Temporarily remove character
        self.character_locations.remove_first (&character_id);

        for direction in movements {
            location_new = match self.try_move (&location_new, direction) {
                Some (d) => d.0.clone (),
                None => {
                    // Restore character
                    self.character_locations.insert ((character_id, location_old));

                    return false
                }
            };
        }

        self.character_locations.insert ((character_id, location_new));

        true
    }

    pub fn get_character (&self, location: &Location) -> Option<&ID> {
        assert! (self.is_in_bounds (location));

        self.character_locations.get_second (location)
    }

    pub fn get_controller (&self, location: &Location) -> Option<&ID> {
        assert! (self.is_in_bounds (location));

        self.controller_locations.get_second (location)
    }

    pub fn get_location (&self, character_id: &ID) -> Option<&Location> {
        self.character_locations.get_first (character_id)
    }

    pub fn get_locations (&self, faction_id: &ID) -> Option<&HashSet<Location>> {
        self.controller_locations.get_first (faction_id)
    }
}

impl fmt::Display for Terrain {
    fn fmt (&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write! (f, "{}", self.information)
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

        for i in 0 .. self.map.len () {
            for j in 0 .. self.map[i].len () {
                let tile: &Tile = &self.map[i][j];

                if self.is_occupied (&(i, j)) {
                    display.push_str (&format! ("{}o{} ",
                            self.get_character (&(i, j))
                                    .expect (&format! ("Missing character on ({}, {})", i, j)),
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

#[cfg (test)]
mod tests {
    use super::*;

    fn generate_terrains () -> HashMap<ID, Terrain> {
        let grass: Terrain = Terrain::new (Information::new (String::from ("Grass"), vec![String::from ("grass")], 0), Vec::new (), 1);
        let dirt: Terrain = Terrain::new (Information::new (String::from ("Dirt"), vec![String::from ("dirt")], 0), Vec::new (), 2);
        let stone: Terrain = Terrain::new (Information::new (String::from ("Stone"), vec![String::from ("stone")], 0), Vec::new (), 0);
        let mut terrains: HashMap<ID, Terrain> = HashMap::new ();

        terrains.insert (0, grass);
        terrains.insert (1, dirt);
        terrains.insert (2, stone);

        terrains
    }

    fn generate_tile_map_builder () -> TileMapBuilder {
        TileMapBuilder::new (vec![
            vec![TileBuilder::new (0, 0), TileBuilder::new (0, 1), TileBuilder::new (0, 0)],
            vec![TileBuilder::new (1, 2), TileBuilder::new (1, 1), TileBuilder::new (2, 0)]
        ])
    }

    fn generate_map () -> Map {
        let terrains: HashMap<ID, Terrain> = generate_terrains ();
        let tile_map_builder: TileMapBuilder = generate_tile_map_builder ();

        Map::new (terrains, tile_map_builder)
    }

    fn generate_tile (terrains: Rc<HashMap<ID, Terrain>>, terrain_id: ID, height: u8) -> Tile {
        let tile_builder: TileBuilder = TileBuilder::new (terrain_id, height);

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
        let tile_builder: TileBuilder = TileBuilder::new (0, 0);

        assert_eq! (tile_builder.terrain_id, 0);
        assert_eq! (tile_builder.height, 0);
    }

    #[test]
    fn tile_builder_build () {
        let tile_builder: TileBuilder = TileBuilder::new (0, 0);
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
    fn tile_map_builder_build () {
        let tile_map_builder: TileMapBuilder = generate_tile_map_builder ();
        let terrains: Rc<HashMap<u8, Terrain>> = Rc::new (generate_terrains ());
        let tile_map: Vec<Vec<Tile>> = tile_map_builder.build (Rc::clone (&terrains));

        assert_eq! (Rc::strong_count (&terrains), 7);
        assert_eq! (tile_map[0][0].get_terrain_id (), 0);
        assert_eq! (tile_map[0][0].get_height (), 0);
        assert_eq! (tile_map[0][0].is_impassable (), false);
        assert_eq! (tile_map[0][0].get_terrain_id (), 0);
        assert_eq! (tile_map[0][1].get_height (), 1);
        assert_eq! (tile_map[0][1].is_impassable (), false);
        assert_eq! (tile_map[0][2].get_terrain_id (), 0);
        assert_eq! (tile_map[0][2].get_height (), 0);
        assert_eq! (tile_map[0][2].is_impassable (), false);

        assert_eq! (tile_map[1][0].get_terrain_id (), 1);
        assert_eq! (tile_map[1][0].get_height (), 2);
        assert_eq! (tile_map[1][0].is_impassable (), false);
        assert_eq! (tile_map[1][0].get_terrain_id (), 1);
        assert_eq! (tile_map[1][1].get_height (), 1);
        assert_eq! (tile_map[1][1].is_impassable (), false);
        assert_eq! (tile_map[1][2].get_terrain_id (), 2);
        assert_eq! (tile_map[1][2].get_height (), 0);
        assert_eq! (tile_map[1][2].is_impassable (), true);
    }

    #[test]
    fn adjacency_matrix_get_connection () {
        let terrains: Rc<HashMap<u8, Terrain>> = Rc::new (generate_terrains ());
        let tile_map_builder: TileMapBuilder = generate_tile_map_builder ();
        let tile_map: Vec<Vec<Tile>> = tile_map_builder.build (terrains);
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
    fn map_is_in_bounds () {
        let map: Map = generate_map ();

        // Test in-bounds
        assert_eq! (map.is_in_bounds (&(0, 0)), true);
        assert_eq! (map.is_in_bounds (&(0, 1)), true);
        assert_eq! (map.is_in_bounds (&(0, 2)), true);
        assert_eq! (map.is_in_bounds (&(1, 0)), true);
        assert_eq! (map.is_in_bounds (&(1, 1)), true);
        assert_eq! (map.is_in_bounds (&(1, 2)), true);
        // Test out-of-bounds
        assert_eq! (map.is_in_bounds (&(0, 3)), false);
        assert_eq! (map.is_in_bounds (&(1, 3)), false);
        assert_eq! (map.is_in_bounds (&(2, 0)), false);
        assert_eq! (map.is_in_bounds (&(2, 1)), false);
        assert_eq! (map.is_in_bounds (&(2, 2)), false);
        assert_eq! (map.is_in_bounds (&(2, 3)), false);
    }

    #[test]
    fn map_is_occupied () {
        let mut map: Map = generate_map ();

        // Test empty
        assert_eq! (map.is_occupied (&(0, 0)), false);
        // Test occupied
        map.place_character (0, (0, 0));
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
        map.place_character (0, (0, 0));
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
    fn map_place_character () {
        let mut map: Map = generate_map ();

        // Test empty place
        assert_eq! (map.place_character (0, (0, 0)), true);
        // Test impassable place
        assert_eq! (map.place_character (1, (1, 2)), false);
        // Test non-empty place
        assert_eq! (map.place_character (2, (0, 0)), false);
    }

    #[test]
    fn map_move_character () {
        let mut map: Map = generate_map ();

        map.place_character (0, (0, 0));
        assert_eq! (map.move_character (0, vec![Direction::Up]), false); // Test out-of-bounnds
        assert_eq! (map.move_character (0, vec![Direction::Down]), false); // Test not climbable
        assert_eq! (map.move_character (0, vec![Direction::Left]), false); // Test out-of-bounds
        // Test normal move
        assert_eq! (map.move_character (0, vec![Direction::Right]), true);
        assert_eq! (map.get_location (&0).unwrap (), &(0, 1));
        // Test sequential move
        assert_eq! (map.move_character (0, vec![Direction::Right, Direction::Left, Direction::Down]), true);
        assert_eq! (map.get_location (&0).unwrap (), &(1, 1));
        // Test atomic move
        assert_eq! (map.move_character (0, vec![Direction::Left, Direction::Right, Direction::Right]), false); // Test impassable
        assert_eq! (map.get_location (&0).unwrap (), &(1, 1));
    }
}
