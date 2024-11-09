use std::{collections::HashMap, fmt};

pub type ID = u8; // Up to 256 unique entities
pub type Cursor = (usize, usize);

pub enum Direction {
    Up,
    Right,
    Down,
    Left
}

pub struct Character {

}

#[derive (Debug)]
pub struct Terrain {
    display: char,
}

#[derive (Debug)]
pub struct Tile {
    terrain_id: ID,
    occupant_id: Option<ID>,
    height: u8,
    is_impassable: bool,
}

#[derive (Debug)]
pub struct Map {
    map: Vec<Vec<Tile>>,
    terrains: UniqueManager<Terrain>,
    cursor: Cursor
}

#[derive (Debug)]
pub struct UniqueManager<T> {
    map: HashMap<ID, T>,
    current_id: ID
}

impl Terrain {
    pub fn new (display: char) -> Self {
        Self { display }
    }
}

impl Tile {
    pub fn new (terrain_id: ID, height: u8, is_impassable: bool) -> Self {
        Self { terrain_id, occupant_id: None, height, is_impassable }
    }
}

impl Map {
    pub fn new () -> Self {
        Self { map: Vec::new (), terrains: UniqueManager::new (), cursor: (0, 0) }
    }

    pub fn build (map: Vec<Vec<Tile>>, terrains: UniqueManager<Terrain>) -> Self {
        Self { map, terrains, cursor: (0, 0) }
    }

    pub fn move_cursor (&mut self, direction: Direction) -> Option<Cursor> {
        match direction {
            Direction::Up => if self.cursor.0 > 0 {
                self.cursor.0 -= 1;

                Some (self.cursor)
            } else {
                None
            }
            Direction::Right => if self.cursor.1 < self.map[0].len () - 1 {
                self.cursor.1 += 1;

                Some (self.cursor)
            } else {
                None
            }
            Direction::Down => if self.cursor.0 < self.map.len () - 1 {
                self.cursor.0 += 1;

                Some (self.cursor)
            } else {
                None
            }
            Direction::Left => if self.cursor.1 > 0 {
                self.cursor.1 -= 1;

                Some (self.cursor)
            } else {
                None
            }
        }
    }

    pub fn get_cursor (&self) -> Cursor {
        self.cursor
    }
}

impl<T> UniqueManager<T> {
    pub fn new () -> Self {
        Self { map: HashMap::new (), current_id: 0 }
    }

    pub fn add (&mut self, item: T) -> () {
        self.map.insert (self.current_id, item);
        self.current_id += 1;
    }

    pub fn get (&self, id: &ID) -> Option<&T> {
        self.map.get (id)
    }
}

impl fmt::Display for Terrain {
    fn fmt (&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write! (f, "{}", self.display)
    }
}

impl fmt::Display for Tile {
    fn fmt (&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.occupant_id {
            Some (id) => write! (f, "{}_{}", id, self.height),
            None => write! (f, "{}.{}", self.terrain_id, self.height)
        }
    }
}

impl fmt::Display for Map {
    fn fmt (&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut display: String = String::from ("");

        for i in 0 .. self.map.len () {
            for j in 0 .. self.map[i].len () {
                let tile: &Tile = &self.map[i][j];

                if self.cursor.0 == i && self.cursor.1 == j {
                    display.push_str (">");
                } else {
                    display.push_str (" ");
                }

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
