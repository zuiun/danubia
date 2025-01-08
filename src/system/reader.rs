use super::Validator;
use std::fmt::Debug;
use std::io::BufRead;

#[derive (Debug)]
pub struct Reader<R: BufRead> {
    stream: R,
}

impl<R: BufRead> Reader<R> {
    pub fn new (stream: R) -> Self {
        Self { stream }
    }

    // This was written as a test and is unlikely to ever be used
    // fn read_byte (&mut self) -> Option<u8> {
    //     let mut input: [u8; 1] = [b'0'];

    //     if self.stream.fill_buf ()
    //             .unwrap_or_else (|e| panic! ("{}", e))
    //             .is_empty () {
    //         None
    //     } else {
    //         self.stream.read_exact (&mut input)
    //                 .unwrap_or_else (|e| panic! ("{}", e));

    //         Some (input[0])
    //     }
    // }

    fn read_line (&mut self) -> String {
        let mut input: String = String::new ();

        self.stream.read_line (&mut input)
                .unwrap_or_else (|e| panic! ("{}", e));

        input.to_lowercase ()
    }

    pub fn read_validate<T> (&mut self, validator: &impl Validator<T>) -> Option<T> {
        print! ("{}: ", validator.get_prompt ());

        loop {
            let input: String = self.read_line ();
            let input: &str = input.trim ();
            let result: Result<Option<T>, String> = validator.validate (input);

            println! ("{}", input);

            if let Err (e) = result {
                println! ("{:?}", e);
            } else {
                return result.unwrap_or_else (|e| panic! ("{}", e))
            }
        }
    }
}

#[cfg (test)]
mod tests {
    use super::*;

    struct LessThanFiveValidator {
        prompt: &'static str,
    }

    impl LessThanFiveValidator {
        pub fn new () -> Self {
            let prompt: &'static str = "test validator";

            Self { prompt }
        }
    }

    impl Validator<bool> for LessThanFiveValidator {
        fn validate (&self, input: &str) -> Result<Option<bool>, String> {
            let input: u8 = input.parse ().unwrap_or_else (|e| panic! ("{}", e));

            Ok (Some (input < 5))
        }

        fn get_prompt (&self) -> &str {
            self.prompt
        }
    }

    fn generate_reader<R: BufRead> (stream: R) -> Reader<R> {
        Reader::new (stream)
    }

    // #[test]
    // fn reader_read_byte () {
    //     let mut reader = generate_reader (&b"0123456789"[..]);

    //     assert_eq! (reader.read_byte ().unwrap (), b'0');
    //     assert_eq! (reader.read_byte ().unwrap (), b'1');
    //     assert_eq! (reader.read_byte ().unwrap (), b'2');
    //     assert_eq! (reader.read_byte ().unwrap (), b'3');
    //     assert_eq! (reader.read_byte ().unwrap (), b'4');
    //     assert_eq! (reader.read_byte ().unwrap (), b'5');
    //     assert_eq! (reader.read_byte ().unwrap (), b'6');
    //     assert_eq! (reader.read_byte ().unwrap (), b'7');
    //     assert_eq! (reader.read_byte ().unwrap (), b'8');
    //     assert_eq! (reader.read_byte ().unwrap (), b'9');
    //     assert! (reader.read_byte ().is_none ());
    // }

    #[test]
    fn reader_read_line () {
        let mut reader = generate_reader (&b"0123456789"[..]);

        assert_eq! (reader.read_line (), "0123456789");
        assert_eq! (reader.read_line (), "");
    }

    #[test]
    fn reader_read_validate () {
        let mut reader = generate_reader (&b"0\n1\n2\n3\n4\n5\n6\n7\n8\n9"[..]);
        let validator = LessThanFiveValidator::new ();
        assert! (reader.read_validate (&validator).unwrap ());
        assert! (reader.read_validate (&validator).unwrap ());
        assert! (reader.read_validate (&validator).unwrap ());
        assert! (reader.read_validate (&validator).unwrap ());
        assert! (reader.read_validate (&validator).unwrap ());
        assert! (!reader.read_validate (&validator).unwrap ());
        assert! (!reader.read_validate (&validator).unwrap ());
        assert! (!reader.read_validate (&validator).unwrap ());
        assert! (!reader.read_validate (&validator).unwrap ());
        assert! (!reader.read_validate (&validator).unwrap ());
    }
}
