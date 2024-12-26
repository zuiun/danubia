use std::fmt;

#[derive (Debug)]
pub struct Information {
    name: String,
    descriptions: Vec<String>,
    current_description: usize
}

impl Information {
    pub fn new (name: String, descriptions: Vec<String>, current_description: usize) -> Self {
        Self { name, descriptions, current_description }
    }

    pub fn get_name (&self) -> &str {
        &self.name
    }

    pub fn get_description (&self) -> &str {
        &self.descriptions[self.current_description]
    }
}

impl fmt::Display for Information {
    fn fmt (&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write! (f, "{}\n{}", self.name, self.descriptions[self.current_description])
    }
}
