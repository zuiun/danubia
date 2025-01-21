use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Write};
use std::sync::mpsc::Receiver;
use std::time::Instant;

#[derive (Debug)]
pub struct Logger {
    stream: BufWriter<File>,
    receiver: Receiver<String>,
    start: Instant,
}

impl Logger {
    pub fn new (file: &str, receiver: Receiver<String>) -> Self {
        let file: File = OpenOptions::new ()
                .write (true)
                .create (true)
                .truncate (true)
                .open (file)
                .expect ("Log creation failed");
        let stream: BufWriter<File> = BufWriter::new (file);
        let start: Instant = Instant::now ();

        Self { stream, receiver, start }
    }

    pub fn run (&mut self) {
        while let Ok (message) = self.receiver.recv () {
            let elapsed: f32 = self.start.elapsed ().as_secs_f32 ();
            let message: String = format! ("[{:.2}]: {}\n", elapsed, message);
            let message: &[u8] = message.as_bytes ();

            self.stream.write_all (message).unwrap_or_else (|e| println! ("{}", e));
        }

        self.stream.flush ().unwrap_or_else (|e| println! ("{}", e));
    }
}
