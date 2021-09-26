use thiserror::Error;

#[derive(Error, Debug)]
pub enum InvalidCommandError {
    #[error("empty command")]
    Empty,
}
