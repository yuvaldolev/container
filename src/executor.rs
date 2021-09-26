use crate::container::Container;

pub struct Executor;

impl Executor {
    pub fn new() -> Self {
        return Self;
    }

    pub fn execute(&self, container: &Container, command: &Vec<String>) {}
}
