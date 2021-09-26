use clap::{AppSettings, Clap};

#[derive(Clap)]
#[clap(version = "1.0", author = "ydolev")]
#[clap(setting = AppSettings::ColoredHelp)]
pub struct Opts {
    pub image: String,
    pub command: Vec<String>,
}
