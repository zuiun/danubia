use danubia::Danubia;
use std::error::Error;

fn main () -> Result<(), Box<dyn Error>> {
    let mut danubia = Danubia::new ()?;

    danubia.run ()
}
