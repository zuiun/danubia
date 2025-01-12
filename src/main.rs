use danubia::system::{Game, Logger, Reader};
use danubia::Scene;
use std::io::{self, StdinLock};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;

fn main () {
    // println! ("Hello world!");
    let scene: Scene = Scene::default ();
    let stdin: StdinLock = io::stdin ().lock ();
    let reader: Reader<StdinLock> = Reader::new (stdin);
    let (sender, receiver): (Sender<String>, Receiver<String>) = mpsc::channel ();
    let mut game: Game<StdinLock> = Game::new (scene, reader, sender);

    thread::spawn (move || Logger::new ("log.txt", receiver).do_log ());

    game.do_game ();
}
