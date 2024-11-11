use std::{collections::HashMap, fmt};
use crate::engine::common::{Delta, ID, Information, Modifier, Point};

#[derive (Debug)]
pub struct Terrain {
    information: Information,
    modifiers: Vec<Modifier>
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
    character_locations: HashMap<Point, Option<ID>>,
    controller_locations: HashMap<Point, Option<ID>>,
}

impl Terrain {
    pub fn new (information: Information, modifiers: Vec<Modifier> ) -> Self {
        Self { information, modifiers }
    }

    pub fn get_modifiers (&self) -> &Vec<Modifier> {
        &self.modifiers
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

        let mut character_locations: HashMap<Point, Option<ID>> = HashMap::new ();
        let mut controller_locations: HashMap<Point, Option<ID>> = HashMap::new ();

        for i in 0 .. map.len () {
            for j in 0 .. map[i].len () {
                let location: Point = (i, j);

                character_locations.insert (location, None);
                controller_locations.insert (location, None);
            }
        }

        Self { map, terrains, character_locations, controller_locations }
    }

    pub fn is_in_bounds (&self, location: Point) -> bool {
        location.0 < self.map.len () && location.1 < self.map[0].len ()
    }

    pub fn is_occupied (&self, location: Point) -> bool {
        assert! (self.is_in_bounds (location));

        match self.character_locations.get (&location) {
            Some (p) => match p {
                Some (_) => true,
                None => false
            }
            None => false // This should never happen, but if it does, then there is clearly not a character there
        }
    }

    fn is_impassable (&self, location: Point) -> bool {
        assert! (self.is_in_bounds (location));

        self.map[location.0][location.1].is_impassable ()
    }

    fn is_placeable (&self, location: Point) -> bool {
        assert! (self.is_in_bounds (location));

        !self.is_occupied (location) && !self.is_impassable (location)
    }

    pub fn is_movable (&self, location: Point, movement: Delta) -> bool {
        assert! (self.is_in_bounds (location));

        let mut destination: Point = location;

        match location.0.checked_add_signed (movement.0) {
            Some (r) => destination.0 = r,
            None => return false
        }

        match location.1.checked_add_signed (movement.1) {
            Some (c) => destination.1 = c,
            None => return false
        }

        self.is_in_bounds (destination)
    }

    pub fn place_character (&mut self, location: Point, character_id: ID) -> bool {
        if self.is_placeable (location) {
            self.character_locations.insert (location, Some (character_id));

            true
        } else {
            false
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
        let grass: Terrain = Terrain::new (Information::new (String::from ("Grass"), vec![String::from ("grass")], 0), Vec::new ());
        let dirt: Terrain = Terrain::new (Information::new (String::from ("Dirt"), vec![String::from ("dirt")], 0), Vec::new ());
        let stone: Terrain = Terrain::new (Information::new (String::from ("Stone"), vec![String::from ("stone")], 0), Vec::new ());
        let mut terrains: HashMap<ID, Terrain> = HashMap::new ();

        terrains.insert (0, grass);
        terrains.insert (1, dirt);
        terrains.insert (2, stone);

        terrains
    }

    fn generate_map (terrains: HashMap<ID, Terrain>) -> Map {
        Map::new (vec![
            vec![Tile::new (0, 0, false), Tile::new (0, 1, false), Tile::new (0, 0, true)],
            vec![Tile::new (1, 0, false), Tile::new (1, 0, false), Tile::new (1, 0, false)],
            vec![Tile::new (2, 0, false), Tile::new (1, 0, false), Tile::new (2, 0, false)],
            vec![Tile::new (0, 1, false), Tile::new (0, 2, false), Tile::new (0, 3, false)]
        ], terrains)
    }

    #[test]
    fn terrains_build () {
        let terrains: HashMap<ID, Terrain> = generate_terrains ();

        todo! ("Write tests");
    }

    #[test]
    fn map_build () {
        let terrains: HashMap<ID, Terrain> = generate_terrains ();
        let map: Map = generate_map (terrains);

        todo! ("Write tests");
    }

    // #[test]
    // fn map_cursor () {
    //     let terrains: UniqueManager<Terrain> = build_terrains ();
    //     let mut map: Map = build_map (terrains);

    //     assert_eq! (map.get_cursor (), (0, 0));
    //     assert_eq! (map.move_cursor (Direction::Down), Some ((1, 0)));
    //     assert_eq! (map.move_cursor (Direction::Down), Some ((2, 0)));
    //     assert_eq! (map.move_cursor (Direction::Down), Some ((3, 0)));
    //     assert_eq! (map.get_cursor (), (3, 0));
    //     assert_eq! (map.move_cursor (Direction::Down), None);
    //     assert_eq! (map.to_string (),
    //             " g0  g0  g0 \n \
    //             d0  d0  d0 \n \
    //             s0  d0  s0 \n\
    //             >g1  g2  g3 \n");
    //     assert_eq! (map.get_cursor (), (3, 0));
    //     assert_eq! (map.move_cursor (Direction::Up), Some ((2, 0)));
    //     assert_eq! (map.move_cursor (Direction::Up), Some ((1, 0)));
    //     assert_eq! (map.move_cursor (Direction::Up), Some ((0, 0)));
    //     assert_eq! (map.get_cursor (), (0, 0));
    //     assert_eq! (map.move_cursor (Direction::Up), None);
    //     assert_eq! (map.to_string (),
    //             ">g0  g0  g0 \n \
    //             d0  d0  d0 \n \
    //             s0  d0  s0 \n \
    //             g1  g2  g3 \n");
    //     assert_eq! (map.get_cursor (), (0, 0));
    //     assert_eq! (map.move_cursor (Direction::Right), Some ((0, 1)));
    //     assert_eq! (map.move_cursor (Direction::Right), Some ((0, 2)));
    //     assert_eq! (map.get_cursor (), (0, 2));
    //     assert_eq! (map.move_cursor (Direction::Right), None);
    //     assert_eq! (map.to_string (),
    //             " g0  g0 >g0 \n \
    //             d0  d0  d0 \n \
    //             s0  d0  s0 \n \
    //             g1  g2  g3 \n");
    //     assert_eq! (map.get_cursor (), (0, 2));
    //     assert_eq! (map.move_cursor (Direction::Left), Some ((0, 1)));
    //     assert_eq! (map.move_cursor (Direction::Left), Some ((0, 0)));
    //     assert_eq! (map.get_cursor (), (0, 0));
    //     assert_eq! (map.move_cursor (Direction::Left), None);
    //     assert_eq! (map.get_cursor (), (0, 0));
    //     assert_eq! (map.to_string (),
    //             ">g0  g0  g0 \n \
    //             d0  d0  d0 \n \
    //             s0  d0  s0 \n \
    //             g1  g2  g3 \n");
    // }

    #[test]
    fn map_place_character () {
        let terrains: HashMap<ID, Terrain> = generate_terrains ();
        let mut map: Map = generate_map (terrains);

        assert_eq! (map.is_in_bounds ((1, 1)), true);
        assert_eq! (map.is_placeable ((1, 1)), true);
        assert_eq! (map.place_character ((1, 1), 0), true);
        assert_eq! (map.is_in_bounds ((1, 1)), true);
        assert_eq! (map.is_placeable ((1, 1)), false);
        assert_eq! (map.place_character ((1, 1), 0), false);
    }
}
