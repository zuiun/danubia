mod city;
pub use self::city::*;
mod grid;
pub use self::grid::*;
mod terrain;
pub use self::terrain::*;
mod tile;
pub use self::tile::*;

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
