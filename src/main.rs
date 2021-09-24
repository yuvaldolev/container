use clap::Clap;

use container::Opts;

fn main() -> anyhow::Result<()> {
    let opts = Opts::parse();
    container::run(opts)?;

    Ok(())
}
