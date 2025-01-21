use crate::common::Scene;
use crate::system::{Game, Logger, Reader};
use sdl2::event::Event;
use sdl2::image::{self as sdl2_image, Sdl2ImageContext};
use sdl2::keyboard::Keycode;
use sdl2::mixer::{self as sdl2_mixer, Sdl2MixerContext};
use sdl2::pixels::Color;
use sdl2::render::Canvas;
use sdl2::ttf::{self as sdl2_ttf, Sdl2TtfContext};
use sdl2::video::Window;
use sdl2::EventPump;
use std::error::Error;
use std::io::BufRead;
use std::sync::mpsc;
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

pub struct Danubia<R: BufRead> {
    game: Game<R>,
    image: Sdl2ImageContext,
    mixer: Sdl2MixerContext,
    ttf: Sdl2TtfContext,
    canvas: Canvas<Window>,
    event_pump: EventPump,
}

impl<R: BufRead> Danubia<R> {
    pub fn new (reader: Reader<R>) -> Result<Self, Box<dyn Error>> {
        // SDL2 boilerplate
        let sdl = sdl2::init ()?;
        let image = sdl2_image::init (sdl2_image::InitFlag::all ())?;
        let mixer = sdl2_mixer::init (sdl2_mixer::InitFlag::all ())?;
        let ttf = sdl2_ttf::init ()?;
        let video = sdl.video ()?;
        let window = video.window ("Danubia", 640, 480)
            .position_centered ()
            .build ()?;
        let mut canvas = window.into_canvas ().build ()?;
        let event_pump = sdl.event_pump ()?;

        canvas.set_draw_color (Color::RGB (0, 0, 0));
        canvas.clear ();
        canvas.present ();
    
        let scene = Scene::default ();
        let (sender, receiver) = mpsc::channel::<String> ();
        let mut game = Game::new (scene, reader, sender);
    
        thread::spawn (move || Logger::new ("log.txt", receiver).run ());
        game.init ()?;

        Ok (Danubia { game, image, mixer, ttf, canvas, event_pump })
    }

    pub fn run (&mut self) -> Result<(), Box<dyn Error>> {
        let mut is_display_turn = true;
        let mut is_display_prompt = true;

        'running: loop {
            let frame_start = Instant::now ();
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
                    Event::KeyUp { keycode: Some (keycode), repeat: false, .. } => {
                        match keycode {
                        // TODO: others
                        Keycode::Z => {
                            println! ("z");

                            input = Some (Keycode::Z);
                        }
                        Keycode::X => {
                            println! ("x");

                            input = Some (Keycode::X);
                        }
                        Keycode::C => {
                            println! ("c");

                            input = Some (Keycode::C);
                        }
                        Keycode::W => {
                            println! ("w");

                            input = Some (Keycode::W);
                        }
                        Keycode::A => {
                            println! ("a");

                            input = Some (Keycode::A);
                        }
                        Keycode::S => {
                            println! ("s");

                            input = Some (Keycode::S);
                        }
                        Keycode::D => {
                            println! ("d");

                            input = Some (Keycode::D);
                        }
                        Keycode::Q => {
                            println! ("q");

                            input = Some (Keycode::Q);
                        }
                        Keycode::E => {
                            println! ("e");

                            input = Some (Keycode::E);
                        }
                        Keycode::Escape => break 'running,
                        _ => println! ("Unexpected input"),
                        }
                    }
                    _ => (),
                }
            }

            // game.do_turn ();
            if let Some (input) = input {
                is_display_prompt = true;

                if self.game.update (input) {
                    is_display_turn = true;
                }
            }

            self.canvas.clear ();
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
