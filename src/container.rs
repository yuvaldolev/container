use std::fs;
use std::path::PathBuf;
use std::str::Utf8Error;

use btrfsutil::subvolume::Subvolume;
use uuid::Uuid;

use crate::image::Image;
use crate::settings::Settings;

const FS_DIR: &str = "fs";

const UUID_SIZE: usize = 32;
const CONTAINER_UUID_SIZE: usize = 12;

#[derive(Debug)]
pub struct Container {
    pub uuid: String,
    dir: PathBuf,
    pub fs: Subvolume,
}

impl Container {
    pub fn new(image: &Image, settings: &Settings) -> anyhow::Result<Self> {
        // Generate the container's UUID.
        let uuid = Self::generate_uuid()?;

        // Generate the container's directory.
        let mut dir = PathBuf::new();
        dir.push(&settings.disk.containers_dir);
        dir.push(&uuid);
        fs::create_dir(&dir)?;

        // Create the container's file system snapshot.
        let fs_dir = dir.join(FS_DIR);
        let fs = image.subvolume.snapshot(fs_dir, None, None)?;

        Ok(Self { uuid, dir, fs })
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
