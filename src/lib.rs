use crate::common::Scene;
use crate::system::{Game, Logger, Renderer};
use sdl2::event::Event;
use sdl2::image::{self as sdl2_image, Sdl2ImageContext};
use sdl2::keyboard::Keycode;
use sdl2::mixer::{self as sdl2_mixer, Sdl2MixerContext};
use sdl2::pixels::Color;
use sdl2::render::{Canvas, TextureCreator};
use sdl2::ttf::{self as sdl2_ttf, Sdl2TtfContext};
use sdl2::video::{Window, WindowContext};
use sdl2::{EventPump, Sdl, VideoSubsystem};
use std::error::Error;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::{Duration, Instant};

pub mod character;
pub mod collections;
pub mod common;
pub mod controller;
pub mod dynamic;
pub mod event;
pub mod map;
pub mod system;

const FRAMES_PER_SECOND: u128 = 30;
const NANOS_PER_FRAME: u128 = 1_000_000_000 / FRAMES_PER_SECOND;

pub struct Danubia {
    image: Sdl2ImageContext,
    mixer: Sdl2MixerContext,
    ttf: Sdl2TtfContext,
    canvas: Canvas<Window>,
    event_pump: EventPump,
    renderer: Renderer,
    game: Game,
}

impl Danubia {
    pub fn new () -> Result<Self, Box<dyn Error>> {
        // SDL2 boilerplate
        let sdl: Sdl = sdl2::init ()?;
        let image: Sdl2ImageContext = sdl2_image::init (sdl2_image::InitFlag::PNG)?;
        let mixer: Sdl2MixerContext = sdl2_mixer::init (sdl2_mixer::InitFlag::all ())?;
        let ttf: Sdl2TtfContext = sdl2_ttf::init ()?;
        let video: VideoSubsystem = sdl.video ()?;
        let window: Window = video.window ("Danubia", 640, 480)
                .position_centered ()
                .build ()?;
        let mut canvas: Canvas<Window> = window.into_canvas ().build ()?;
        let texture_creator: TextureCreator<WindowContext> = canvas.texture_creator ();
        let event_pump: EventPump = sdl.event_pump ()?;

        canvas.set_draw_color (Color::RGB (255, 255, 255));
        canvas.clear ();
        canvas.present ();

        let (sender, receiver): (Sender<String>, Receiver<String>) = mpsc::channel ();

        thread::spawn (move || Logger::new ("log.txt", receiver).run ());

        let scene: Scene = Scene::default ();
        let renderer: Renderer = Renderer::new (&texture_creator, &scene)?;
        let mut game: Game = Game::new (scene, sender);

        game.init ()?;

        Ok (Danubia { image, mixer, ttf, canvas, event_pump, renderer, game })
    }

    pub fn run (&mut self) -> Result<(), Box<dyn Error>> {
        let mut is_display_turn: bool = true;
        let mut is_display_prompt: bool = true;

        'running: loop {
            let frame_start: Instant = Instant::now ();
            let mut input: Option<Keycode> = None;

            if is_display_turn {
                self.game.display_turn ();
                is_display_turn = false;
            }

            if is_display_prompt {
                self.game.display_prompt ();
                is_display_prompt = false;
            }

            for event in self.event_pump.poll_iter () {
                match event {
                    Event::Quit { .. } => break 'running,
                    // TODO: KeyDown or KeyUp?
                    Event::KeyDown { keycode: Some (keycode), repeat: false, .. } => {
                        match keycode {
                            Keycode::Escape => break 'running,
                            _ => {
                                input = Some (keycode);
                                println! ("{}", keycode.name ());
                            }
                        }
                    }
                    _ => (),
                }
            }

            if let Some (input) = input {
                is_display_prompt = true;

                if self.game.update (input) {
                    is_display_turn = true;
                }
            }

            self.canvas.clear ();
            // TODO: Render context
            self.renderer.render (&mut self.canvas, &self.game.get_render_context ());
            self.canvas.present ();

            let frame_elapsed: u128 = frame_start.elapsed ().as_nanos ();

            if let Some (frame_sleep) = NANOS_PER_FRAME.checked_sub (frame_elapsed) {
                thread::sleep (Duration::new (0, frame_sleep as u32));
            }
        }

        Ok (())
    }
}

pub mod tests {    
    use super::*;
    use common::Scene;
    use std::rc::Rc;

    pub fn generate_scene () -> Rc<Scene> {
        Rc::new (Scene::default ())
    }
}
