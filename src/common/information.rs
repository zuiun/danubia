use std::fmt::{self, Display, Formatter};

#[derive (Debug)]
pub struct Information {
    name: &'static str,
    descriptions: &'static [&'static str],
    description_current: usize,
}

impl Information {
    pub const fn new (name: &'static str, descriptions: &'static [&'static str]) -> Self {
        let description_current: usize = 0;

        Self { name, descriptions, description_current }
    }

    pub fn get_name (&self) -> &str {
        self.name
    }

    pub fn get_description (&self) -> &str {
        self.descriptions[self.description_current]
    }
}

impl Default for Information {
    fn default () -> Self {
        let name: &str = "";
        let descriptions: &[&str] = &[""];
        let description_current: usize = 0;

        Self { name, descriptions, description_current }
    }
}

impl Display for Information {
    fn fmt (&self, f: &mut Formatter<'_>) -> fmt::Result {
        write! (f, "{}\n{}", self.name, self.descriptions[self.description_current])
    }
}
