use danubia::system::{Game, Logger, Reader};
use danubia::Scene;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use std::time::Duration;
use std::error::Error;
use std::io;
use std::sync::mpsc;
use std::thread;

fn main () -> Result<(), Box<dyn Error>> {
    // SDL2 boilerplate
    let sdl = sdl2::init ()?;
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
        for event in event_pump.poll_iter () {
            match event {
                Event::Quit { .. } | Event::KeyDown { keycode: Some (Keycode::Escape), .. } => break 'running,
                _ => (),
            }
        }

        canvas.clear ();
        canvas.present ();
        thread::sleep (Duration::new (0, 1_000_000_000u32 / 30));

        game.do_turn ();
    }

    Ok (())
}
