use std::error::Error;

use clap::Clap;

use container::Opts;

fn main() -> Result<(), Box<dyn Error>> {
    let opts = Opts::parse();
    container::run(opts)?;

    Ok(())
}
