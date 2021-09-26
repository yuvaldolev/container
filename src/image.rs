use std::fs;
use std::io;
use std::path::PathBuf;

use crate::settings::Settings;

pub struct Image {
    name: String,
    path: PathBuf,
}

impl Image {
    pub fn new(name: String, settings: &Settings) -> io::Result<Self> {
        let mut image = Self {
            name,
            path: PathBuf::new(),
        };

        // Generate the image path.
        image.path.push(&settings.disk.images_dir);
        let mut image_parts = image.name.split(':').collect::<Vec<_>>();
        match image_parts.len() {
            0 => return Err(io::Error::new(io::ErrorKind::InvalidInput, "Empty image")),
            1 => image_parts.push("latest"),
            2 => (),
            _ => return Err(io::Error::new(io::ErrorKind::InvalidInput, "Invalid image")),
        }
        for part in &image_parts {
            // Check if the image part is invalid.
            if part.is_empty() {
                return Err(io::Error::new(io::ErrorKind::InvalidInput, "Invalid image"));
            }

            // Append the part to the image path.
            image.path.push(part);
        }

        // Validate that an image exists at the specified path.
        if let Err(_) = fs::metadata(&image.path) {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "Image does not exist",
            ));
        }

        return Ok(image);
    }
}
