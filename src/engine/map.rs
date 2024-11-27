use std::{collections::{HashMap, HashSet}, fmt, rc::Rc};
use crate::engine::common::{DuplicateCollectionMap, DuplicateMap, Information, Location, Modifier, Movement, ID};

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
pub struct Map {
    terrains: Rc<HashMap<ID, Terrain>>,
    map: Vec<Vec<Tile>>,
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
    pub fn is_impassable (&self) -> bool {
        match self.terrains.get (&self.terrain_id) {
            Some (t) => t.cost == 0,
            None => panic! ("Terrain {} not found", self.terrain_id)
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
        for i in 1 .. tiles.len () {
            assert! (tiles[i].len () == tiles[i - 1].len ());
        }

        Self { tiles }
    }

    pub fn build (self, terrains: Rc<HashMap<ID, Terrain>>) -> Vec<Vec<Tile>> {
        let mut map: Vec<Vec<Tile>> = Vec::new ();
        
        for i in 0 .. self.tiles.len () {
            map.push (Vec::new ());

            for j in 0 .. self.tiles[i].len () {
                let terrains: Rc<HashMap<ID, Terrain>> = Rc::clone (&terrains);

                map[i].push (self.tiles[i][j].build (terrains));
            }
        }

        map
    }

    pub fn get_rows (&self) -> usize {
        self.tiles.len ()
    }

    pub fn get_columns (&self) -> usize {
        assert! (self.tiles.len () > 0);

        let columns: usize = self.tiles[0].len ();

        if self.tiles.iter ().all (|r| r.len () == columns) { columns } else { panic! ("Map builder is not rectangular") }
    }
}

impl Map {
    pub fn new (terrains: HashMap<ID, Terrain>, tile_map_builder: TileMapBuilder) -> Self {
        let factions: Vec<ID> = Vec::new (); // TODO: Import factions
        let terrains: Rc<HashMap<ID, Terrain>> = Rc::new (terrains);
        let map: Vec<Vec<Tile>> = tile_map_builder.build (Rc::clone (&terrains));
        let character_locations: DuplicateMap<ID, Location> = DuplicateMap::new ();
        let controller_locations: DuplicateCollectionMap<ID, Location> = DuplicateCollectionMap::new (factions);

        Self { terrains, map, character_locations, controller_locations }
    }

    pub fn is_in_bounds (&self, location: &Location) -> bool {
        location.0 < self.map.len () && location.1 < self.map[0].len ()
    }

    pub fn is_occupied (&self, location: &Location) -> bool {
        assert! (self.is_in_bounds (location));

        match self.get_character (location) {
            Some (_) => true,
            None => false
        }
    }

    fn is_impassable (&self, location: &Location) -> bool {
        assert! (self.is_in_bounds (location));

        self.map[location.0][location.1].is_impassable ()
    }

    fn is_placeable (&self, location: &Location) -> bool {
        assert! (self.is_in_bounds (location));

        !self.is_occupied (location) && !self.is_impassable (location)
    }

    pub fn is_movable (&self, location: &Location, movement: &Movement) -> bool {
        assert! (self.is_in_bounds (location));

        let mut destination: Location = location.clone ();

        match location.0.checked_add_signed (movement.0) {
            Some (r) => destination.0 = r,
            None => return false
        }

        match location.1.checked_add_signed (movement.1) {
            Some (c) => destination.1 = c,
            None => return false
        }

        if !self.is_placeable (&destination) {
            return false;
        }

        // TODO: Check heights
        true
    }

    pub fn place_character (&mut self, location: Location, character_id: ID) -> bool {
        assert! (self.is_in_bounds (&location));

        if self.is_placeable (&location) {
            self.character_locations.insert ((character_id, location));

            true
        } else {
            false
        }
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

                display.push_str (&format! ("{}{} ",
                        match self.terrains.get (&tile.terrain_id) {
                            Some (t) => t,
                            None => panic! ("Unknown terrain ID {}", tile.terrain_id),
                        },
                        tile.height
                ));
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
        let tiles: TileMapBuilder = generate_tile_map_builder ();

        Map::new (terrains, tiles)
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
    fn tile_is_impassable () {
        let terrains: Rc<HashMap<u8, Terrain>> = Rc::new (generate_terrains ()); 

        // Test passable tile
        let tile_builder: TileBuilder = TileBuilder::new (0, 0);
        let tile: Tile = tile_builder.build (Rc::clone (&terrains));
        assert! (!tile.is_impassable ());

        // Test impassable tile
        let tile_builder: TileBuilder = TileBuilder::new (2, 0);
        let tile: Tile = tile_builder.build (Rc::clone (&terrains));
        assert! (tile.is_impassable ());
    }

    #[test]
    fn tile_map_builder_data () {
        let tile_map_builder: TileMapBuilder = generate_tile_map_builder ();

        assert_eq! (tile_map_builder.get_rows (), 2);
        assert_eq! (tile_map_builder.get_columns (), 3);
    }

    #[test]
    fn tile_map_builder_build () {
        let tile_map_builder: TileMapBuilder = generate_tile_map_builder ();
        let terrains: Rc<HashMap<u8, Terrain>> = Rc::new (generate_terrains ());
        let tiles: Vec<Vec<Tile>> = tile_map_builder.build (Rc::clone (&terrains));

        assert_eq! (Rc::strong_count (&terrains), 7);
        assert_eq! (tiles[0][0].get_terrain_id (), 0);
        assert_eq! (tiles[0][0].get_height (), 0);
        assert_eq! (tiles[0][0].is_impassable (), false);
        assert_eq! (tiles[0][0].get_terrain_id (), 0);
        assert_eq! (tiles[0][1].get_height (), 1);
        assert_eq! (tiles[0][1].is_impassable (), false);
        assert_eq! (tiles[0][2].get_terrain_id (), 0);
        assert_eq! (tiles[0][2].get_height (), 0);
        assert_eq! (tiles[0][2].is_impassable (), false);

        assert_eq! (tiles[1][0].get_terrain_id (), 1);
        assert_eq! (tiles[1][0].get_height (), 2);
        assert_eq! (tiles[1][0].is_impassable (), false);
        assert_eq! (tiles[1][0].get_terrain_id (), 1);
        assert_eq! (tiles[1][1].get_height (), 1);
        assert_eq! (tiles[1][1].is_impassable (), false);
        assert_eq! (tiles[1][2].get_terrain_id (), 2);
        assert_eq! (tiles[1][2].get_height (), 0);
        assert_eq! (tiles[1][2].is_impassable (), true);
    }

    #[test]
    fn map_place_character () {
        let mut map: Map = generate_map ();

        assert_eq! (map.is_in_bounds (&(1, 1)), true);
        assert_eq! (map.is_placeable (&(1, 1)), true);
        assert_eq! (map.place_character ((1, 1), 0), true);
        assert_eq! (map.get_character (&(1, 1)).unwrap (), &0);
        assert_eq! (map.is_placeable (&(1, 1)), false);
        assert_eq! (map.place_character ((1, 1), 1), false);
        assert_eq! (map.get_character (&(1, 1)).unwrap (), &0);
    }

    #[test]
    fn map_move_character () {
        let mut map: Map = generate_map ();

        todo! ("Write tests");
    }
}
