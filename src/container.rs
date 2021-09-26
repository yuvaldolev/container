use std::fs;
use std::path::PathBuf;
use std::str::Utf8Error;

use uuid::Uuid;

use crate::{image::Image, settings::Settings};

const FS_DIR: &str = "fs";

const UUID_SIZE: usize = 32;
const CONTAINER_UUID_SIZE: usize = 12;

#[derive(Debug)]
pub struct Container {
    uuid: String,
    image: Image,
    dir: PathBuf,
    pub fs_dir: PathBuf,
}

impl Container {
    pub fn new(image: Image, settings: &Settings) -> anyhow::Result<Self> {
        let mut container = Self {
            uuid: Self::generate_uuid()?,
            image: image,
            dir: PathBuf::new(),
            fs_dir: PathBuf::new(),
        };

        // Generate the container's directory paths.
        container.dir.push(&settings.disk.containers_dir);
        container.dir.push(&container.uuid);
        container.fs_dir = container.dir.join(FS_DIR);

        // Create the container directories.
        fs::create_dir(&container.dir)?;
        fs::create_dir(&container.fs_dir)?;

        Ok(container)
    }

    fn generate_uuid() -> Result<String, Utf8Error> {
        // Generate a UUID.
        let mut buf: [u8; UUID_SIZE] = [0; UUID_SIZE];
        let uuid = Uuid::new_v4().to_simple();
        uuid.encode_lower(&mut buf);

        // Select the first 12 bytes from the generated UUID.
        return Ok(std::str::from_utf8(&buf)?[..CONTAINER_UUID_SIZE].to_owned());
    }
}
