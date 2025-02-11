use crate::common::{ID, Scene};
use crate::map::Location;
use sdl2::image::LoadTexture;
use sdl2::rect::Rect;
use sdl2::render::{Canvas, Texture, TextureCreator};
use sdl2::video::{Window, WindowContext};
use std::error::Error;
use std::path::Path;

const TILE_LENGTH: u32 = 16;
const TILE_HEIGHT: u32 = 16;

pub struct RenderContext {
    terrains: Vec<Vec<ID>>,
    unit_locations: Vec<Option<Location>>,
}

impl RenderContext {
    pub fn new (terrains: Vec<Vec<ID>>, unit_locations: Vec<Option<Location>>) -> Self {
        Self { terrains, unit_locations }
    }

    pub fn get_terrains (&self) -> &[Vec<ID>] {
        &self.terrains
    }

    pub fn get_unit_locations (&self) -> &[Option<Location>] {
        &self.unit_locations
    }
}

pub struct Renderer {
    terrains: Vec<Texture>,
    units: Vec<Texture>,
}

impl Renderer {
    pub fn new (texture_creator: &TextureCreator<WindowContext>, scene: &Scene) -> Result<Renderer, Box<dyn Error>> {
        let mut terrains: Vec<Texture> = Vec::new ();
        let mut units: Vec<Texture> = Vec::new ();

        for texture in scene.textures_terrain_iter () {
            let texture: Texture = texture_creator.load_texture (Path::new (texture))?;

            terrains.push (texture);
        }

        for texture in scene.textures_unit_iter () {
            let texture: Texture = texture_creator.load_texture (Path::new (texture))?;

            units.push (texture);
        }

        // TODO: ...

        Ok (Renderer { terrains, units })
    }

    pub fn render (&self, canvas: &mut Canvas<Window>, context: &RenderContext) {
        // TODO: Optimisation: don't have to redraw terrain if no camera movement/change in terrain
        // row (i) = y, column (j) = x

        for (i, row) in context.get_terrains ().iter ().enumerate () {
            let i: i32 = i as i32;

            for (j, terrain_id) in row.iter ().enumerate () {
                let j: i32 = j as i32;
                let x: i32 = j * (TILE_HEIGHT as i32);
                let y: i32 = i * (TILE_LENGTH as i32);
                let destination: Option<Rect> = Some (Rect::new (x, y, TILE_LENGTH, TILE_HEIGHT));

                canvas.copy (&self.terrains[*terrain_id], None, destination).unwrap ();
            }
        }

        for (unit_id, location) in context.get_unit_locations ().iter ().enumerate () {
            if let Some (location) = location {
                let x: i32 = (location.1 as i32) * (TILE_HEIGHT as i32);
                let y: i32 = (location.0 as i32) * (TILE_LENGTH as i32);
                let destination: Option<Rect> = Some (Rect::new (x, y, TILE_LENGTH, TILE_HEIGHT));
                canvas.copy (&self.units[unit_id], None, destination).unwrap ();
            }
        }
    }
}
