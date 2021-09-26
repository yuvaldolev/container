use config::{Config, ConfigError, Environment, File};

#[derive(Debug, Deserialize)]
pub struct Disk {
    pub root_dir: String,
    pub images_dir: String,
    pub containers_dir: String,
}

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub disk: Disk,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let mut settings = Config::default();

        // Start off by merging in the "default" configuration file.
        settings.merge(File::with_name("config/default"))?;

        // Add in settings from the environment (with a prefix of APP)
        // Eg.. `APP_DEBUG=1 ./target/app` would set the `debug` key.
        settings.merge(Environment::with_prefix("app"))?;

        // Deserialize (and thus freeze) the entire configuration.
        settings.try_into()
    }
}
