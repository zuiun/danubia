use danubia::{ID, Direction, UniqueManager, Map, Terrain, Tile};

fn main () {
    println! ("Hello world!");
}

#[cfg (test)]
mod tests {
    use super::*;

    fn build_terrains () -> UniqueManager<Terrain> {
        let grass: Terrain = Terrain::new ('g'); // 0
        let dirt: Terrain = Terrain::new ('d'); // 1
        let stone: Terrain = Terrain::new ('s'); // 2
        let mut terrains: UniqueManager<Terrain> = UniqueManager::new ();

        terrains.add (grass);
        terrains.add (dirt);
        terrains.add (stone);

        terrains
    }

    fn build_map (terrains: UniqueManager<Terrain>) -> Map {
        Map::build (vec![
            vec![Tile::new (0, 0, false), Tile::new (0, 0, false), Tile::new (0, 0, false)], // g0 g0 g0
            vec![Tile::new (1, 0, false), Tile::new (1, 0, false), Tile::new (1, 0, false)], // d0 d0 d0
            vec![Tile::new (2, 0, false), Tile::new (1, 0, false), Tile::new (2, 0, false)], // s0 s0 s0
            vec![Tile::new (0, 1, false), Tile::new (0, 2, false), Tile::new (0, 3, false)] // g1 g2 g3
        ], terrains)
    }

    #[test]
    fn terrains_build () {
        let terrains: UniqueManager<Terrain> = build_terrains ();

        assert_eq! (terrains.get (&0).unwrap ().to_string (), "g");
        assert_eq! (terrains.get (&1).unwrap ().to_string (), "d");
        assert_eq! (terrains.get (&2).unwrap ().to_string (), "s");
    }

    #[test]
    fn map_build () {
        let terrains: UniqueManager<Terrain> = build_terrains ();
        let map: Map = build_map (terrains);

        assert_eq! (map.to_string (),
                ">g0  g0  g0 \n \
                d0  d0  d0 \n \
                s0  d0  s0 \n \
                g1  g2  g3 \n");
    }

    #[test]
    fn map_cursor () {
        let terrains: UniqueManager<Terrain> = build_terrains ();
        let mut map: Map = build_map (terrains);

        assert_eq! (map.get_cursor (), (0, 0));
        assert_eq! (map.move_cursor (Direction::Down), Some ((1, 0)));
        assert_eq! (map.move_cursor (Direction::Down), Some ((2, 0)));
        assert_eq! (map.move_cursor (Direction::Down), Some ((3, 0)));
        assert_eq! (map.get_cursor (), (3, 0));
        assert_eq! (map.move_cursor (Direction::Down), None);
        assert_eq! (map.to_string (),
                " g0  g0  g0 \n \
                d0  d0  d0 \n \
                s0  d0  s0 \n\
                >g1  g2  g3 \n");
        assert_eq! (map.get_cursor (), (3, 0));
        assert_eq! (map.move_cursor (Direction::Up), Some ((2, 0)));
        assert_eq! (map.move_cursor (Direction::Up), Some ((1, 0)));
        assert_eq! (map.move_cursor (Direction::Up), Some ((0, 0)));
        assert_eq! (map.get_cursor (), (0, 0));
        assert_eq! (map.move_cursor (Direction::Up), None);
        assert_eq! (map.to_string (),
                ">g0  g0  g0 \n \
                d0  d0  d0 \n \
                s0  d0  s0 \n \
                g1  g2  g3 \n");
        assert_eq! (map.get_cursor (), (0, 0));
        assert_eq! (map.move_cursor (Direction::Right), Some ((0, 1)));
        assert_eq! (map.move_cursor (Direction::Right), Some ((0, 2)));
        assert_eq! (map.get_cursor (), (0, 2));
        assert_eq! (map.move_cursor (Direction::Right), None);
        assert_eq! (map.to_string (),
                " g0  g0 >g0 \n \
                d0  d0  d0 \n \
                s0  d0  s0 \n \
                g1  g2  g3 \n");
        assert_eq! (map.get_cursor (), (0, 2));
        assert_eq! (map.move_cursor (Direction::Left), Some ((0, 1)));
        assert_eq! (map.move_cursor (Direction::Left), Some ((0, 0)));
        assert_eq! (map.get_cursor (), (0, 0));
        assert_eq! (map.move_cursor (Direction::Left), None);
        assert_eq! (map.get_cursor (), (0, 0));
        assert_eq! (map.to_string (),
                ">g0  g0  g0 \n \
                d0  d0  d0 \n \
                s0  d0  s0 \n \
                g1  g2  g3 \n");
    }
}
