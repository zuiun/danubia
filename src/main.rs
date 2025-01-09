use danubia::system::{Game, Reader};
use danubia::Scene;
use std::io::{self, StdinLock};

fn main () {
    println! ("Hello world!");
    let scene: Scene = Scene::debug ();
    let stdin: StdinLock = io::stdin ().lock ();
    let reader: Reader<StdinLock> = Reader::new (stdin);
    let mut game: Game<StdinLock> = Game::new (scene, reader);

    game.do_game ();
}
