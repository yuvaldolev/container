use clap::{AppSettings, Clap};

#[derive(Clap)]
#[clap(version = "1.0", author = "ydolev")]
#[clap(setting = AppSettings::ColoredHelp)]
pub struct Opts {
    pub mount: String,
    pub uid: u32,
    pub command: Vec<String>,
}
