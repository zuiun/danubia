use danubia::system::Reader;
use danubia::Danubia;
use std::error::Error;
use std::io;

fn main () -> Result<(), Box<dyn Error>> {
    let stdin = io::stdin ().lock ();
    let reader = Reader::new (stdin);
    let mut danubia = Danubia::new (reader)?;

    danubia.run ()
}
