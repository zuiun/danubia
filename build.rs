use std::env;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

/*
 * Calculates delay table
 * Unit speed is an index into the table
 * Regular: 21 delay at 0, 20 delay at 1, 2 delay at 77, and 1 delay at 100
 * Magic (* 1.4): 29 delay at 0, 28 delay at 1, 2 delay at 77, and 1 delay at 100
 * Wait (* 0.67): 14 delay at 0, 13 delay at 1, 2 delay at 54, and 1 delay at 77
 */
fn main () -> () {
    let out_dir: String = env::var ("OUT_DIR").unwrap ();
    let dest_path: PathBuf = Path::new (&out_dir).join ("delays.rs");
    let mut file: File = File::create (&dest_path).unwrap ();

    write! (file, "const DELAYS: [u8; 101] = [").unwrap ();

    for i in 0 .. 101 {
        write! (file, "{}", 1 + ((20 as f32 / f32::exp (0.03 * (i as f32))) as u8)).unwrap ();

        if i < 100 {
            write! (file, ", ").unwrap ();
        }
    }

    write! (file, "];\n").unwrap ();
}
