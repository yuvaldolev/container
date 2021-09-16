use clap::{AppSettings, Clap};

#[derive(Clap)]
#[clap(version = "1.0", author = "ydolev")]
#[clap(setting = AppSettings::ColoredHelp)]
pub struct Opts {
    mount: String,
    uid: u32,
    command: Vec<String>,
}
