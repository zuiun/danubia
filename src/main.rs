use danubia::common::Scene;
use danubia::system::{Game, Logger, Reader};
use danubia::Danubia;
use sdl2::event::Event;
use sdl2::image as sdl2_image;
use sdl2::keyboard::Keycode;
use sdl2::mixer as sdl2_mixer;
use sdl2::pixels::Color;
use sdl2::ttf as sdl2_ttf;
use std::error::Error;
use std::io::{self, StdinLock};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

const FRAMES_PER_SECOND: u128 = 30;
const NANOS_PER_FRAME: u128 = 1_000_000_000 / FRAMES_PER_SECOND;

fn main () -> Result<(), Box<dyn Error>> {
    let stdin = io::stdin ().lock ();
    let reader = Reader::new (stdin);
    let mut danubia = Danubia::new (reader)?;

    danubia.run ()
}
