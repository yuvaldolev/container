#[macro_use(defer)]
extern crate scopeguard;

#[macro_use]
extern crate serde_derive;

mod container;
// mod executor;
mod image;
mod invalid_command_error;
mod opts;
mod settings;

pub use opts::Opts;

// use std::error::Error;
use std::str::Utf8Error;

use container::Container;
// use executor::Executor;
use image::Image;
use settings::Settings;

pub fn run(opts: Opts) -> anyhow::Result<()> {
    println!(
        "Error: {}",
        invalid_command_error::InvalidCommandError::Empty
    );
    // Read the configuration.
    let settings = Settings::new()?;

    // Create an Image instance to validate and wrap the given image.
    let image = Image::new(opts.image.clone(), &settings)?;

    // Create a new container with the given image.
    let container = Container::new(image.clone(), &settings)?;

    // Execute the given command in the container.
    // let executor = Executor::new();
    // executor.execute(&container, &opts.command);

    test()?;

    Ok(())
}

fn test() -> anyhow::Result<()> {
    Err(invalid_command_error::InvalidCommandError::Empty.into())
}
