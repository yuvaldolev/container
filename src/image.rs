use std::fs;
use std::io;
use std::path::PathBuf;

use btrfsutil::subvolume::Subvolume;

use crate::settings::Settings;

#[derive(Debug, Clone)]
pub struct Image {
    name: String,
    pub subvolume: Subvolume,
}

impl Image {
    pub fn new(name: String, settings: &Settings) -> anyhow::Result<Self> {
        // Generate the image path.
        let mut path = PathBuf::new();
        path.push(&settings.disk.images_dir);
        let mut image_parts = name.split(':').collect::<Vec<_>>();
        match image_parts.len() {
            0 => return Err(io::Error::new(io::ErrorKind::InvalidInput, "Empty image").into()),
            1 => image_parts.push("latest"),
            2 => (),
            _ => return Err(io::Error::new(io::ErrorKind::InvalidInput, "Invalid image").into()),
        }
        for part in &image_parts {
            // Check if the image part is invalid.
            if part.is_empty() {
                return Err(io::Error::new(io::ErrorKind::InvalidInput, "Invalid image").into());
            }

            // Append the part to the image path.
            path.push(part);
        }

        // Validate that an image exists at the specified path.
        if let Err(_) = fs::metadata(&path) {
            return Err(io::Error::new(io::ErrorKind::NotFound, "Image does not exist").into());
        }

        return Ok(Self {
            name,
            subvolume: Subvolume::get(path)?,
        });
    }
}
