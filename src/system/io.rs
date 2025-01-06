// THIS IS A TEST SCRATCH FILE

fn get_input () {
    let (tx, rx): (Sender<String>, Receiver<String>) = mpsc::channel ();
    let _ = thread::spawn (move || {
        let mut input: String = String::new ();
 
        let _ = io::stdin ().read_line (&mut input);

        let _ = tx.send (input);
    });

    let _ = rx.recv ();
}

fn read_test<F> (&mut self, prompt: &str, mut validator: F) -> char
where F: FnMut (char) -> bool {
    print! ("{}: ", prompt);

    let mut input: char = self.read_char ();

    while !validator (input) {
        input = self.read_char ();
    }

    input
}

fn read_test (&mut self, prompt: &str, inputs_valid: &[char]) -> char {
    print! ("{}: ", prompt);

    let mut input: char = self.read_char ();

    while !inputs_valid.contains (&input) {
        input = self.read_char ();
    }

    input
}

fn read_input<F, T> (&mut self, prompt: &str, mut validator: F) -> T
where F: FnMut (u8) -> Option<T> {
    print! ("{}: ", prompt);

    let mut input: u8 = self.read_char () as u8;
    let mut result: Option<T> = validator (input);

    while result.is_none () {
        input = self.read_char () as u8;
        result = validator (input);
    }

    result.unwrap ()
}

fn read_char (&mut self) -> char {
    let mut input: [u8; 1] = [b'0'];

    self.reader.read_exact (&mut input).unwrap_or_else (|e| panic! ("{:?}", e));

    char::from (input[0])
}
