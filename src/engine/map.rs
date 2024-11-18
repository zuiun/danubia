use std::{collections::HashMap, fmt};
use crate::engine::common::{ID, Information, Location, Modifier, Movement};

#[derive (Debug)]
pub struct Terrain {
    information: Information,
    modifiers: Vec<Modifier>,
    cost: u8
}

#[derive (Debug)]
pub struct Tile {
    terrain_id: ID,
    height: u8,
    is_impassable: bool
}

#[derive (Debug)]
pub struct Map {
    map: Vec<Vec<Tile>>,
    terrains: HashMap<ID, Terrain>,
    character_locations: HashMap<Location, Option<ID>>,
    controller_locations: HashMap<Location, Option<ID>>,
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
    pub fn new (terrain_id: ID, height: u8, is_impassable: bool) -> Self {
        Self { terrain_id, height, is_impassable }
    }

    pub fn get_terrain_id (&self) -> ID {
        self.terrain_id
    }

    pub fn get_height (&self) -> u8 {
        self.height
    }

    pub fn is_impassable (&self) -> bool {
        self.is_impassable
    }

    pub fn set_impassable (&mut self, is_impassable: bool) -> () {
        self.is_impassable = is_impassable;
    }
}

impl Map {
    pub fn new (map: Vec<Vec<Tile>>, terrains: HashMap<ID, Terrain>) -> Self {
        assert! (map.len () > 0);
        assert! (map[0].len () > 0);

        let mut character_locations: HashMap<Location, Option<ID>> = HashMap::new ();
        let mut controller_locations: HashMap<Location, Option<ID>> = HashMap::new ();

        for i in 0 .. map.len () {
            for j in 0 .. map[i].len () {
                let location: Location = (i, j);

                character_locations.insert (location, None);
                controller_locations.insert (location, None);
            }
        }

        Self { map, terrains, character_locations, controller_locations }
    }

    pub fn is_in_bounds (&self, location: Location) -> bool {
        location.0 < self.map.len () && location.1 < self.map[0].len ()
    }

    pub fn is_occupied (&self, location: Location) -> bool {
        assert! (self.is_in_bounds (location));

        match self.character_locations.get (&location) {
            Some (p) => match p {
                Some (_) => true,
                None => false
            }
            None => panic! ("Location {:#?} not found", location)
        }
    }

    fn is_impassable (&self, location: Location) -> bool {
        assert! (self.is_in_bounds (location));

        self.map[location.0][location.1].is_impassable ()
    }

    fn is_placeable (&self, location: Location) -> bool {
        assert! (self.is_in_bounds (location));

        !self.is_occupied (location) && !self.is_impassable (location)
    }

    pub fn is_movable (&self, location: Location, movement: Movement) -> bool {
        assert! (self.is_in_bounds (location));

        let mut destination: Location = location;

        match location.0.checked_add_signed (movement.0) {
            Some (r) => destination.0 = r,
            None => return false
        }

        match location.1.checked_add_signed (movement.1) {
            Some (c) => destination.1 = c,
            None => return false
        }

        if !self.is_placeable (destination) {
            return false;
        }

        // TODO: Check heights
        true
    }

    pub fn place_character (&mut self, location: Location, character_id: ID) -> bool {
        assert! (self.is_in_bounds (location));

        if self.is_placeable (location) {
            self.character_locations.insert (location, Some (character_id));

            true
        } else {
            false
        }
    }

    pub fn get_character (&self, location: Location) -> Option<ID> {
        assert! (self.is_in_bounds (location));

        match self.character_locations.get (&location) {
            Some (c) => *c,
            None => panic! ("Location {:#?} not found", location)
        }
    }

    pub fn get_controller (&self, location: Location) -> Option<ID> {
        assert! (self.is_in_bounds (location));

        match self.controller_locations.get (&location) {
            Some (c) => *c,
            None => panic! ("Location {:#?} not found", location)
        }
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
        let grass: Terrain = Terrain::new (Information::new (String::from ("Grass"), vec![String::from ("grass")], 0), Vec::new (), 0);
        let dirt: Terrain = Terrain::new (Information::new (String::from ("Dirt"), vec![String::from ("dirt")], 0), Vec::new (), 1);
        let stone: Terrain = Terrain::new (Information::new (String::from ("Stone"), vec![String::from ("stone")], 0), Vec::new (), 2);
        let mut terrains: HashMap<ID, Terrain> = HashMap::new ();

        terrains.insert (0, grass);
        terrains.insert (1, dirt);
        terrains.insert (2, stone);

        terrains
    }

    fn generate_map (terrains: HashMap<ID, Terrain>) -> Map {
        Map::new (vec![
            vec![Tile::new (0, 0, false), Tile::new (0, 1, false), Tile::new (0, 0, true)],
            vec![Tile::new (1, 2, false), Tile::new (1, 1, false), Tile::new (2, 0, false)]
        ], terrains)
    }

    #[test]
    fn terrains_build () {
        let terrains: HashMap<ID, Terrain> = generate_terrains ();

        assert_eq! (terrains.get (&0).unwrap ().get_modifiers ().len (), 0);
        assert_eq! (terrains.get (&0).unwrap ().get_cost (), 0);
        assert_eq! (terrains.get (&1).unwrap ().get_modifiers ().len (), 0);
        assert_eq! (terrains.get (&1).unwrap ().get_cost (), 1);
        assert_eq! (terrains.get (&2).unwrap ().get_modifiers ().len (), 0);
        assert_eq! (terrains.get (&2).unwrap ().get_cost (), 2);
    }

    #[test]
    fn map_build () {
        let terrains: HashMap<ID, Terrain> = generate_terrains ();
        let map: Map = generate_map (terrains);

        assert_eq! (map.map[0][0].get_terrain_id (), 0);
        assert_eq! (map.map[0][0].get_height (), 0);
        assert_eq! (map.map[0][0].is_impassable (), false);
        assert_eq! (map.map[0][0].get_terrain_id (), 0);
        assert_eq! (map.map[0][1].get_height (), 1);
        assert_eq! (map.map[0][1].is_impassable (), false);
        assert_eq! (map.map[0][2].get_terrain_id (), 0);
        assert_eq! (map.map[0][2].get_height (), 0);
        assert_eq! (map.map[0][2].is_impassable (), true);

        assert_eq! (map.map[1][0].get_terrain_id (), 1);
        assert_eq! (map.map[1][0].get_height (), 2);
        assert_eq! (map.map[1][0].is_impassable (), false);
        assert_eq! (map.map[1][0].get_terrain_id (), 1);
        assert_eq! (map.map[1][1].get_height (), 1);
        assert_eq! (map.map[1][1].is_impassable (), false);
        assert_eq! (map.map[1][2].get_terrain_id (), 2);
        assert_eq! (map.map[1][2].get_height (), 0);
        assert_eq! (map.map[1][2].is_impassable (), false);
    }

    #[test]
    fn map_place_character () {
        let terrains: HashMap<ID, Terrain> = generate_terrains ();
        let mut map: Map = generate_map (terrains);

        assert_eq! (map.is_in_bounds ((1, 1)), true);
        assert_eq! (map.is_placeable ((1, 1)), true);
        assert_eq! (map.place_character ((1, 1), 0), true);
        assert_eq! (map.get_character ((1, 1)).unwrap (), 0);
        
        assert_eq! (map.is_in_bounds ((1, 1)), true);
        assert_eq! (map.is_placeable ((1, 1)), false);
        assert_eq! (map.place_character ((1, 1), 1), false);
        assert_eq! (map.get_character ((1, 1)).unwrap (), 0);
    }

    #[test]
    fn map_move_character () {
        let terrains: HashMap<ID, Terrain> = generate_terrains ();
        let mut map: Map = generate_map (terrains);

        todo! ("Write tests");
    }
}
