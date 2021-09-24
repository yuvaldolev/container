use std::path::PathBuf;
use std::str::Utf8Error;

use uuid::Uuid;

const ROOT_DIR: &str = "/var/container";
const CONTAINERS_DIR: &str = "containers";
const FS_DIR: &str = "fs";

const UUID_SIZE: usize = 32;
const CONTAINER_UUID_SIZE: usize = 12;

#[derive(Debug)]
pub struct Container {
    image: String,
    uuid: String,
    dir: PathBuf,
    fs_dir: PathBuf,
}

impl Container {
    pub fn new(image: String) -> anyhow::Result<Self> {
        // Generate the container's uuid.
        let uuid = Self::generate_uuid()?;

        // Generate the container's directory paths.
        let mut dir = PathBuf::new();
        dir.push(ROOT_DIR);
        dir.push(CONTAINERS_DIR);
        dir.push(&uuid);

        let fs_dir = dir.join(FS_DIR);

        Ok(Self {
            image,
            uuid,
            dir,
            fs_dir,
        })
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
