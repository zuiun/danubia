mod city;
pub use self::city::City;
mod grid;
pub use self::grid::Location;
pub use self::grid::Direction;
pub use self::grid::Grid;
mod terrain;
pub use self::terrain::Terrain;
mod tile;
pub use self::tile::Tile;
pub use self::tile::TileBuilder;

pub const COST_IMPASSABLE: u8 = 0;
pub const COST_MINIMUM: u8 = 1;

#[derive (Debug)]
#[derive (Clone, Copy)]
pub enum Area {
    Single,
    Radial (u8), // radius
    Path (u8), // width
}

#[derive (Debug)]
#[derive (Clone, Copy)]
pub enum Search {
    Single,
    Radial (u8), // range
    Path (u8, u8, Direction), // width, range, direction
}
