use danubia::common::Scene;
use danubia::system::{Game, Logger, Reader};
use sdl2::event::Event;
use sdl2::image as sdl2_image;
use sdl2::keyboard::Keycode;
use sdl2::mixer as sdl2_mixer;
use sdl2::pixels::Color;
use sdl2::ttf as sdl2_ttf;
use std::error::Error;
use std::io;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

const FRAMES_PER_SECOND: u128 = 30;
const NANOS_PER_FRAME: u128 = 1_000_000_000 / FRAMES_PER_SECOND;

fn main () -> Result<(), Box<dyn Error>> {
    // SDL2 boilerplate
    let sdl = sdl2::init ()?;
    let _image = sdl2_image::init (sdl2_image::InitFlag::all ())?;
    let _mixer = sdl2_mixer::init (sdl2_mixer::InitFlag::all ())?;
    let _ttf = sdl2_ttf::init ()?;
    let video = sdl.video ()?;
    let window = video.window ("Danubia", 640, 480)
        .position_centered ()
        .build ()?;
    let mut canvas = window.into_canvas ().build ()?;
    let mut event_pump = sdl.event_pump ()?;

    canvas.set_draw_color (Color::RGB (0, 0, 0));
    canvas.clear ();
    canvas.present ();

    let scene = Scene::default ();
    let stdin = io::stdin ().lock ();
    let reader = Reader::new (stdin);
    let (sender, receiver) = mpsc::channel::<String> ();
    let mut game = Game::new (scene, reader, sender);

    thread::spawn (move || Logger::new ("log.txt", receiver).do_log ());
    game.init ();

    'running: loop {
        let frame_start = Instant::now ();

        for event in event_pump.poll_iter () {
            match event {
                Event::Quit { .. } => break 'running,
                Event::KeyDown { keycode: Some (keycode), .. } => match keycode {
                    // TODO: others
                    Keycode::Z => println! ("z"),
                    Keycode::X => println! ("x"),
                    Keycode::C => println! ("c"),
                    Keycode::W => println! ("w"),
                    Keycode::A => println! ("a"),
                    Keycode::S => println! ("s"),
                    Keycode::D => println! ("d"),
                    Keycode::Q => println! ("q"),
                    Keycode::E => println! ("e"),
                    Keycode::Escape => break 'running,
                    _ => (),
                }
                _ => (),
            }
        }

        game.do_turn ();

        canvas.clear ();
        canvas.present ();

        let frame_elapsed: u128 = frame_start.elapsed ().as_nanos ();

        if let Some (frame_sleep) = NANOS_PER_FRAME.checked_sub (frame_elapsed) {
            thread::sleep (Duration::new (0, frame_sleep as u32));
        }
    }

    Ok (())
}
